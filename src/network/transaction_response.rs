use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResponse {
    #[serde(flatten)]
    inner: alloy_rpc_types_eth::Transaction,
}

impl alloy_network::TransactionResponse for TransactionResponse {
    fn tx_hash(&self) -> alloy_primitives::TxHash {
        self.inner.tx_hash()
    }

    fn from(&self) -> alloy_primitives::Address {
        self.inner.from()
    }

    fn to(&self) -> Option<alloy_primitives::Address> {
        self.inner.to()
    }

    fn value(&self) -> alloy_primitives::U256 {
        self.inner.value()
    }

    fn gas(&self) -> u128 {
        self.inner.gas()
    }

    fn input(&self) -> &alloy_primitives::Bytes {
        self.inner.input()
    }
}
