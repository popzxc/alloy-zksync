use core::fmt;

use alloy::consensus::{TxReceipt, TxType};
use alloy::network::eip2718::{Decodable2718, Eip2718Error, Encodable2718};
use alloy::network::AnyReceiptEnvelope;
use alloy::primitives::Log;
use serde::{Deserialize, Serialize};

/// Receipt envelope is a wrapper around the receipt data.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ReceiptEnvelope<T = Log> {
    /// Receipt for an Ethereum-native transaction.
    Native(alloy::consensus::ReceiptEnvelope<T>),
    /// Receipt for ZKsync-native EIP712 transaction.
    ///
    /// For now AnyReceiptEnvelope is used due to the fact that
    /// alloy::consensus::ReceiptEnvelope cannot be decoded because of different transaction type,
    /// but for now we don't need any custom functionality for EIP712 receipt
    Eip712(AnyReceiptEnvelope<T>),
}

impl<T> TxReceipt for ReceiptEnvelope<T>
where
    T: Clone + fmt::Debug + PartialEq + Eq + Send + Sync,
{
    type Log = T;

    fn status_or_post_state(&self) -> alloy::consensus::Eip658Value {
        match self {
            ReceiptEnvelope::Native(re) => re.status_or_post_state(),
            ReceiptEnvelope::Eip712(re) => re.status_or_post_state(),
        }
    }

    fn status(&self) -> bool {
        match self {
            ReceiptEnvelope::Native(re) => re.status(),
            ReceiptEnvelope::Eip712(re) => re.status(),
        }
    }

    fn bloom(&self) -> alloy::primitives::Bloom {
        match self {
            ReceiptEnvelope::Native(re) => re.bloom(),
            ReceiptEnvelope::Eip712(re) => re.bloom(),
        }
    }

    fn cumulative_gas_used(&self) -> u64 {
        match self {
            ReceiptEnvelope::Native(re) => re.cumulative_gas_used(),
            ReceiptEnvelope::Eip712(re) => re.cumulative_gas_used(),
        }
    }

    fn logs(&self) -> &[T] {
        match self {
            ReceiptEnvelope::Native(re) => re.logs(),
            ReceiptEnvelope::Eip712(re) => re.logs(),
        }
    }
}

impl Encodable2718 for ReceiptEnvelope {
    fn type_flag(&self) -> Option<u8> {
        match self {
            ReceiptEnvelope::Native(re) => re.type_flag(),
            ReceiptEnvelope::Eip712(re) => re.type_flag(),
        }
    }

    fn encode_2718_len(&self) -> usize {
        match self {
            ReceiptEnvelope::Native(re) => re.encode_2718_len(),
            ReceiptEnvelope::Eip712(re) => re.encode_2718_len(),
        }
    }

    fn encode_2718(&self, out: &mut dyn alloy::primitives::bytes::BufMut) {
        match self {
            ReceiptEnvelope::Native(re) => re.encode_2718(out),
            ReceiptEnvelope::Eip712(re) => re.encode_2718(out),
        }
    }
}

impl Decodable2718 for ReceiptEnvelope {
    fn typed_decode(ty: u8, buf: &mut &[u8]) -> alloy::network::eip2718::Eip2718Result<Self> {
        let tx_type_result: Result<TxType, Eip2718Error> = ty.try_into();
        match tx_type_result {
            Ok(_) => alloy::consensus::ReceiptEnvelope::typed_decode(ty, buf)
                .map(ReceiptEnvelope::Native),
            Err(_) => AnyReceiptEnvelope::typed_decode(ty, buf).map(ReceiptEnvelope::Eip712),
        }
    }

    fn fallback_decode(buf: &mut &[u8]) -> alloy::network::eip2718::Eip2718Result<Self> {
        // there is no type available to assume what kind of envelope it is
        // try to decode as a native envelope then as Eip712
        match alloy::consensus::ReceiptEnvelope::fallback_decode(buf) {
            Ok(envelope) => Ok(ReceiptEnvelope::Native(envelope)),
            Err(_) => Ok(ReceiptEnvelope::Eip712(
                AnyReceiptEnvelope::fallback_decode(buf).unwrap(),
            )),
        }
    }
}
