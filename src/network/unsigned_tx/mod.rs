use eip712::TxEip712;

pub mod eip712;

#[derive(Debug)]
pub enum TypedTransaction {
    Native(alloy::consensus::TypedTransaction),
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
