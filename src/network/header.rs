use serde::{Deserialize, Serialize};

/// See [Header](https://docs.rs/alloy/latest/alloy/rpc/types/struct.Header.html).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Header {
    #[serde(flatten)]
    inner: alloy::consensus::Header,
}

impl Header {
    pub fn hash_slow(&self) -> alloy::primitives::B256 {
        self.inner.hash_slow()
    }
}

impl alloy::consensus::BlockHeader for Header {
    fn parent_hash(&self) -> alloy::primitives::B256 {
        self.inner.parent_hash()
    }

    fn ommers_hash(&self) -> alloy::primitives::B256 {
        self.inner.ommers_hash()
    }

    fn beneficiary(&self) -> alloy::primitives::Address {
        self.inner.beneficiary()
    }

    fn state_root(&self) -> alloy::primitives::B256 {
        self.inner.state_root()
    }

    fn transactions_root(&self) -> alloy::primitives::B256 {
        self.inner.transactions_root()
    }

    fn receipts_root(&self) -> alloy::primitives::B256 {
        self.inner.receipts_root()
    }

    fn withdrawals_root(&self) -> Option<alloy::primitives::B256> {
        self.inner.withdrawals_root()
    }

    fn logs_bloom(&self) -> alloy::primitives::Bloom {
        self.inner.logs_bloom()
    }

    fn difficulty(&self) -> alloy::primitives::U256 {
        self.inner.difficulty()
    }

    fn number(&self) -> alloy::primitives::BlockNumber {
        self.inner.number()
    }

    fn gas_limit(&self) -> u64 {
        self.inner.gas_limit()
    }

    fn gas_used(&self) -> u64 {
        self.inner.gas_used()
    }

    fn timestamp(&self) -> u64 {
        self.inner.timestamp()
    }

    fn mix_hash(&self) -> Option<alloy::primitives::B256> {
        self.inner.mix_hash()
    }

    fn nonce(&self) -> Option<alloy::primitives::B64> {
        self.inner.nonce()
    }

    fn base_fee_per_gas(&self) -> Option<u64> {
        self.inner.base_fee_per_gas()
    }

    fn blob_gas_used(&self) -> Option<u64> {
        self.inner.blob_gas_used()
    }

    fn excess_blob_gas(&self) -> Option<u64> {
        self.inner.excess_blob_gas()
    }

    fn parent_beacon_block_root(&self) -> Option<alloy::primitives::B256> {
        self.inner.parent_beacon_block_root()
    }

    fn requests_hash(&self) -> Option<alloy::primitives::B256> {
        self.inner.requests_hash()
    }

    fn target_blobs_per_block(&self) -> Option<u64> {
        self.inner.target_blobs_per_block()
    }

    fn extra_data(&self) -> &alloy::primitives::Bytes {
        self.inner.extra_data()
    }
}
