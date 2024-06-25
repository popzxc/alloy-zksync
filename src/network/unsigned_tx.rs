#[derive(Debug)]
pub struct TypedTransaction {
    pub(crate) inner: alloy_consensus::TypedTransaction,
}

impl From<crate::network::tx_envelope::TxEnvelope> for TypedTransaction {
    fn from(value: crate::network::tx_envelope::TxEnvelope) -> Self {
        Self {
            inner: From::from(value.inner),
        }
    }
}
