use alloy::consensus::Signed;
use alloy::network::eip2718::{Decodable2718, Encodable2718};
use alloy::rlp::Header;

use super::unsigned_tx::eip712::TxEip712;

#[derive(Debug)]
pub enum TxEnvelope {
    Native(alloy::consensus::TxEnvelope),
    Eip712(Signed<TxEip712>),
}

impl Encodable2718 for TxEnvelope {
    fn type_flag(&self) -> Option<u8> {
        match self {
            Self::Native(inner) => inner.type_flag(),
            Self::Eip712(inner) => Some(inner.tx().tx_type() as u8),
        }
    }

    fn encode_2718_len(&self) -> usize {
        match self {
            Self::Native(inner) => inner.encode_2718_len(),
            Self::Eip712(inner) => {
                let payload_length = inner.tx().fields_len() + inner.signature().rlp_vrs_len();
                Header {
                    list: true,
                    payload_length,
                }
                .length()
                    + payload_length
            }
        }
    }

    fn encode_2718(&self, out: &mut dyn alloy::primitives::bytes::BufMut) {
        match self {
            Self::Native(inner) => inner.encode_2718(out),
            Self::Eip712(tx) => {
                tx.tx().encode_with_signature(tx.signature(), out);
            }
        }
    }
}

impl Decodable2718 for TxEnvelope {
    fn typed_decode(ty: u8, buf: &mut &[u8]) -> alloy::network::eip2718::Eip2718Result<Self> {
        let inner = alloy::consensus::TxEnvelope::typed_decode(ty, buf)?;
        Ok(Self::Native(inner))
    }

    fn fallback_decode(buf: &mut &[u8]) -> alloy::network::eip2718::Eip2718Result<Self> {
        let inner = alloy::consensus::TxEnvelope::fallback_decode(buf)?;
        Ok(Self::Native(inner))
    }
}
