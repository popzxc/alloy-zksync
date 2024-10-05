use std::mem;

use alloy_consensus::{SignableTransaction, Signed, Transaction};
use alloy_eips::eip2930::AccessList;
use alloy_primitives::{keccak256, Address, Bytes, ChainId, TxKind, U256};
use alloy_rlp::{BufMut, Decodable, Encodable, Header};
use alloy_signer::Signature;
use meta::Eip712Meta;
use serde::{Deserialize, Serialize};

use crate::network::tx_type::TxType;

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
    #[serde(with = "alloy_serde::quantity")]
    pub chain_id: ChainId,
    /// A scalar value equal to the number of transactions sent by the sender; formally Tn.
    // TODO: Explain composite nonce?
    pub nonce: U256,
    /// A scalar value equal to the maximum
    /// amount of gas that should be used in executing
    /// this transaction. This is paid up-front, before any
    /// computation is done and may not be increased
    /// later; formally Tg.
    #[serde(with = "alloy_serde::quantity")]
    pub gas_limit: u128,
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
    #[serde(with = "alloy_serde::quantity")]
    pub max_fee_per_gas: u128,
    /// Max Priority fee that transaction is paying
    ///
    /// As ethereum circulation is around 120mil eth as of 2022 that is around
    /// 120000000000000000000000000 wei we are safe to use u128 as its max number is:
    /// 340282366920938463463374607431768211455
    ///
    /// This is also known as `GasTipCap`
    #[serde(with = "alloy_serde::quantity")]
    pub max_priority_fee_per_gas: u128, // TODO: Should be option
    /// The 160-bit address of the message call’s recipient or, for a contract creation
    /// transaction, ∅, used here to denote the only member of B0 ; formally Tt.
    #[serde(default, skip_serializing_if = "TxKind::is_create")]
    pub to: TxKind,
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

    /// Outputs the length of the transaction's fields, without a RLP header.
    #[doc(hidden)]
    pub fn fields_len(&self) -> usize {
        let mut len = 0;
        len += self.chain_id.length();
        len += self.nonce.length();
        len += self.max_priority_fee_per_gas.length();
        len += self.max_fee_per_gas.length();
        len += self.gas_limit.length();
        len += self.to.length();
        len += self.value.length();
        len += self.input.0.length();
        len
    }

    /// Encodes the transaction fields
    pub(crate) fn encode_fields_for_signing(&self, out: &mut dyn alloy_rlp::BufMut) {
        self.chain_id.encode(out);
        self.nonce.encode(out);
        self.max_priority_fee_per_gas.encode(out);
        self.max_fee_per_gas.encode(out);
        self.gas_limit.encode(out);
        self.to.encode(out);
        self.value.encode(out);
        self.input.0.encode(out);
    }

    /// Returns what the encoded length should be, if the transaction were RLP encoded with the
    /// given signature, depending on the value of `with_header`.
    ///
    /// If `with_header` is `true`, the payload length will include the RLP header length.
    /// If `with_header` is `false`, the payload length will not include the RLP header length.
    pub(crate) fn encoded_len_with_signature(
        &self,
        signature: &Signature,
        with_header: bool,
    ) -> usize {
        // this counts the tx fields and signature fields
        let payload_length = self.fields_len() + signature.rlp_vrs_len();

        // this counts:
        // * tx type byte
        // * inner header length
        // * inner payload length
        let inner_payload_length = 1
            + Header {
                list: true,
                payload_length,
            }
            .length()
            + payload_length;

        if with_header {
            // header length plus length of the above, wrapped with a string header
            Header {
                list: false,
                payload_length: inner_payload_length,
            }
            .length()
                + inner_payload_length
        } else {
            inner_payload_length
        }
    }

    /// Inner encoding function that is used for both rlp [`Encodable`] trait and for calculating
    /// hash that for eip2718 does not require a rlp header.
    #[doc(hidden)]
    pub fn encode_with_signature(
        &self,
        signature: &Signature,
        out: &mut dyn BufMut,
        with_header: bool,
    ) {
        let payload_length = self.fields_len() + signature.rlp_vrs_len();
        if with_header {
            Header {
                list: false,
                payload_length: 1
                    + Header {
                        list: true,
                        payload_length,
                    }
                    .length()
                    + payload_length,
            }
            .encode(out);
        }
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
    pub fn decode_signed_fields(buf: &mut &[u8]) -> alloy_rlp::Result<Signed<Self>> {
        let header = Header::decode(buf)?;
        if !header.list {
            return Err(alloy_rlp::Error::UnexpectedString);
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
        let signature = Signature::decode_rlp_vrs(buf)?;
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
            return Err(alloy_rlp::Error::ListLengthMismatch {
                expected: header.payload_length,
                got: original_len - buf.len(),
            });
        }

        Ok(signed)
    }

    /// Encodes the transaction from RLP bytes, including the signature. This __does not__ encode a
    /// tx type byte or string header.
    ///
    /// This __does__ encode a list header and include a signature.
    pub(crate) fn encode_with_signature_fields(&self, signature: &Signature, out: &mut dyn BufMut) {
        let payload_length = self.fields_len() + signature.rlp_vrs_len();
        let header = Header {
            list: true,
            payload_length,
        };
        header.encode(out);
        todo!("encode fields");
        signature.write_rlp_vrs(out);
    }

    /// Get transaction type
    #[doc(alias = "transaction_type")]
    pub(crate) const fn tx_type(&self) -> TxType {
        TxType::Eip712
    }

    /// Calculates a heuristic for the in-memory size of the [TxEip712] transaction.
    #[inline]
    pub fn size(&self) -> usize {
        mem::size_of::<ChainId>() + // chain_id
        mem::size_of::<u64>() + // nonce
        mem::size_of::<u64>() + // gas_limit
        mem::size_of::<u128>() + // max_fee_per_gas
        mem::size_of::<u128>() + // max_priority_fee_per_gas
        self.to.size() + // to
        mem::size_of::<U256>() + // value
        self.input.len() // input
    }
}

impl Transaction for TxEip712 {
    fn chain_id(&self) -> Option<ChainId> {
        Some(self.chain_id)
    }

    fn nonce(&self) -> u64 {
        // TODO: Better interface for nonce decomposition?
        (self.nonce % U256::from(u64::MAX)).try_into().unwrap()
    }

    fn gas_limit(&self) -> u128 {
        self.gas_limit
    }

    fn gas_price(&self) -> Option<u128> {
        None
    }

    fn to(&self) -> TxKind {
        self.to
    }

    fn value(&self) -> U256 {
        self.value
    }

    fn input(&self) -> &[u8] {
        &self.input
    }
}

// Context: Encodable/Decodable assume that there is no signature in the transaction
impl SignableTransaction<Signature> for TxEip712 {
    fn set_chain_id(&mut self, chain_id: ChainId) {
        self.chain_id = chain_id;
    }

    fn encode_for_signing(&self, out: &mut dyn alloy_rlp::BufMut) {
        // TODO: Cache values via Lazy?
        let eip712_domain_typehash =
            keccak256("EIP712Domain(string name,string version,uint256 chainId)");
        let eip712_transaction_type_hash = keccak256("Transaction(uint256 txType,uint256 from,uint256 to,uint256 gasLimit,uint256 gasPerPubdataByteLimit,uint256 maxFeePerGas,uint256 maxPriorityFeePerGas,uint256 paymaster,uint256 nonce,uint256 value,bytes data,bytes32[] factoryDeps,bytes paymasterInput)");

        // let factory_deps_hash =

        // out.put_u8(self.tx_type() as u8);
        // self.encode(out)
    }

    fn payload_len_for_signature(&self) -> usize {
        todo!()
        // self.length() + 1
    }

    fn into_signed(self, signature: Signature) -> Signed<Self> {
        // Drop any v chain id value to ensure the signature format is correct at the time of
        // combination for an EIP-1559 transaction. V should indicate the y-parity of the
        // signature.
        let signature = signature.with_parity_bool();

        let mut buf = Vec::with_capacity(self.encoded_len_with_signature(&signature, false));
        self.encode_with_signature(&signature, &mut buf, false);
        let hash = keccak256(&buf);

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
//     fn decode(data: &mut &[u8]) -> alloy_rlp::Result<Self> {
//         let header = Header::decode(data)?;
//         let remaining_len = data.len();

//         if header.payload_length > remaining_len {
//             return Err(alloy_rlp::Error::InputTooShort);
//         }

//         println!("decode fields");
//         Self::decode_fields(data)
//     }
// }

#[cfg(test)]
mod tests {
    use crate::network::unsigned_tx::eip712::Eip712Meta;

    use super::TxEip712;
    use alloy_consensus::SignableTransaction;
    use alloy_eips::eip2930::AccessList;
    use alloy_primitives::{address, b256, hex, Address, Signature, B256, U256};
    use alloy_rlp::Decodable;

    #[test]
    fn decode_eip712_tx() {
        // Does not have type byte.
        let encoded = hex::decode("f8b701800b0c940754b07d1ea3071c3ec9bd86b2aa6f1a59a514980a8301020380a0635f9ee3a1523de15fc8b72a0eea12f5247c6b6e2369ed158274587af6496599a030f7c66d1ed24fca92527e6974b85b07ec30fdd5c2d41eae46966224add965f982010e9409a6aa96b9a17d7f7ba3e3b19811c082aba9f1e304e1a0020202020202020202020202020202020202020202020202020202020202020283010203d694000000000000000000000000000000000000000080").unwrap();
        println!("222");
        let tx = TxEip712::decode_signed_fields(&mut &encoded[..]).unwrap();
        println!("Tx is {tx:#?}");
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
