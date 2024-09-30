use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderResponse {
    #[serde(flatten)]
    inner: alloy_rpc_types_eth::Header,
}
