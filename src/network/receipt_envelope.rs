use alloy_consensus::TxReceipt;
use alloy_network::eip2718::{Decodable2718, Encodable2718};
use alloy_primitives::Log;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ReceiptEnvelope<T = Log> {
    Native(alloy_consensus::ReceiptEnvelope<T>),
}

impl<T> TxReceipt<T> for ReceiptEnvelope<T> {
    fn status_or_post_state(&self) -> &alloy_consensus::Eip658Value {
        match self {
            ReceiptEnvelope::Native(re) => re.status_or_post_state(),
        }
    }

    fn status(&self) -> bool {
        match self {
            ReceiptEnvelope::Native(re) => re.status(),
        }
    }

    fn bloom(&self) -> alloy_primitives::Bloom {
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

    fn encode_2718(&self, out: &mut dyn alloy_primitives::bytes::BufMut) {
        match self {
            ReceiptEnvelope::Native(re) => re.encode_2718(out),
        }
    }
}

impl Decodable2718 for ReceiptEnvelope {
    fn typed_decode(ty: u8, buf: &mut &[u8]) -> alloy_network::eip2718::Eip2718Result<Self> {
        alloy_consensus::ReceiptEnvelope::typed_decode(ty, buf).map(ReceiptEnvelope::Native)
    }

    fn fallback_decode(buf: &mut &[u8]) -> alloy_network::eip2718::Eip2718Result<Self> {
        alloy_consensus::ReceiptEnvelope::fallback_decode(buf).map(ReceiptEnvelope::Native)
    }
}
