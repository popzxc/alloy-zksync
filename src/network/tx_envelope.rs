use alloy_network::eip2718::{Decodable2718, Encodable2718};

#[derive(Debug)]
pub enum TxEnvelope {
    Native(alloy_consensus::TxEnvelope),
}

/// Macro that delegates a method call to the inner variant implementation.
macro_rules! delegate {
    ($_self:ident.$method:ident($($args:expr),*)) => {
        match $_self {
            Self::Native(inner) => inner.$method($($args),*),
        }
    };
}

impl Encodable2718 for TxEnvelope {
    fn type_flag(&self) -> Option<u8> {
        delegate!(self.type_flag())
    }

    fn encode_2718_len(&self) -> usize {
        delegate!(self.encode_2718_len())
    }

    fn encode_2718(&self, out: &mut dyn alloy_primitives::bytes::BufMut) {
        delegate!(self.encode_2718(out))
    }
}

impl Decodable2718 for TxEnvelope {
    fn typed_decode(ty: u8, buf: &mut &[u8]) -> alloy_network::eip2718::Eip2718Result<Self> {
        let inner = alloy_consensus::TxEnvelope::typed_decode(ty, buf)?;
        Ok(Self::Native(inner))
    }

    fn fallback_decode(buf: &mut &[u8]) -> alloy_network::eip2718::Eip2718Result<Self> {
        let inner = alloy_consensus::TxEnvelope::fallback_decode(buf)?;
        Ok(Self::Native(inner))
    }
}
