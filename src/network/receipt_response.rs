use serde::{Deserialize, Serialize};

use alloy::consensus::{AnyReceiptEnvelope, TxReceipt};
use alloy::primitives::{Address, B256, U64};
use alloy::rpc::types::{Log, TransactionReceipt};

use crate::types::*;
//use super::receipt_envelope::ReceiptEnvelope;
use alloy::eips::eip7702::SignedAuthorization;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptResponse<T = AnyReceiptEnvelope<Log>> {
    #[serde(flatten)]
    inner: TransactionReceipt<T>,

    l1_batch_number: Option<U64>,
    l1_batch_tx_index: Option<U64>,
    l2_to_l1_logs: Vec<L2ToL1Log>,
}

impl ReceiptResponse {
    pub fn logs(&self) -> &[Log] {
        self.inner.inner.logs()
    }
    pub fn logs_bloom(&self) -> alloy::primitives::Bloom {
        self.inner.inner.bloom()
    }
    pub fn l1_batch_number(&self) -> Option<U64> {
        self.l1_batch_number
    }
    pub fn l1_batch_tx_index(&self) -> Option<U64> {
        self.l1_batch_tx_index
    }
    pub fn l2_to_l1_logs(&self) -> &[L2ToL1Log] {
        &self.l2_to_l1_logs
    }
}

impl<T: TxReceipt<Log>> alloy::network::ReceiptResponse for ReceiptResponse<T> {
    /// Address of the created contract, or `None` if the transaction was not a deployment.
    fn contract_address(&self) -> Option<Address> {
        self.inner.contract_address()
    }

    /// Status of the transaction.
    fn status(&self) -> bool {
        self.inner.status()
    }

    /// Hash of the block this transaction was included within.
    fn block_hash(&self) -> Option<alloy::primitives::BlockHash> {
        self.inner.block_hash()
    }

    /// Number of the block this transaction was included within.
    fn block_number(&self) -> Option<u64> {
        self.inner.block_number()
    }

    /// Transaction Hash.
    fn transaction_hash(&self) -> alloy::primitives::TxHash {
        self.inner.transaction_hash()
    }

    /// Index within the block.
    fn transaction_index(&self) -> Option<u64> {
        self.inner.transaction_index()
    }

    /// Gas used by this transaction alone.
    fn gas_used(&self) -> u128 {
        self.inner.gas_used()
    }

    /// Effective gas price.
    fn effective_gas_price(&self) -> u128 {
        self.inner.effective_gas_price()
    }

    /// Blob gas used by the eip-4844 transaction.
    fn blob_gas_used(&self) -> Option<u128> {
        self.inner.blob_gas_used()
    }

    /// Blob gas price paid by the eip-4844 transaction.
    fn blob_gas_price(&self) -> Option<u128> {
        self.inner.blob_gas_price()
    }

    /// Address of the sender.
    fn from(&self) -> Address {
        self.inner.from()
    }

    /// Address of the receiver.
    fn to(&self) -> Option<Address> {
        self.inner.to()
    }

    /// EIP-7702 Authorization list.
    fn authorization_list(&self) -> Option<&[SignedAuthorization]> {
        self.inner.authorization_list()
    }

    /// Returns the cumulative gas used at this receipt.
    fn cumulative_gas_used(&self) -> u128 {
        self.inner.cumulative_gas_used()
    }

    /// The post-transaction state root (pre Byzantium)
    ///
    /// EIP98 makes this field optional.
    fn state_root(&self) -> Option<B256> {
        self.inner.state_root()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use alloy::network::ReceiptResponse as AlloyReceiptResponse;
    use alloy::primitives::address;
    use alloy::primitives::U256;

    #[tokio::test(flavor = "multi_thread")]
    async fn receipt_test() {
        let receipt_json = r#"
        {
            "blockHash": "0x5046bdc714b2a9b40e9fbfdfc5140371c1b03b40335d908de92a7686dcc067e9",
            "blockNumber": "0x1d1551e",
            "contractAddress": "0x0000000000000000000000000000000000008006",
            "cumulativeGasUsed": "0x0",
            "effectiveGasPrice": "0x17d7840",
            "from": "0x1bc3366b3664c01b8687b1efcfc6478d9351a8a9",
            "gasUsed": "0x2b9bcb",
            "l1BatchNumber": "0x72ae1",
            "l1BatchTxIndex": "0x469",
            "l2ToL1Logs": [
                {
                    "blockHash": "0x5046bdc714b2a9b40e9fbfdfc5140371c1b03b40335d908de92a7686dcc067e9",
                    "blockNumber": "30496030",
                    "isService": true,
                    "key": "0x000000000000000000000000000000000000000000000000000000000000800a",
                    "l1BatchNumber": "0x72ae1",
                    "logIndex": "0x0",
                    "sender": "0x0000000000000000000000000000000000008008",
                    "shardId": "0x0",
                    "transactionHash": "0xb2adc4d2b3203e186001dc37fdf02cc8e772518425d263adc6a17dbddff3bfda",
                    "transactionIndex": "0x0",
                    "transactionLogIndex": "0x0",
                    "txIndexInL1Batch": "0x12d",
                    "value": "0x30c635c6a0084404145f3723046c1c1b21eb5ccbb97893c90747c7a8bd83a641"
                }
            ],
            "logs": [
                {
                    "address": "0x000000000000000000000000000000000000800a",
                    "blockHash": "0x5046bdc714b2a9b40e9fbfdfc5140371c1b03b40335d908de92a7686dcc067e9",
                    "blockNumber": "0x1d1551e",
                    "blockTimestamp": "0x660c1740",
                    "data": "0x0000000000000000000000000000000000000000000000000001011c8f80b6c0",
                    "l1BatchNumber": "0x72ae1",
                    "logIndex": "0x0",
                    "logType": null,
                    "removed": false,
                    "topics": [
                        "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef",
                        "0x0000000000000000000000001bc3366b3664c01b8687b1efcfc6478d9351a8a9",
                        "0x0000000000000000000000000000000000000000000000000000000000008001"
                    ],
                    "transactionHash": "0xb2adc4d2b3203e186001dc37fdf02cc8e772518425d263adc6a17dbddff3bfda",
                    "transactionIndex": "0x0",
                    "transactionLogIndex": "0x0"
                },
                {
                    "address": "0x000000000000000000000000000000000000800a",
                    "blockHash": "0x5046bdc714b2a9b40e9fbfdfc5140371c1b03b40335d908de92a7686dcc067e9",
                    "blockNumber": "0x1d1551e",
                    "blockTimestamp": "0x660c1740",
                    "data": "0x000000000000000000000000000000000000000000000000000042a896fb71c0",
                    "l1BatchNumber": "0x72ae1",
                    "logIndex": "0x1",
                    "logType": null,
                    "removed": false,
                    "topics": [
                        "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef",
                        "0x0000000000000000000000000000000000000000000000000000000000008001",
                        "0x0000000000000000000000001bc3366b3664c01b8687b1efcfc6478d9351a8a9"
                    ],
                    "transactionHash": "0xb2adc4d2b3203e186001dc37fdf02cc8e772518425d263adc6a17dbddff3bfda",
                    "transactionIndex": "0x0",
                    "transactionLogIndex": "0x1"
                }
            ],
            "logsBloom": "0x00280000000400000000000088000000000000000000008000004000800000100000000000000000002000000010000002000000000040000008000000000080000000002000840000000008000000240000000000000000000080010000080000000000020500000004000000000800000000000000000001000010000000000000000000000000000004000000000000001000000000000000004000000000000000000001100004000000000010000000000000000000000000000000000000000002008002800000080400500110402000000000000000000000000020000000000000000000000000000000000000002041000000020000000000000060",
            "status": "0x1",
            "to": "0x9b5def958d0f3b6955cbea4d5b7809b2fb26b059",
            "transactionHash": "0xb2adc4d2b3203e186001dc37fdf02cc8e772518425d263adc6a17dbddff3bfda",
            "transactionIndex": "0x0",
            "type": "0x2",
            "blobGasUsed": 111111,
            "blobGasPrice": 222222
        }
        "#;

        let receipt = serde_json::from_str::<ReceiptResponse>(receipt_json).unwrap();
        assert_eq!(receipt.l1_batch_number(), Some(U64::from(0x72ae1)));
        assert_eq!(receipt.l1_batch_tx_index(), Some(U64::from(0x469)));
        assert_eq!(
            receipt.l2_to_l1_logs(),
            vec![L2ToL1Log {
                block_hash: Some(
                    B256::from_str(
                        "0x5046bdc714b2a9b40e9fbfdfc5140371c1b03b40335d908de92a7686dcc067e9"
                    )
                    .unwrap()
                ),
                block_number: U64::from(30496030),
                l1_batch_number: Some(U64::from(0x72ae1)),
                log_index: U256::from(0),
                transaction_index: U64::from(0),
                transaction_hash: B256::from_str(
                    "0xb2adc4d2b3203e186001dc37fdf02cc8e772518425d263adc6a17dbddff3bfda"
                )
                .unwrap(),
                transaction_log_index: U256::from(0),
                tx_index_in_l1_batch: Some(U64::from(301)),
                shard_id: U64::from(0),
                is_service: true,
                sender: address!("0000000000000000000000000000000000008008"),
                key: B256::from_str(
                    "0x000000000000000000000000000000000000000000000000000000000000800a"
                )
                .unwrap(),
                value: B256::from_str(
                    "0x30c635c6a0084404145f3723046c1c1b21eb5ccbb97893c90747c7a8bd83a641"
                )
                .unwrap()
            }]
        );
        assert_eq!(receipt.logs(), receipt.inner.inner.logs());
        assert_eq!(receipt.logs_bloom(), receipt.inner.inner.bloom());
        assert_eq!(
            receipt.contract_address(),
            Some(address!("0000000000000000000000000000000000008006"))
        );
        assert!(receipt.status());
        assert_eq!(
            receipt.block_hash(),
            Some(
                B256::from_str(
                    "0x5046bdc714b2a9b40e9fbfdfc5140371c1b03b40335d908de92a7686dcc067e9"
                )
                .unwrap()
            )
        );
        assert_eq!(receipt.block_number(), Some(30496030));
        assert_eq!(
            receipt.transaction_hash(),
            B256::from_str("0xb2adc4d2b3203e186001dc37fdf02cc8e772518425d263adc6a17dbddff3bfda")
                .unwrap()
        );
        assert_eq!(receipt.transaction_index(), Some(0));
        assert_eq!(receipt.gas_used(), 2857931);
        assert_eq!(receipt.effective_gas_price(), 25000000);
        assert_eq!(receipt.blob_gas_used(), Some(111111));
        assert_eq!(receipt.blob_gas_price(), Some(222222));
        assert_eq!(
            receipt.from(),
            address!("1bc3366b3664c01b8687b1efcfc6478d9351a8a9")
        );
        assert_eq!(
            receipt.to(),
            Some(address!("9b5def958d0f3b6955cbea4d5b7809b2fb26b059"))
        );
        assert_eq!(receipt.authorization_list(), None);
        assert_eq!(receipt.cumulative_gas_used(), 0);
        assert_eq!(receipt.state_root(), None);
    }
}
