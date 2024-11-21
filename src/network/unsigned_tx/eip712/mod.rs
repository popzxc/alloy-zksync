use alloy::consensus::{SignableTransaction, Signed, Transaction};
use alloy::primitives::PrimitiveSignature as Signature;
use alloy::primitives::{keccak256, Address, Bytes, ChainId, TxKind, U256};
use alloy::rlp::{BufMut, Decodable, Encodable, Header};
use alloy::rpc::types::TransactionInput;
use serde::{Deserialize, Serialize};

use crate::network::tx_type::TxType;

pub use self::meta::{Eip712Meta, PaymasterParams};
pub use self::utils::{hash_bytecode, BytecodeHashError};

mod meta;
mod signing;
mod utils;

/// A ZKsync-native transaction type with additional fields.
/// Besides additional fields, represents an EIP-1559 transaction.
///
/// Note: Unlike EIP-1559, does not have `access_list` field.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[doc(
    alias = "Eip712Transaction",
    alias = "TransactionEip712",
    alias = "Eip712Tx"
)]
pub struct TxEip712 {
    /// EIP-155: Simple replay attack protection
    #[serde(with = "alloy::serde::quantity")]
    pub chain_id: ChainId,
    /// A scalar value equal to the number of transactions sent by the sender; formally Tn.
    // TODO: Explain composite nonce?
    pub nonce: U256,
    /// A scalar value equal to the maximum
    /// amount of gas that should be used in executing
    /// this transaction. This is paid up-front, before any
    /// computation is done and may not be increased
    /// later; formally Tg.
    #[serde(with = "alloy::serde::quantity")]
    pub gas_limit: u64,
    /// A scalar value equal to the maximum
    /// amount of gas that should be used in executing
    /// this transaction. This is paid up-front, before any
    /// computation is done and may not be increased
    /// later; formally Tg.
    ///
    /// As ethereum circulation is around 120mil eth as of 2022 that is around
    /// 120000000000000000000000000 wei we are safe to use u128 as its max number is:
    /// 340282366920938463463374607431768211455
    ///
    /// This is also known as `GasFeeCap`
    #[serde(with = "alloy::serde::quantity")]
    pub max_fee_per_gas: u128,
    /// Max Priority fee that transaction is paying
    ///
    /// As ethereum circulation is around 120mil eth as of 2022 that is around
    /// 120000000000000000000000000 wei we are safe to use u128 as its max number is:
    /// 340282366920938463463374607431768211455
    ///
    /// This is also known as `GasTipCap`
    #[serde(with = "alloy::serde::quantity")]
    pub max_priority_fee_per_gas: u128, // TODO: Should be option
    /// Address of the receiver of the message.
    /// Unlike with other transactions, this field must be present.
    /// In case of the contract deployment, the address should be set to the deployer system contract,
    /// and the payload should contain ABI-encoded salt, contract bytecode hash, and constructor arguments.
    pub to: Address,
    // TODO: document
    pub from: Address,
    /// A scalar value equal to the number of Wei to
    /// be transferred to the message call’s recipient or,
    /// in the case of contract creation, as an endowment
    /// to the newly created account; formally Tv.
    pub value: U256,
    /// Input has two uses depending if transaction is Create or Call (if `to` field is None or
    /// Some). pub init: An unlimited size byte array specifying the
    /// EVM-code for the account initialisation procedure CREATE,
    /// data: An unlimited size byte array specifying the
    /// input data of the message call, formally Td.
    pub input: Bytes,
    /// ZKsync-specific fields.
    pub eip712_meta: Eip712Meta,
}

impl TxEip712 {
    /// Returns the effective gas price for the given `base_fee`.
    pub const fn effective_gas_price(&self, base_fee: Option<u64>) -> u128 {
        match base_fee {
            None => self.max_fee_per_gas,
            Some(base_fee) => {
                // if the tip is greater than the max priority fee per gas, set it to the max
                // priority fee per gas + base fee
                let tip = self.max_fee_per_gas.saturating_sub(base_fee as u128);
                if tip > self.max_priority_fee_per_gas {
                    self.max_priority_fee_per_gas + base_fee as u128
                } else {
                    // otherwise return the max fee per gas
                    self.max_fee_per_gas
                }
            }
        }
    }

    /// Inner encoding function that is used for both rlp [`Encodable`] trait and for calculating
    /// hash that for eip2718 does not require a rlp header.
    pub(crate) fn encode_with_signature(
        &self,
        signature: &Signature,
        out: &mut dyn BufMut,
        // with_header: bool,
    ) {
        // if with_header {
        //     let payload_length = self.payload_length_unoptimized(signature);
        //     Header {
        //         list: false,
        //         payload_length: 1
        //             + Header {
        //                 list: true,
        //                 payload_length,
        //             }
        //             .length()
        //             + payload_length,
        //     }
        //     .encode(out);
        // }
        out.put_u8(self.tx_type() as u8);
        self.encode_with_signature_fields(signature, out);
    }

    /// Decodes the transaction from RLP bytes, including the signature.
    ///
    /// This __does not__ expect the bytes to start with a transaction type byte or string
    /// header.
    ///
    /// This __does__ expect the bytes to start with a list header and include a signature.
    #[doc(hidden)]
    pub fn decode_signed_fields(buf: &mut &[u8]) -> alloy::rlp::Result<Signed<Self>> {
        let header = Header::decode(buf)?;
        if !header.list {
            return Err(alloy::rlp::Error::UnexpectedString);
        }

        // record original length so we can check encoding
        let original_len = buf.len();

        let nonce = Decodable::decode(buf)?;
        let max_priority_fee_per_gas = Decodable::decode(buf)?;
        let max_fee_per_gas = Decodable::decode(buf)?;
        let gas_limit = Decodable::decode(buf)?;
        let to = Decodable::decode(buf)?;
        let value = Decodable::decode(buf)?;
        let input = Decodable::decode(buf)?;
        let signature = Signature::decode_rlp_vrs(buf, bool::decode)?;
        let chain_id = Decodable::decode(buf)?;
        let from = Decodable::decode(buf)?;
        let eip712_meta = Decodable::decode(buf)?;

        let tx = Self {
            chain_id,
            nonce,
            gas_limit,
            max_fee_per_gas,
            max_priority_fee_per_gas,
            to,
            from,
            value,
            input,
            eip712_meta,
        };

        // Context: `SignableTransaction` is commented out for now.
        let signed = tx.into_signed(signature);

        if buf.len() + header.payload_length != original_len {
            return Err(alloy::rlp::Error::ListLengthMismatch {
                expected: header.payload_length,
                got: original_len - buf.len(),
            });
        }

        Ok(signed)
    }

    pub(crate) fn fields_len(&self) -> usize {
        self.nonce.length()
            + self.max_priority_fee_per_gas.length()
            + self.max_fee_per_gas.length()
            + self.gas_limit.length()
            + self.to.length()
            + self.value.length()
            + self.input.length()
            + self.chain_id.length()
            + self.from.length()
            + self.eip712_meta.length()
    }

    /// Encodes the transaction from RLP bytes, including the signature. This __does not__ encode a
    /// tx type byte or string header.
    ///
    /// This __does__ encode a list header and include a signature.
    pub(crate) fn encode_with_signature_fields(&self, signature: &Signature, out: &mut dyn BufMut) {
        let payload_length = self.fields_len() + signature.rlp_rs_len() + signature.v().length();
        let header = Header {
            list: true,
            payload_length,
        };
        header.encode(out);

        self.nonce.encode(out);
        self.max_priority_fee_per_gas.encode(out);
        self.max_fee_per_gas.encode(out);
        self.gas_limit.encode(out);
        self.to.encode(out);
        self.value.encode(out);
        self.input.0.encode(out);
        signature.write_rlp_vrs(out, signature.v());
        self.chain_id.encode(out);
        self.from.encode(out);
        self.eip712_meta.encode(out);
    }

    /// Gets the length of the encoded header.
    pub(crate) fn encoded_length(&self, signature: &Signature) -> usize {
        let payload_length = self.fields_len() + signature.rlp_rs_len() + signature.v().length();
        alloy::rlp::length_of_length(payload_length) + payload_length
    }

    /// Get transaction type
    #[doc(alias = "transaction_type")]
    pub(crate) const fn tx_type(&self) -> TxType {
        TxType::Eip712
    }

    // /// Calculates a heuristic for the in-memory size of the [TxEip712] transaction.
    // #[inline]
    // pub fn size(&self) -> usize {
    //     mem::size_of::<ChainId>() + // chain_id
    //     mem::size_of::<u64>() + // nonce
    //     mem::size_of::<u64>() + // gas_limit
    //     mem::size_of::<u128>() + // max_fee_per_gas
    //     mem::size_of::<u128>() + // max_priority_fee_per_gas
    //     self.to.size() + // to
    //     mem::size_of::<U256>() + // value
    //     self.input.len() // input
    // }
}

impl Transaction for TxEip712 {
    fn chain_id(&self) -> Option<ChainId> {
        Some(self.chain_id)
    }

    fn nonce(&self) -> u64 {
        // TODO: Better interface for nonce decomposition?
        (self.nonce % U256::from(u64::MAX)).try_into().unwrap()
    }

    fn gas_limit(&self) -> u64 {
        self.gas_limit
    }

    fn gas_price(&self) -> Option<u128> {
        None
    }

    fn to(&self) -> Option<Address> {
        self.to.into()
    }

    fn value(&self) -> U256 {
        self.value
    }

    fn input(&self) -> &Bytes {
        &self.input
    }

    fn max_fee_per_gas(&self) -> u128 {
        self.max_fee_per_gas
    }

    fn max_priority_fee_per_gas(&self) -> Option<u128> {
        Some(self.max_priority_fee_per_gas)
    }

    fn max_fee_per_blob_gas(&self) -> Option<u128> {
        None
    }

    fn priority_fee_or_price(&self) -> u128 {
        todo!()
    }

    fn ty(&self) -> u8 {
        self.tx_type() as u8
    }

    fn access_list(&self) -> Option<&alloy::rpc::types::AccessList> {
        None
    }

    fn blob_versioned_hashes(&self) -> Option<&[alloy::primitives::B256]> {
        None
    }

    fn authorization_list(&self) -> Option<&[alloy::eips::eip7702::SignedAuthorization]> {
        None
    }

    fn kind(&self) -> TxKind {
        self.to.into()
    }

    fn effective_gas_price(&self, base_fee: Option<u64>) -> u128 {
        self.effective_gas_price(base_fee)
    }

    fn is_dynamic_fee(&self) -> bool {
        false
    }
}

// Context: Encodable/Decodable assume that there is no signature in the transaction
impl SignableTransaction<Signature> for TxEip712 {
    fn set_chain_id(&mut self, chain_id: ChainId) {
        self.chain_id = chain_id;
    }

    fn encode_for_signing(&self, out: &mut dyn alloy::rlp::BufMut) {
        // Reimplementation of `SolStruct::eip712_signing_hash`
        out.put_u8(0x19);
        out.put_u8(0x01);
        out.put_slice(self.domain_hash().as_slice());
        out.put_slice(self.eip712_hash_struct().as_slice());
    }

    fn payload_len_for_signature(&self) -> usize {
        // 2 bytes for the header, 32 bytes for the domain hash, 32 bytes for the transaction hash
        2 + 32 + 32
    }

    fn into_signed(self, signature: Signature) -> Signed<Self> {
        // Drop any v chain id value to ensure the signature format is correct at the time of
        // combination for an EIP-1559 transaction. V should indicate the y-parity of the
        // signature.
        let signature = signature.with_parity(true);

        let mut buf = [0u8; 64];
        buf[..32].copy_from_slice(self.signature_hash().as_slice());
        buf[32..].copy_from_slice(keccak256(signature.as_bytes()).as_slice());
        let hash = keccak256(buf);

        Signed::new_unchecked(self, signature, hash)
    }
}

// // Context: encoding is implemented for the purpose of EIP-712 signing.
// impl Encodable for TxEip712 {
//     fn encode(&self, out: &mut dyn BufMut) {
//         Header {
//             list: true,
//             payload_length: self.fields_len(),
//         }
//         .encode(out);
//         self.encode_fields(out);
//     }

//     fn length(&self) -> usize {
//         let payload_length = self.fields_len();
//         Header {
//             list: true,
//             payload_length,
//         }
//         .length()
//             + payload_length
//     }
// }

// impl Decodable for TxEip712 {
//     fn decode(data: &mut &[u8]) -> alloy::rlp::Result<Self> {
//         let header = Header::decode(data)?;
//         let remaining_len = data.len();

//         if header.payload_length > remaining_len {
//             return Err(alloy::rlp::Error::InputTooShort);
//         }

//         println!("decode fields");
//         Self::decode_fields(data)
//     }
// }

impl From<TxEip712> for alloy::rpc::types::transaction::TransactionRequest {
    fn from(tx: TxEip712) -> Self {
        Self {
            transaction_type: Some(tx.tx_type() as u8),
            chain_id: Some(tx.chain_id),
            nonce: Some((tx.nonce % U256::from(u64::MAX)).try_into().unwrap()), // TODO: Is decomposed nonce fine here?
            gas: Some(tx.gas_limit),
            max_fee_per_gas: Some(tx.max_fee_per_gas),
            max_priority_fee_per_gas: Some(tx.max_priority_fee_per_gas),
            to: Some(tx.to.into()),
            from: Some(tx.from),
            value: Some(tx.value),
            input: TransactionInput::new(tx.input),
            access_list: None,
            blob_versioned_hashes: None,
            max_fee_per_blob_gas: None,
            gas_price: None,
            sidecar: None,
            authorization_list: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::network::unsigned_tx::eip712::{Eip712Meta, PaymasterParams};

    use super::TxEip712;
    use alloy::consensus::SignableTransaction;
    use alloy::hex::FromHex;
    use alloy::primitives::{
        address, hex, Address, Bytes, FixedBytes, PrimitiveSignature as Signature, B256, U256,
    };

    #[test]
    fn decode_eip712_tx() {
        // Does not have type byte.
        let encoded = hex::decode("f8b701800b0c940754b07d1ea3071c3ec9bd86b2aa6f1a59a514980a8301020380a0635f9ee3a1523de15fc8b72a0eea12f5247c6b6e2369ed158274587af6496599a030f7c66d1ed24fca92527e6974b85b07ec30fdd5c2d41eae46966224add965f982010e9409a6aa96b9a17d7f7ba3e3b19811c082aba9f1e304e1a0020202020202020202020202020202020202020202020202020202020202020283010203d694000000000000000000000000000000000000000080").unwrap();
        let signed_tx = TxEip712::decode_signed_fields(&mut &encoded[..]).unwrap();
        let tx = signed_tx.tx();
        assert_eq!(tx.chain_id, 270);
        assert_eq!(tx.nonce, U256::from(1));
        assert_eq!(tx.gas_limit, 12);
        assert_eq!(tx.max_fee_per_gas, 11);
        assert_eq!(tx.max_priority_fee_per_gas, 0);
        assert_eq!(tx.to, address!("0754b07d1ea3071c3ec9bd86b2aa6f1a59a51498"));
        assert_eq!(
            tx.from,
            address!("09a6aa96b9a17d7f7ba3e3b19811c082aba9f1e3")
        );
        assert_eq!(tx.value, U256::from(10));
        assert_eq!(tx.input, Bytes::from_hex("0x010203").unwrap());
        assert_eq!(tx.eip712_meta.gas_per_pubdata, U256::from(4));
        assert_eq!(
            tx.eip712_meta.factory_deps,
            vec![Bytes::from_hex(
                "0x0202020202020202020202020202020202020202020202020202020202020202"
            )
            .unwrap()]
        );
        assert_eq!(
            tx.eip712_meta.custom_signature,
            Some(Bytes::from_hex("0x010203").unwrap())
        );
        assert_eq!(
            tx.eip712_meta
                .paymaster_params
                .as_ref()
                .unwrap()
                .paymaster_input,
            Bytes::from_hex("0x").unwrap()
        );
        assert_eq!(
            tx.eip712_meta.paymaster_params.as_ref().unwrap().paymaster,
            address!("0000000000000000000000000000000000000000")
        );
    }

    #[test]
    fn decode_eip712_tx_with_paymaster() {
        // This is request to AA account with paymaster set and no signature.
        // Does not have type byte.
        let encoded = hex::decode("f9036580843b9aca00843b9aca0083989680949c1a3d7c98dbf89c7f5d167f2219c29c2fe775a780b903045abef77a000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000002e0000000000000000000000000000000000000000000000000000000000000008000000000000000000000000051ef809ffd89cf8056d4c17f0aff1b6f8257eb6000000000000000000000000000000000000000000000000000000000000001f4000000000000000000000000000000000000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000001e10100000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000010f0000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000002000000000000000000000000078cad996530109838eb016619f5931a03250489a000000000000000000000000aaf5f437fb0524492886fba64d703df15bf619ae000000000000000000000000000000000000000000000000000000000000010f00000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000064a41368620000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000568656c6c6f0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000080808082010f94b2a6e81272904caf680008078feb36336d9376b482c350c080d89499e12239cbf8112fbb3f7fd473d0558031abcbb5821234").unwrap();
        let signed_tx = TxEip712::decode_signed_fields(&mut &encoded[..]).unwrap();
        let tx = signed_tx.tx();
        assert_eq!(tx.chain_id, 271);
        assert_eq!(tx.nonce, U256::from(0));
        assert_eq!(tx.gas_limit, 10000000);
        assert_eq!(tx.max_fee_per_gas, 1000000000);
        assert_eq!(tx.max_priority_fee_per_gas, 1000000000);
        assert_eq!(tx.to, address!("9c1a3d7c98dbf89c7f5d167f2219c29c2fe775a7"));
        assert_eq!(
            tx.from,
            address!("b2a6e81272904caf680008078feb36336d9376b4")
        );
        assert_eq!(tx.value, U256::from(0));
        assert_eq!(tx.input, Bytes::from_hex("5abef77a000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000002e0000000000000000000000000000000000000000000000000000000000000008000000000000000000000000051ef809ffd89cf8056d4c17f0aff1b6f8257eb6000000000000000000000000000000000000000000000000000000000000001f4000000000000000000000000000000000000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000001e10100000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000010f0000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000002000000000000000000000000078cad996530109838eb016619f5931a03250489a000000000000000000000000aaf5f437fb0524492886fba64d703df15bf619ae000000000000000000000000000000000000000000000000000000000000010f00000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000064a41368620000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000568656c6c6f00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").unwrap());

        assert_eq!(tx.eip712_meta.gas_per_pubdata, U256::from(50000));
        assert_eq!(tx.eip712_meta.factory_deps, vec![] as Vec<Bytes>);
        assert_eq!(
            tx.eip712_meta.custom_signature,
            Some(Bytes::from_hex("0x").unwrap())
        );
        assert_eq!(
            tx.eip712_meta.paymaster_params.as_ref().unwrap().paymaster,
            address!("99E12239CBf8112fBB3f7Fd473d0558031abcbb5")
        );
        assert_eq!(
            tx.eip712_meta
                .paymaster_params
                .as_ref()
                .unwrap()
                .paymaster_input,
            Bytes::from_hex("0x1234").unwrap()
        );
    }

    #[test]
    fn test_eip712_tx1() {
        let eip712_meta = Eip712Meta {
            gas_per_pubdata: U256::from(4),
            factory_deps: vec![vec![2; 32].into()],
            custom_signature: Some(vec![].into()),
            paymaster_params: None,
        };
        let tx = TxEip712 {
            chain_id: 270,
            from: Address::from_str("0xe30f4fb40666753a7596d315f2f1f1d140d1508b").unwrap(),
            to: Address::from_str("0x82112600a140ceaa9d7da373bb65453f7d99af4b").unwrap(),
            nonce: U256::from(1),
            value: U256::from(10),
            gas_limit: 12,
            max_fee_per_gas: 11,
            max_priority_fee_per_gas: 0,
            input: vec![0x01, 0x02, 0x03].into(),
            eip712_meta,
        };
        let expected_signature_hash = FixedBytes::<32>::from_str(
            "0xfc76820a67d9b1b351f2ac661e6d2bcca1c67508ae4930e036f540fa135875fe",
        )
        .unwrap();
        assert_eq!(tx.signature_hash(), expected_signature_hash);

        let signature = Signature::from_str("0x3faf83b5451ad3001f96f577b0bb5dfcaa7769ab11908f281dc6b15c45a3986f0325197832aac9a7ab2f5a83873834d457e0d22c1e72377d45364c6968f8ac3b1c").unwrap();
        let recovered_signer = signature
            .recover_address_from_prehash(&tx.signature_hash())
            .unwrap();
        assert_eq!(recovered_signer, tx.from);

        let mut buf = Vec::new();
        tx.encode_with_signature_fields(&signature, &mut buf);
        let decoded = TxEip712::decode_signed_fields(&mut &buf[..]).unwrap();
        assert_eq!(decoded, tx.into_signed(signature));

        let expected_hash =
            B256::from_str("0xb85668399db249d62d06bbc59eace82e01364602fb7159e161ca810ff6ddbbf4")
                .unwrap();
        assert_eq!(*decoded.hash(), expected_hash);
    }

    #[test]
    fn test_eip712_tx_encode_decode_with_paymaster() {
        let eip712_meta = Eip712Meta {
            gas_per_pubdata: U256::from(4),
            factory_deps: vec![vec![2; 32].into()],
            custom_signature: Some(vec![].into()),
            paymaster_params: Some(PaymasterParams {
                paymaster: address!("99E12239CBf8112fBB3f7Fd473d0558031abcbb5"),
                paymaster_input: Bytes::from_hex("0x112233").unwrap(),
            }),
        };
        let tx = TxEip712 {
            chain_id: 270,
            from: Address::from_str("0xe30f4fb40666753a7596d315f2f1f1d140d1508b").unwrap(),
            to: Address::from_str("0x82112600a140ceaa9d7da373bb65453f7d99af4b").unwrap(),
            nonce: U256::from(1),
            value: U256::from(10),
            gas_limit: 12,
            max_fee_per_gas: 11,
            max_priority_fee_per_gas: 0,
            input: vec![0x01, 0x02, 0x03].into(),
            eip712_meta,
        };

        // This is a random signature, but that's ok.
        let signature = Signature::from_str("0x3faf83b5451ad3001f96f577b0bb5dfcaa7769ab11908f281dc6b15c45a3986f0325197832aac9a7ab2f5a83873834d457e0d22c1e72377d45364c6968f8ac3b1c").unwrap();

        let mut buf = Vec::new();
        tx.encode_with_signature_fields(&signature, &mut buf);
        let decoded = TxEip712::decode_signed_fields(&mut &buf[..]).unwrap();
        // Make sure that paymaster data was loaded correctly.
        assert_eq!(decoded, tx.into_signed(signature));
    }

    // #[test]
    // fn recover_signer_eip712() {
    //     let signer: Address = address!("dd6b8b3dc6b7ad97db52f08a275ff4483e024cea");
    //     let hash: B256 = b256!("0ec0b6a2df4d87424e5f6ad2a654e27aaeb7dac20ae9e8385cc09087ad532ee0");

    //     let tx =  TxEip712 {
    //         chain_id: 1,
    //         nonce: 0x42,
    //         gas_limit: 44386,
    //         to: address!("6069a6c32cf691f5982febae4faf8a6f3ab2f0f6").into(),
    //         value: U256::from(0_u64),
    //         input:  hex!("a22cb4650000000000000000000000005eee75727d804a2b13038928d36f8b188945a57a0000000000000000000000000000000000000000000000000000000000000000").into(),
    //         max_fee_per_gas: 0x4a817c800,
    //         max_priority_fee_per_gas: 0x3b9aca00,
    //         access_list: AccessList::default(),
    //         eip712_meta: Eip712Meta::default(),
    //     };

    //     let sig = Signature::from_scalars_and_parity(
    //         b256!("840cfc572845f5786e702984c2a582528cad4b49b2a10b9db1be7fca90058565"),
    //         b256!("25e7109ceb98168d95b09b18bbf6b685130e0562f233877d492b94eee0c5b6d1"),
    //         false,
    //     )
    //     .unwrap();

    //     assert_eq!(
    //         tx.signature_hash(),
    //         hex!("0d5688ac3897124635b6cf1bc0e29d6dfebceebdc10a54d74f2ef8b56535b682")
    //     );

    //     let signed_tx = tx.into_signed(sig);
    //     assert_eq!(*signed_tx.hash(), hash, "Expected same hash");
    //     // assert_eq!(
    //     //     signed_tx.recover_signer().unwrap(),
    //     //     signer,
    //     //     "Recovering signer should pass."
    //     // );
    // }

    // #[test]
    // fn encode_decode_eip712() {
    //     let hash: B256 = b256!("0ec0b6a2df4d87424e5f6ad2a654e27aaeb7dac20ae9e8385cc09087ad532ee0");

    //     let tx =  TxEip712 {
    //         chain_id: 1,
    //         nonce: 0x42,
    //         gas_limit: 44386,
    //         to: address!("6069a6c32cf691f5982febae4faf8a6f3ab2f0f6").into(),
    //         value: U256::from(0_u64),
    //         input:  hex!("a22cb4650000000000000000000000005eee75727d804a2b13038928d36f8b188945a57a0000000000000000000000000000000000000000000000000000000000000000").into(),
    //         max_fee_per_gas: 0x4a817c800,
    //         max_priority_fee_per_gas: 0x3b9aca00,
    //         access_list: AccessList::default(),
    //         eip712_meta: Eip712Meta::default(),
    //     };

    //     let sig = Signature::from_scalars_and_parity(
    //         b256!("840cfc572845f5786e702984c2a582528cad4b49b2a10b9db1be7fca90058565"),
    //         b256!("25e7109ceb98168d95b09b18bbf6b685130e0562f233877d492b94eee0c5b6d1"),
    //         false,
    //     )
    //     .unwrap();

    //     let mut buf = vec![];
    //     tx.encode_with_signature_fields(&sig, &mut buf);
    //     let decoded = TxEip712::decode_signed_fields(&mut &buf[..]).unwrap();
    //     assert_eq!(decoded, tx.into_signed(sig));
    //     assert_eq!(*decoded.hash(), hash);
    // }
}
