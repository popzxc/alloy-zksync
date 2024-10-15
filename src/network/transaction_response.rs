use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResponse {
    #[serde(flatten)]
    inner: alloy::rpc::types::Transaction,
}

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

    fn to(&self) -> alloy::primitives::TxKind {
        self.inner.to()
    }

    fn value(&self) -> alloy::primitives::U256 {
        self.inner.value()
    }

    fn input(&self) -> &[u8] {
        self.inner.input()
    }

    fn ty(&self) -> u8 {
        self.inner.ty()
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
}

impl alloy::network::TransactionResponse for TransactionResponse {
    fn tx_hash(&self) -> alloy::primitives::TxHash {
        self.inner.tx_hash()
    }

    fn from(&self) -> alloy::primitives::Address {
        self.inner.from()
    }

    fn to(&self) -> Option<alloy::primitives::Address> {
        self.inner.to()
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
