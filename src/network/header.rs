use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Header {
    #[serde(flatten)]
    inner: alloy_consensus::Header,
}
