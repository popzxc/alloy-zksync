use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HeaderResponse {
    hash: alloy::primitives::BlockHash,
    #[serde(flatten)]
    inner: crate::network::header::Header,
}

impl alloy::consensus::BlockHeader for HeaderResponse {
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

    fn gas_limit(&self) -> u64 {
        self.inner.gas_limit()
    }

    fn mix_hash(&self) -> Option<alloy::primitives::B256> {
        self.inner.mix_hash()
    }

    fn difficulty(&self) -> alloy::primitives::U256 {
        self.inner.difficulty()
    }

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

    fn gas_used(&self) -> u64 {
        self.inner.gas_used()
    }

    fn nonce(&self) -> Option<alloy::primitives::B64> {
        self.inner.nonce()
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
}

impl alloy::network::primitives::HeaderResponse for HeaderResponse {
    fn hash(&self) -> alloy::primitives::BlockHash {
        self.hash
    }
}

impl AsRef<crate::network::header::Header> for HeaderResponse {
    fn as_ref(&self) -> &crate::network::header::Header {
        &self.inner
    }
}
