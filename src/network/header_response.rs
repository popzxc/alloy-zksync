use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderResponse {
    #[serde(flatten)]
    inner: alloy::rpc::types::Header,
}

impl alloy::network::HeaderResponse for HeaderResponse {
    fn hash(&self) -> alloy::primitives::BlockHash {
        self.inner.hash()
    }

    fn number(&self) -> u64 {
        self.inner.number()
    }

    fn timestamp(&self) -> u64 {
        self.inner.timestamp()
    }

    fn extra_data(&self) -> &alloy::primitives::Bytes {
        self.inner.extra_data()
    }

    fn base_fee_per_gas(&self) -> Option<u64> {
        self.inner.base_fee_per_gas()
    }

    fn next_block_blob_fee(&self) -> Option<u128> {
        self.inner.next_block_blob_fee()
    }

    fn coinbase(&self) -> alloy::primitives::Address {
        self.inner.coinbase()
    }

    fn gas_limit(&self) -> u64 {
        self.inner.gas_limit()
    }

    fn mix_hash(&self) -> Option<alloy::primitives::B256> {
        self.inner.mix_hash()
    }

    fn difficulty(&self) -> alloy::primitives::U256 {
        self.inner.difficulty()
    }
}
