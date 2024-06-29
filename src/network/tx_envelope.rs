use alloy_network::eip2718::{Decodable2718, Encodable2718};

#[derive(Debug)]
pub struct TxEnvelope {
    pub(crate) inner: alloy_consensus::TxEnvelope,
}

impl Encodable2718 for TxEnvelope {
    fn type_flag(&self) -> Option<u8> {
        Encodable2718::type_flag(&self.inner)
    }

    fn encode_2718_len(&self) -> usize {
        Encodable2718::encode_2718_len(&self.inner)
    }

    fn encode_2718(&self, out: &mut dyn alloy_primitives::bytes::BufMut) {
        Encodable2718::encode_2718(&self.inner, out)
    }
}

impl Decodable2718 for TxEnvelope {
    fn typed_decode(ty: u8, buf: &mut &[u8]) -> alloy_network::eip2718::Eip2718Result<Self> {
        let inner = alloy_consensus::TxEnvelope::typed_decode(ty, buf)?;
        Ok(Self { inner })
    }

    fn fallback_decode(buf: &mut &[u8]) -> alloy_network::eip2718::Eip2718Result<Self> {
        let inner = alloy_consensus::TxEnvelope::fallback_decode(buf)?;
        Ok(Self { inner })
    }
}
