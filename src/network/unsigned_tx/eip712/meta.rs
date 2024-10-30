use alloy::primitives::{Address, Bytes, FixedBytes, U256};
use alloy::rlp::{Decodable, Encodable, Header};
use serde::{ser::SerializeSeq, Deserialize, Serialize};

use super::utils::{hash_bytecode, BytecodeHashError};

// Serialize `Bytes` as `Vec<u8>` as they are encoded as hex string for human-friendly serializers
fn serialize_bytes<S: serde::Serializer>(
    bytes: &Vec<Bytes>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let mut seq = serializer.serialize_seq(Some(bytes.len()))?;
    for e in bytes {
        seq.serialize_element(&e.0)?;
    }
    seq.end()
}

// Seralize 'Bytes' to encode them into RLP friendly format.
fn serialize_bytes_custom<S: serde::Serializer>(
    bytes: &Bytes,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_bytes(&bytes.0)
}

// TODO: The structure should be correct by construction, e.g. we should not allow
// creating or deserializing meta that has invalid factory deps.
// TODO: Serde here is used for `TransactionRequest` needs, this has to be reworked once
// `TransactionRequest` uses custom `Eip712Meta` structure.
#[derive(Default, Serialize, Deserialize, Clone, PartialEq, Debug, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct Eip712Meta {
    pub gas_per_pubdata: U256,
    #[serde(default)]
    #[serde(serialize_with = "serialize_bytes")]
    pub factory_deps: Vec<Bytes>,
    pub custom_signature: Option<Bytes>,
    pub paymaster_params: Option<PaymasterParams>,
}

impl Eip712Meta {
    pub fn factory_deps_hashes(&self) -> Result<Vec<FixedBytes<32>>, BytecodeHashError> {
        let mut hashes = Vec::with_capacity(self.factory_deps.len() * 32);
        for dep in &self.factory_deps {
            let hash = hash_bytecode(dep)?;
            hashes.push(hash.into());
        }
        Ok(hashes)
    }
}

impl Decodable for Eip712Meta {
    fn decode(buf: &mut &[u8]) -> alloy::rlp::Result<Self> {
        fn opt_decode<T: Decodable>(buf: &mut &[u8]) -> alloy::rlp::Result<Option<T>> {
            Ok(Decodable::decode(buf).ok()) // TODO: better validation of error?
        }

        let gas_per_pubdata = Decodable::decode(buf)?;
        let factory_deps = Decodable::decode(buf)?;
        let custom_signature = opt_decode(buf)?;
        let paymaster_params = opt_decode(buf)?;

        Ok(Self {
            gas_per_pubdata,
            factory_deps,
            custom_signature,
            paymaster_params,
        })
    }
}

impl Encodable for Eip712Meta {
    fn encode(&self, out: &mut dyn alloy::rlp::BufMut) {
        fn opt_encode<T>(stream: &mut dyn alloy::rlp::BufMut, value: Option<T>)
        where
            T: Encodable,
        {
            if let Some(v) = value {
                v.encode(stream);
            } else {
                "".encode(stream);
            }
        }
        self.gas_per_pubdata.encode(out);
        self.factory_deps.encode(out);
        opt_encode(out, self.custom_signature.clone());
        opt_encode(out, self.paymaster_params.clone());
    }

    // TODO: Implement `length` method
}

#[derive(Default, Serialize, Deserialize, Clone, PartialEq, Debug, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct PaymasterParams {
    pub paymaster: Address,
    /// A custom serialization is needed (otherwise RLP treats it as string).
    #[serde(serialize_with = "serialize_bytes_custom")]
    pub paymaster_input: Bytes,
}

impl Decodable for PaymasterParams {
    fn decode(buf: &mut &[u8]) -> alloy::rlp::Result<Self> {
        let mut bytes = Header::decode_bytes(buf, true)?;
        let payload_view = &mut bytes;
        Ok(Self {
            paymaster: dbg!(Decodable::decode(payload_view))?,
            paymaster_input: dbg!(Decodable::decode(payload_view))?,
        })
    }
}

impl Encodable for PaymasterParams {
    fn encode(&self, out: &mut dyn alloy::rlp::BufMut) {
        // paymaster params have to be encoded as a list.
        let h = Header {
            list: true,
            payload_length: self.paymaster.length() + self.paymaster_input.length(),
        };
        h.encode(out);
        self.paymaster.encode(out);
        self.paymaster_input.encode(out);
    }
}
