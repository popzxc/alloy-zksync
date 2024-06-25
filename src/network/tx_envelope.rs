use alloy_network::eip2718::{Decodable2718, Encodable2718};

#[derive(Debug)]
pub struct TxEnvelope {
    pub(crate) inner: alloy_consensus::TxEnvelope,
}

impl Encodable2718 for TxEnvelope {
    fn type_flag(&self) -> Option<u8> {
        todo!()
    }

    fn encode_2718_len(&self) -> usize {
        todo!()
    }

    fn encode_2718(&self, out: &mut dyn alloy_primitives::bytes::BufMut) {
        todo!()
    }
}

impl Decodable2718 for TxEnvelope {
    fn typed_decode(ty: u8, buf: &mut &[u8]) -> alloy_network::eip2718::Eip2718Result<Self> {
        todo!()
    }

    fn fallback_decode(buf: &mut &[u8]) -> alloy_network::eip2718::Eip2718Result<Self> {
        todo!()
    }
}
