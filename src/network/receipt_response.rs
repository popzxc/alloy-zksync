use alloy::rpc::types::AnyTransactionReceipt;

pub type ReceiptResponse = AnyTransactionReceipt;

// TODO: aka TransactionReceipt
// TODO: can we get away with `AnyTransactionReceipt`?
// #[derive(Debug, Serialize, Deserialize)]
// pub struct ReceiptResponse<T = ReceiptEnvelope<Log>> {
//     #[serde(flatten)]
//     inner: alloy::rpc_types_eth::TransactionReceipt<T>,
// }

// impl<T> alloy::network::ReceiptResponse for ReceiptResponse<T> {
//     fn contract_address(&self) -> Option<alloy::primitives::Address> {
//         alloy::network::ReceiptResponse::contract_address(&self.inner)
//     }

//     fn status(&self) -> bool {
//         alloy::network::ReceiptResponse::status(&self.inner)
//     }

//     fn block_hash(&self) -> Option<alloy::primitives::BlockHash> {
//         alloy::network::ReceiptResponse::block_hash(&self.inner)
//     }

//     fn block_number(&self) -> Option<u64> {
//         alloy::network::ReceiptResponse::block_number(&self.inner)
//     }
// }
