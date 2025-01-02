use alloy::consensus::{Signed, Typed2718};
use alloy::network::eip2718::{Decodable2718, Encodable2718};
use alloy::rlp::{Encodable, Header};
use serde::{Deserialize, Serialize};

use super::unsigned_tx::eip712::TxEip712;

/// Transaction envelope is a wrapper around the transaction data.
/// See [`alloy::consensus::TxEnvelope`](https://docs.rs/alloy/latest/alloy/consensus/enum.TxEnvelope.html) for more details.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    into = "serde_from::TaggedTxEnvelope",
    from = "serde_from::MaybeTaggedTxEnvelope"
)]
pub enum TxEnvelope {
    /// Ethereum-native transaction.
    Native(alloy::consensus::TxEnvelope),
    /// ZKsync-native EIP712 transaction.
    Eip712(Signed<TxEip712>),
}

impl TxEnvelope {
    /// Returns true if the transaction is a legacy transaction.
    #[inline]
    pub const fn is_legacy(&self) -> bool {
        match self {
            Self::Native(inner) => inner.is_legacy(),
            Self::Eip712(_) => false,
        }
    }

    /// Returns true if the transaction is an EIP-2930 transaction.
    #[inline]
    pub const fn is_eip2930(&self) -> bool {
        match self {
            Self::Native(inner) => inner.is_eip2930(),
            Self::Eip712(_) => false,
        }
    }

    /// Returns true if the transaction is an EIP-1559 transaction.
    #[inline]
    pub const fn is_eip1559(&self) -> bool {
        match self {
            Self::Native(inner) => inner.is_eip1559(),
            Self::Eip712(_) => false,
        }
    }

    /// Returns true if the transaction is an EIP-4844 transaction.
    #[inline]
    pub const fn is_eip4844(&self) -> bool {
        match self {
            Self::Native(inner) => inner.is_eip4844(),
            Self::Eip712(_) => false,
        }
    }

    /// Returns true if the transaction is an EIP-7702 transaction.
    #[inline]
    pub const fn is_eip7702(&self) -> bool {
        match self {
            Self::Native(inner) => inner.is_eip7702(),
            Self::Eip712(_) => false,
        }
    }

    /// Returns true if the transaction is an EIP-712 transaction.
    #[inline]
    pub const fn is_eip712(&self) -> bool {
        matches!(self, Self::Eip712(_))
    }

    /// Returns true if the transaction is replay protected.
    ///
    /// All non-legacy transactions are replay protected, as the chain id is
    /// included in the transaction body. Legacy transactions are considered
    /// replay protected if the `v` value is not 27 or 28, according to the
    /// rules of [EIP-155].
    ///
    /// [EIP-155]: https://eips.ethereum.org/EIPS/eip-155
    #[inline]
    pub const fn is_replay_protected(&self) -> bool {
        match self {
            Self::Native(inner) => inner.is_replay_protected(),
            Self::Eip712(_) => true,
        }
    }

    /// Returns the [`TxLegacy`] variant if the transaction is a legacy transaction.
    pub const fn as_legacy(&self) -> Option<&Signed<alloy::consensus::TxLegacy>> {
        match self {
            Self::Native(inner) => inner.as_legacy(),
            Self::Eip712(_) => None,
        }
    }

    /// Returns the [`TxEip2930`] variant if the transaction is an EIP-2930 transaction.
    pub const fn as_eip2930(&self) -> Option<&Signed<alloy::consensus::TxEip2930>> {
        match self {
            Self::Native(inner) => inner.as_eip2930(),
            Self::Eip712(_) => None,
        }
    }

    /// Returns the [`TxEip1559`] variant if the transaction is an EIP-1559 transaction.
    pub const fn as_eip1559(&self) -> Option<&Signed<alloy::consensus::TxEip1559>> {
        match self {
            Self::Native(inner) => inner.as_eip1559(),
            Self::Eip712(_) => None,
        }
    }

    /// Returns the [`TxEip4844`] variant if the transaction is an EIP-4844 transaction.
    pub const fn as_eip4844(&self) -> Option<&Signed<alloy::consensus::TxEip4844Variant>> {
        match self {
            Self::Native(inner) => inner.as_eip4844(),
            Self::Eip712(_) => None,
        }
    }

    /// Returns the [`TxEip7702`] variant if the transaction is an EIP-7702 transaction.
    pub const fn as_eip7702(&self) -> Option<&Signed<alloy::consensus::TxEip7702>> {
        match self {
            Self::Native(inner) => inner.as_eip7702(),
            Self::Eip712(_) => None,
        }
    }

    /// Returns the [`TxEip712`] variant if the transaction is an EIP-712 transaction.
    pub const fn as_eip712(&self) -> Option<&Signed<TxEip712>> {
        match self {
            Self::Native(_) => None,
            Self::Eip712(inner) => Some(inner),
        }
    }

    /// Calculate the signing hash for the transaction.
    pub fn signature_hash(&self) -> alloy::primitives::B256 {
        match self {
            Self::Native(inner) => inner.signature_hash(),
            Self::Eip712(inner) => inner.signature_hash(),
        }
    }

    /// Return the reference to signature.
    pub const fn signature(&self) -> &alloy::primitives::PrimitiveSignature {
        match self {
            Self::Native(inner) => inner.signature(),
            Self::Eip712(inner) => inner.signature(),
        }
    }

    /// Return the hash of the inner Signed.
    #[doc(alias = "transaction_hash")]
    pub const fn tx_hash(&self) -> alloy::primitives::B256 {
        match self {
            Self::Native(inner) => *inner.tx_hash(),
            Self::Eip712(inner) => *inner.hash(),
        }
    }

    /// Return the [`TxType`] of the inner txn.
    #[doc(alias = "transaction_type")]
    pub const fn tx_type(&self) -> crate::network::tx_type::TxType {
        match self {
            Self::Native(inner) => match inner.tx_type() {
                alloy::consensus::TxType::Legacy => crate::network::tx_type::TxType::Legacy,
                alloy::consensus::TxType::Eip2930 => crate::network::tx_type::TxType::Eip2930,
                alloy::consensus::TxType::Eip1559 => crate::network::tx_type::TxType::Eip1559,
                alloy::consensus::TxType::Eip4844 => crate::network::tx_type::TxType::Eip4844,
                alloy::consensus::TxType::Eip7702 => crate::network::tx_type::TxType::Eip7702,
            },
            Self::Eip712(_) => crate::network::tx_type::TxType::Eip712,
        }
    }

    /// Return the length of the inner txn, including type byte length
    pub fn eip2718_encoded_length(&self) -> usize {
        match self {
            Self::Native(inner) => inner.eip2718_encoded_length(),
            Self::Eip712(inner) => inner.tx().encoded_length(inner.signature()),
        }
    }
}

impl Typed2718 for TxEnvelope {
    fn ty(&self) -> u8 {
        match self {
            Self::Native(inner) => inner.ty(),
            Self::Eip712(inner) => inner.tx().tx_type() as u8,
        }
    }
}

impl Encodable2718 for TxEnvelope {
    fn type_flag(&self) -> Option<u8> {
        match self {
            Self::Native(inner) => inner.type_flag(),
            Self::Eip712(inner) => Some(inner.tx().tx_type() as u8),
        }
    }

    fn encode_2718_len(&self) -> usize {
        match self {
            Self::Native(inner) => inner.encode_2718_len(),
            Self::Eip712(inner) => {
                let payload_length = inner.tx().fields_len()
                    + inner.signature().rlp_rs_len()
                    + inner.signature().v().length();
                Header {
                    list: true,
                    payload_length,
                }
                .length()
                    + payload_length
            }
        }
    }

    fn encode_2718(&self, out: &mut dyn alloy::primitives::bytes::BufMut) {
        match self {
            Self::Native(inner) => inner.encode_2718(out),
            Self::Eip712(tx) => {
                tx.tx().encode_with_signature(tx.signature(), out);
            }
        }
    }
}

impl Decodable2718 for TxEnvelope {
    fn typed_decode(ty: u8, buf: &mut &[u8]) -> alloy::network::eip2718::Eip2718Result<Self> {
        let inner = alloy::consensus::TxEnvelope::typed_decode(ty, buf)?;
        Ok(Self::Native(inner))
    }

    fn fallback_decode(buf: &mut &[u8]) -> alloy::network::eip2718::Eip2718Result<Self> {
        let inner = alloy::consensus::TxEnvelope::fallback_decode(buf)?;
        Ok(Self::Native(inner))
    }
}

impl AsRef<dyn alloy::consensus::Transaction> for TxEnvelope {
    fn as_ref(&self) -> &dyn alloy::consensus::Transaction {
        match self {
            TxEnvelope::Native(inner) => inner,
            TxEnvelope::Eip712(signed_inner) => signed_inner.tx(),
        }
    }
}

impl alloy::consensus::Transaction for TxEnvelope {
    fn chain_id(&self) -> Option<alloy::primitives::ChainId> {
        self.as_ref().chain_id()
    }

    fn nonce(&self) -> u64 {
        self.as_ref().nonce()
    }

    fn gas_limit(&self) -> u64 {
        self.as_ref().gas_limit()
    }

    fn gas_price(&self) -> Option<u128> {
        self.as_ref().gas_price()
    }

    fn max_fee_per_gas(&self) -> u128 {
        self.as_ref().max_fee_per_gas()
    }

    fn max_priority_fee_per_gas(&self) -> Option<u128> {
        self.as_ref().max_priority_fee_per_gas()
    }

    fn max_fee_per_blob_gas(&self) -> Option<u128> {
        self.as_ref().max_fee_per_blob_gas()
    }

    fn priority_fee_or_price(&self) -> u128 {
        self.as_ref().priority_fee_or_price()
    }

    fn effective_gas_price(&self, base_fee: Option<u64>) -> u128 {
        self.as_ref().effective_gas_price(base_fee)
    }

    fn is_dynamic_fee(&self) -> bool {
        self.as_ref().is_dynamic_fee()
    }

    fn kind(&self) -> alloy::primitives::TxKind {
        self.as_ref().kind()
    }

    fn is_create(&self) -> bool {
        self.as_ref().is_create()
    }

    fn value(&self) -> alloy::primitives::U256 {
        self.as_ref().value()
    }

    fn input(&self) -> &alloy::primitives::Bytes {
        self.as_ref().input()
    }

    fn access_list(&self) -> Option<&alloy::rpc::types::AccessList> {
        self.as_ref().access_list()
    }

    fn blob_versioned_hashes(&self) -> Option<&[alloy::primitives::B256]> {
        self.as_ref().blob_versioned_hashes()
    }

    fn authorization_list(&self) -> Option<&[alloy::eips::eip7702::SignedAuthorization]> {
        self.as_ref().authorization_list()
    }
}

mod serde_from {
    //! NB: Why do we need this?
    //!
    //! We are following the same approach as [`alloy::consensus::TxEnvelope`] but with an additional
    //! ZKsync-specific transaction type (`type: "0x71"`).
    //!
    //! Because the tag may be missing, we need an abstraction over tagged (with
    //! type) and untagged (always legacy). This is [`MaybeTaggedTxEnvelope`].
    //!
    //! The tagged variant is [`TaggedTxEnvelope`], which always has a type tag.
    //!
    //! We serialize via [`TaggedTxEnvelope`] and deserialize via
    //! [`MaybeTaggedTxEnvelope`].
    use crate::network::tx_envelope::TxEnvelope;
    use crate::network::unsigned_tx::eip712::TxEip712;
    use alloy::consensus::{Signed, TxEip1559, TxEip2930, TxEip4844Variant, TxEip7702, TxLegacy};

    #[derive(Debug, serde::Deserialize)]
    #[serde(untagged)]
    pub(crate) enum MaybeTaggedTxEnvelope {
        Tagged(TaggedTxEnvelope),
        Untagged {
            #[serde(
                default,
                rename = "type",
                deserialize_with = "alloy::serde::reject_if_some"
            )]
            _ty: Option<()>,
            #[serde(flatten, with = "alloy::consensus::transaction::signed_legacy_serde")]
            tx: Signed<TxLegacy>,
        },
    }

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    #[serde(tag = "type")]
    pub(crate) enum TaggedTxEnvelope {
        // Native transaction types below
        #[serde(
            rename = "0x0",
            alias = "0x00",
            with = "alloy::consensus::transaction::signed_legacy_serde"
        )]
        Legacy(Signed<TxLegacy>),
        #[serde(rename = "0x1", alias = "0x01")]
        Eip2930(Signed<TxEip2930>),
        #[serde(rename = "0x2", alias = "0x02")]
        Eip1559(Signed<TxEip1559>),
        #[serde(rename = "0x3", alias = "0x03")]
        Eip4844(Signed<TxEip4844Variant>),
        #[serde(rename = "0x4", alias = "0x04")]
        Eip7702(Signed<TxEip7702>),
        // Custom ZKsync transaction type
        #[serde(rename = "0x71")]
        Eip712(Signed<TxEip712>),
    }

    impl From<MaybeTaggedTxEnvelope> for TxEnvelope {
        fn from(value: MaybeTaggedTxEnvelope) -> Self {
            match value {
                MaybeTaggedTxEnvelope::Tagged(tagged) => tagged.into(),
                MaybeTaggedTxEnvelope::Untagged { tx, .. } => {
                    Self::Native(alloy::consensus::TxEnvelope::Legacy(tx))
                }
            }
        }
    }

    impl From<TaggedTxEnvelope> for TxEnvelope {
        fn from(value: TaggedTxEnvelope) -> Self {
            match value {
                TaggedTxEnvelope::Legacy(signed) => {
                    Self::Native(alloy::consensus::TxEnvelope::Legacy(signed))
                }
                TaggedTxEnvelope::Eip2930(signed) => {
                    Self::Native(alloy::consensus::TxEnvelope::Eip2930(signed))
                }
                TaggedTxEnvelope::Eip1559(signed) => {
                    Self::Native(alloy::consensus::TxEnvelope::Eip1559(signed))
                }
                TaggedTxEnvelope::Eip4844(signed) => {
                    Self::Native(alloy::consensus::TxEnvelope::Eip4844(signed))
                }
                TaggedTxEnvelope::Eip7702(signed) => {
                    Self::Native(alloy::consensus::TxEnvelope::Eip7702(signed))
                }
                TaggedTxEnvelope::Eip712(signed) => Self::Eip712(signed),
            }
        }
    }

    impl From<TxEnvelope> for TaggedTxEnvelope {
        fn from(value: TxEnvelope) -> Self {
            match value {
                TxEnvelope::Native(alloy::consensus::TxEnvelope::Legacy(signed)) => {
                    Self::Legacy(signed)
                }
                TxEnvelope::Native(alloy::consensus::TxEnvelope::Eip2930(signed)) => {
                    Self::Eip2930(signed)
                }
                TxEnvelope::Native(alloy::consensus::TxEnvelope::Eip1559(signed)) => {
                    Self::Eip1559(signed)
                }
                TxEnvelope::Native(alloy::consensus::TxEnvelope::Eip4844(signed)) => {
                    Self::Eip4844(signed)
                }
                TxEnvelope::Native(alloy::consensus::TxEnvelope::Eip7702(signed)) => {
                    Self::Eip7702(signed)
                }
                TxEnvelope::Native(tx) => panic!(
                    "Unsupported native Ethereum transaction type: {}",
                    tx.tx_type()
                ),
                TxEnvelope::Eip712(signed) => Self::Eip712(signed),
            }
        }
    }
}
