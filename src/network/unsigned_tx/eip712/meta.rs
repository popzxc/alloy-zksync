use alloy_primitives::{keccak256, Address, Bytes, FixedBytes, U256};
use alloy_rlp::{Decodable, Header};
use serde::{Deserialize, Serialize};

use super::utils::{hash_bytecode, BytecodeHashError};

// TODO: The structure should be correct by construction, e.g. we should not allow
// creating or deserializing meta that has invalid factory deps.
#[derive(Default, Serialize, Deserialize, Clone, PartialEq, Debug, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct Eip712Meta {
    pub gas_per_pubdata: U256,
    #[serde(default)]
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
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        fn opt_decode<T: Decodable>(buf: &mut &[u8]) -> alloy_rlp::Result<Option<T>> {
            if buf.is_empty() {
                return Ok(None);
            }
            Ok(Some(Decodable::decode(buf)?))
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

#[derive(Default, Serialize, Deserialize, Clone, PartialEq, Debug, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct PaymasterParams {
    pub paymaster: Address,
    pub paymaster_input: Bytes,
}

impl Decodable for PaymasterParams {
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        let mut bytes = Header::decode_bytes(buf, true)?;
        let payload_view = &mut bytes;
        Ok(Self {
            paymaster: dbg!(Decodable::decode(payload_view))?,
            paymaster_input: dbg!(Decodable::decode(payload_view))?,
        })
    }
}
