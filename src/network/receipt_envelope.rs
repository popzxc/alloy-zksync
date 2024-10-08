use alloy::consensus::TxReceipt;
use alloy::network::eip2718::{Decodable2718, Encodable2718};
use alloy::primitives::Log;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ReceiptEnvelope<T = Log> {
    Native(alloy::consensus::ReceiptEnvelope<T>),
}

impl<T> TxReceipt<T> for ReceiptEnvelope<T> {
    fn status_or_post_state(&self) -> alloy::consensus::Eip658Value {
        match self {
            ReceiptEnvelope::Native(re) => re.status_or_post_state(),
        }
    }

    fn status(&self) -> bool {
        match self {
            ReceiptEnvelope::Native(re) => re.status(),
        }
    }

    fn bloom(&self) -> alloy::primitives::Bloom {
        match self {
            ReceiptEnvelope::Native(re) => re.bloom(),
        }
    }

    fn cumulative_gas_used(&self) -> u128 {
        match self {
            ReceiptEnvelope::Native(re) => re.cumulative_gas_used(),
        }
    }

    fn logs(&self) -> &[T] {
        match self {
            ReceiptEnvelope::Native(re) => re.logs(),
        }
    }
}

impl Encodable2718 for ReceiptEnvelope {
    fn type_flag(&self) -> Option<u8> {
        match self {
            ReceiptEnvelope::Native(re) => re.type_flag(),
        }
    }

    fn encode_2718_len(&self) -> usize {
        match self {
            ReceiptEnvelope::Native(re) => re.encode_2718_len(),
        }
    }

    fn encode_2718(&self, out: &mut dyn alloy::primitives::bytes::BufMut) {
        match self {
            ReceiptEnvelope::Native(re) => re.encode_2718(out),
        }
    }
}

impl Decodable2718 for ReceiptEnvelope {
    fn typed_decode(ty: u8, buf: &mut &[u8]) -> alloy::network::eip2718::Eip2718Result<Self> {
        alloy::consensus::ReceiptEnvelope::typed_decode(ty, buf).map(ReceiptEnvelope::Native)
    }

    fn fallback_decode(buf: &mut &[u8]) -> alloy::network::eip2718::Eip2718Result<Self> {
        alloy::consensus::ReceiptEnvelope::fallback_decode(buf).map(ReceiptEnvelope::Native)
    }
}
