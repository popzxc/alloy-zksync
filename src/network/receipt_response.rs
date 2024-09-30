use alloy_rpc_types_eth::AnyTransactionReceipt;

pub type ReceiptResponse = AnyTransactionReceipt;

// TODO: aka TransactionReceipt
// TODO: can we get away with `AnyTransactionReceipt`?
// #[derive(Debug, Serialize, Deserialize)]
// pub struct ReceiptResponse<T = ReceiptEnvelope<Log>> {
//     #[serde(flatten)]
//     inner: alloy_rpc_types_eth::TransactionReceipt<T>,
// }

// impl<T> alloy_network::ReceiptResponse for ReceiptResponse<T> {
//     fn contract_address(&self) -> Option<alloy_primitives::Address> {
//         alloy_network::ReceiptResponse::contract_address(&self.inner)
//     }

//     fn status(&self) -> bool {
//         alloy_network::ReceiptResponse::status(&self.inner)
//     }

//     fn block_hash(&self) -> Option<alloy_primitives::BlockHash> {
//         alloy_network::ReceiptResponse::block_hash(&self.inner)
//     }

//     fn block_number(&self) -> Option<u64> {
//         alloy_network::ReceiptResponse::block_number(&self.inner)
//     }
// }
