use eip712::TxEip712;

pub mod eip712;

/// ZKsync transaction type.
#[derive(Debug)]
pub enum TypedTransaction {
    /// Ethereum-native transaction type, e.g. legacy or EIP-1559.
    Native(alloy::consensus::TypedTransaction),
    /// ZKsync-specific EIP-712 transaction type.
    Eip712(TxEip712),
}

impl From<crate::network::tx_envelope::TxEnvelope> for TypedTransaction {
    fn from(value: crate::network::tx_envelope::TxEnvelope) -> Self {
        match value {
            crate::network::tx_envelope::TxEnvelope::Native(inner) => Self::Native(inner.into()),
            super::tx_envelope::TxEnvelope::Eip712(signed) => Self::Eip712(signed.into_parts().0),
        }
    }
}
