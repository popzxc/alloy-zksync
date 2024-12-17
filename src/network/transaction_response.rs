use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    into = "serde_from::TransactionEither",
    from = "serde_from::TransactionEither"
)]
pub struct TransactionResponse {
    #[serde(flatten)]
    inner: alloy::rpc::types::transaction::Transaction<crate::network::tx_envelope::TxEnvelope>,
}

// impl alloy::consensus::Typed2718 for TransactionResponse {
//     fn ty(&self) -> u8 {
//         self.inner.inner.tx_type() as u8
//     }
// }

impl alloy::consensus::Transaction for TransactionResponse {
    fn chain_id(&self) -> Option<alloy::primitives::ChainId> {
        self.inner.chain_id()
    }

    fn nonce(&self) -> u64 {
        self.inner.nonce()
    }

    fn gas_limit(&self) -> u64 {
        self.inner.gas_limit()
    }

    fn gas_price(&self) -> Option<u128> {
        self.inner.gas_price()
    }

    fn max_fee_per_gas(&self) -> u128 {
        self.inner.max_fee_per_gas()
    }

    fn max_priority_fee_per_gas(&self) -> Option<u128> {
        self.inner.max_priority_fee_per_gas()
    }

    fn max_fee_per_blob_gas(&self) -> Option<u128> {
        self.inner.max_fee_per_blob_gas()
    }

    fn priority_fee_or_price(&self) -> u128 {
        self.inner.priority_fee_or_price()
    }

    fn to(&self) -> Option<alloy::primitives::Address> {
        self.inner.to()
    }

    fn is_create(&self) -> bool {
        self.inner.is_create()
    }

    fn value(&self) -> alloy::primitives::U256 {
        self.inner.value()
    }

    fn input(&self) -> &alloy::primitives::Bytes {
        self.inner.input()
    }

    fn access_list(&self) -> Option<&alloy::rpc::types::AccessList> {
        self.inner.access_list()
    }

    fn blob_versioned_hashes(&self) -> Option<&[alloy::primitives::B256]> {
        self.inner.blob_versioned_hashes()
    }

    fn authorization_list(&self) -> Option<&[alloy::eips::eip7702::SignedAuthorization]> {
        self.inner.authorization_list()
    }

    fn kind(&self) -> alloy::primitives::TxKind {
        self.inner.kind()
    }

    fn effective_gas_price(&self, base_fee: Option<u64>) -> u128 {
        self.inner.effective_gas_price(base_fee)
    }

    fn is_dynamic_fee(&self) -> bool {
        self.inner.is_dynamic_fee()
    }
}

impl alloy::consensus::Typed2718 for TransactionResponse {
    fn ty(&self) -> u8 {
        self.inner.ty()
    }
}

impl alloy::network::TransactionResponse for TransactionResponse {
    fn tx_hash(&self) -> alloy::primitives::TxHash {
        self.inner.tx_hash()
    }

    fn from(&self) -> alloy::primitives::Address {
        self.inner.from()
    }

    fn block_hash(&self) -> Option<alloy::primitives::BlockHash> {
        self.inner.block_hash()
    }

    fn block_number(&self) -> Option<u64> {
        self.inner.block_number()
    }

    fn transaction_index(&self) -> Option<u64> {
        self.inner.transaction_index()
    }
}

impl AsRef<crate::network::tx_envelope::TxEnvelope> for TransactionResponse {
    fn as_ref(&self) -> &crate::network::tx_envelope::TxEnvelope {
        &self.inner.inner
    }
}

mod serde_from {
    //! NB: Why do we need this?
    //!
    //! Helper module for serializing and deserializing ZKsync [`TransactionResponse`].
    //!
    //! This is needed because we might need to deserialize the `from` field into both
    //! [`field@alloy::rpc::types::transaction::Transaction::from`] and [`field@TxEip712::from`].
    use crate::network::transaction_response::TransactionResponse;
    use crate::network::tx_envelope::TxEnvelope;
    use crate::network::unsigned_tx::eip712::TxEip712;
    use alloy::consensus::Signed;
    use alloy::primitives::BlockHash;
    use serde::{Deserialize, Serialize};

    /// Exactly the same thing as [`alloy::rpc::types::transaction::Transaction`] but without the
    /// `from` field. We need it because [`TxEnvelope::Eip712`] can consume `from` first thus
    /// failing the entire deserialization process.
    #[derive(Serialize, Deserialize)]
    pub struct TransactionWithoutFrom {
        #[serde(flatten)]
        pub inner: Signed<TxEip712>,
        pub block_hash: Option<BlockHash>,
        pub block_number: Option<u64>,
        pub transaction_index: Option<u64>,
        pub effective_gas_price: Option<u128>,
    }

    /// (De)serializes both regular [`alloy::rpc::types::transaction::Transaction`] and [`TransactionWithoutFrom`].
    #[derive(Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum TransactionEither {
        Regular(alloy::rpc::types::transaction::Transaction<TxEnvelope>),
        WithoutFrom(TransactionWithoutFrom),
    }

    impl From<TransactionEither> for TransactionResponse {
        fn from(value: TransactionEither) -> Self {
            match value {
                TransactionEither::Regular(tx) => TransactionResponse { inner: tx },
                TransactionEither::WithoutFrom(value) => {
                    let from = value.inner.tx().from;
                    TransactionResponse {
                        inner: alloy::rpc::types::transaction::Transaction {
                            inner: TxEnvelope::Eip712(value.inner),
                            block_hash: value.block_hash,
                            block_number: value.block_number,
                            transaction_index: value.transaction_index,
                            effective_gas_price: value.effective_gas_price,
                            from,
                        },
                    }
                }
            }
        }
    }

    impl From<TransactionResponse> for TransactionEither {
        fn from(value: TransactionResponse) -> Self {
            match value.inner.inner {
                TxEnvelope::Native(_) => TransactionEither::Regular(value.inner),
                TxEnvelope::Eip712(signed) => {
                    TransactionEither::WithoutFrom(TransactionWithoutFrom {
                        inner: signed,
                        block_hash: value.inner.block_hash,
                        block_number: value.inner.block_number,
                        transaction_index: value.inner.transaction_index,
                        effective_gas_price: value.inner.effective_gas_price,
                    })
                }
            }
        }
    }
}
