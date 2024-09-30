#[derive(Debug)]
pub enum TypedTransaction {
    Native(alloy_consensus::TypedTransaction),
}

impl From<crate::network::tx_envelope::TxEnvelope> for TypedTransaction {
    fn from(value: crate::network::tx_envelope::TxEnvelope) -> Self {
        match value {
            crate::network::tx_envelope::TxEnvelope::Native(inner) => Self::Native(inner.into()),
        }
    }
}
