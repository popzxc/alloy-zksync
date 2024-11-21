use crate::network::unsigned_tx::eip712::PaymasterParams;
use alloy::primitives::{Address, Bytes, B256, U256, U64};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Response type for `zks_estimateFee`.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Eip712Fee {
    /// Amount of gas to be spent on the transaction.
    #[serde(with = "alloy::serde::quantity")]
    pub gas_limit: u64,
    /// Maximum gas user agrees to spend on a single pubdata byte published to L1.
    pub gas_per_pubdata_limit: U256,
    /// EIP-1559 gas price.
    #[serde(with = "alloy::serde::quantity")]
    pub max_fee_per_gas: u128,
    /// EIP-1559 tip.
    #[serde(with = "alloy::serde::quantity")]
    pub max_priority_fee_per_gas: u128,
}

/// Response type for `zks_getBridgeContracts`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BridgeAddresses {
    /// The address of the default shared bridge on Layer 1.
    pub l1_shared_default_bridge: Option<Address>,
    /// The address of the default shared bridge on Layer 2.
    pub l2_shared_default_bridge: Option<Address>,
    /// The address of the default ERC-20 bridge on Layer 1.
    pub l1_erc20_default_bridge: Option<Address>,
    /// The address of the default ERC-20 bridge on Layer 2.
    pub l2_erc20_default_bridge: Option<Address>,
    /// The address of the Wrapped Ethereum (WETH) bridge on Layer 1.
    pub l1_weth_bridge: Option<Address>,
    /// The address of the Wrapped Ethereum (WETH) bridge on Layer 2.
    pub l2_weth_bridge: Option<Address>,
    /// The address of the legacy shared bridge on Layer 2.
    pub l2_legacy_shared_bridge: Option<Address>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BaseSystemContractsHashes {
    /// Hash of the bootloader system contract.
    pub bootloader: B256,
    /// Hash of the default account abstraction system contract.
    pub default_aa: B256,
    /// Hash of the evm emulator system contract.
    pub evm_emulator: Option<B256>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum BlockStatus {
    Sealed,
    Verified,
}

/// Response type for `zks_getBlockDetails`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BlockDetails {
    /// Number of the block.
    pub number: u64,
    /// Corresponding L1 batch number.
    pub l1_batch_number: u64,
    /// Address of the operator who committed the block.
    pub operator_address: Address,
    /// Version of the ZKsync protocol the block was committed under.
    pub protocol_version: Option<String>,
    /// Unix timestamp when the block was committed.
    pub timestamp: u64,
    /// Number of L1 transactions included in the block.
    pub l1_tx_count: u64,
    /// Number of L2 transactions included in the block.
    pub l2_tx_count: u64,
    /// Root hash of the block's state after execution.
    pub root_hash: Option<B256>,
    /// Current status of the block: verified or sealed.
    pub status: BlockStatus,
    /// Transaction hash of the commit operation on L1.
    pub commit_tx_hash: Option<B256>,
    /// Timestamp when the block was committed on L1.
    pub committed_at: Option<DateTime<Utc>>,
    /// Transaction hash of the proof submission on L1.
    pub prove_tx_hash: Option<B256>,
    /// Timestamp when the proof was submitted on L1.
    pub proven_at: Option<DateTime<Utc>>,
    /// Transaction hash of the execution on L1.
    pub execute_tx_hash: Option<B256>,
    /// Timestamp when the block execution was completed on L1.
    pub executed_at: Option<DateTime<Utc>>,
    /// L1 gas price at the time of the block's execution.
    pub l1_gas_price: U256,
    /// Fair gas price on L2 at the time of the block's execution.
    pub l2_fair_gas_price: U256,
    /// Cost of publishing one byte (in wei).
    pub fair_pubdata_price: Option<U256>,
    /// Hashes for the base system contracts.
    pub base_system_contracts_hashes: BaseSystemContractsHashes,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum TransactionStatus {
    Pending,
    Included,
    Verified,
    Failed,
}

/// Response type for `zks_getTransactionDetails`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TransactionDetails {
    /// Indicates whether the transaction originated on Layer 1.
    pub is_l1_originated: bool,
    /// Current status of the transaction: pending, included, verified or failed.
    pub status: TransactionStatus,
    /// Transaction fee.
    pub fee: U256,
    /// Gas amount per unit of public data for this transaction.
    pub gas_per_pubdata: U256,
    /// Address of the transaction initiator.
    pub initiator_address: Address,
    /// Timestamp when the transaction was received.
    pub received_at: DateTime<Utc>,
    /// Transaction hash of the commit operation.
    pub eth_commit_tx_hash: Option<B256>,
    /// Transaction hash of the proof submission.
    pub eth_prove_tx_hash: Option<B256>,
    /// Transaction hash of the execution.
    pub eth_execute_tx_hash: Option<B256>,
}

/// Response type for `zks_getL1BatchDetails`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct L1BatchDetails {
    /// L1 batch number.
    pub number: u64,
    /// Unix timestamp when the batch was processed.
    pub timestamp: u64,
    /// Number of L1 transactions included in the batch.
    pub l1_tx_count: u64,
    /// Number of L2 transactions associated with this batch.
    pub l2_tx_count: u64,
    /// Root hash of the state after processing the batch.
    pub root_hash: Option<B256>,
    /// Current status of the batch: sealed or verified.
    pub status: BlockStatus,
    /// L1 transaction hash for the commit operation.
    pub commit_tx_hash: Option<B256>,
    /// Timestamp when the batch was committed on L1.
    pub committed_at: Option<DateTime<Utc>>,
    /// L1 transaction hash for the proof submission.
    pub prove_tx_hash: Option<B256>,
    /// Timestamp when the proof was submitted.
    pub proven_at: Option<DateTime<Utc>>,
    /// L1 transaction hash for the execution.
    pub execute_tx_hash: Option<B256>,
    /// Timestamp when the execution was completed.
    pub executed_at: Option<DateTime<Utc>>,
    /// Gas price on L1 at the time of batch processing.
    pub l1_gas_price: U256,
    /// Fair gas price on L2 at the time of batch processing.
    pub l2_fair_gas_price: U256,
    /// Cost of publishing one byte (in wei).
    pub fair_pubdata_price: Option<U256>,
    /// Hashes of the base system contracts involved in the batch.
    pub base_system_contracts_hashes: BaseSystemContractsHashes,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FeeModelConfigV2 {
    /// Minimal gas price on L2.
    pub minimal_l2_gas_price: U256,
    /// Compute overhead part in fee calculation.
    pub compute_overhead_part: f64,
    /// Public data overhead part in fee calculation.
    pub pubdata_overhead_part: f64,
    /// Overhead in L1 gas for a batch of transactions.
    pub batch_overhead_l1_gas: U256,
    /// Maximum gas allowed per batch.
    pub max_gas_per_batch: U256,
    /// Maximum amount of public data allowed per batch.
    pub max_pubdata_per_batch: U256,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BaseTokenConversionRatio {
    pub numerator: u64,
    pub denominator: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FeeParamsV2 {
    pub config: FeeModelConfigV2,
    /// L1 gas price.
    pub l1_gas_price: U256,
    /// Price of storing public data on L1.
    pub l1_pubdata_price: U256,
    /// BaseToken<->ETH conversion ratio.
    pub conversion_ratio: BaseTokenConversionRatio,
}

/// Response type for `zks_getFeeParams`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FeeParams {
    V2(FeeParamsV2),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct L1VerifierConfig {
    pub recursion_scheduler_level_vk_hash: B256,
}

/// Response type for `zks_getProtocolVersion`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProtocolVersion {
    #[serde(rename = "minorVersion")]
    pub minor_version: Option<u16>,
    /// Unix timestamp of the version's activation.
    pub timestamp: u64,
    /// Hashes of various verification keys used in the protocol.
    pub verification_keys_hashes: Option<L1VerifierConfig>,
    /// Hashes of the base system contracts.
    pub base_system_contracts: Option<BaseSystemContractsHashes>,
    /// Bootloader code hash.
    #[serde(rename = "bootloaderCodeHash")]
    pub bootloader_code_hash: Option<B256>,
    /// Default account code hash.
    #[serde(rename = "defaultAccountCodeHash")]
    pub default_account_code_hash: Option<B256>,
    /// EVM emulator code hash.
    #[serde(rename = "evmSimulatorCodeHash")]
    pub evm_emulator_code_hash: Option<B256>,
    /// Hash of the transaction used for the system upgrade
    #[serde(rename = "l2SystemUpgradeTxHash")]
    pub l2_system_upgrade_tx_hash: Option<B256>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StorageProof {
    /// Storage key for which the proof is provided.
    pub key: B256,
    /// Hashes that constitute the Merkle path from the leaf node (representing the storage key-value pair) to the root of the Merkle tree.
    /// The path is ordered from the root to the leaf. 
    /// The root hash itself is not included in this array because it is published on L1 as part of the L1 batch commit data.
    pub proof: Vec<B256>,
    /// Value stored in the specified storage key at the time of the specified l1BatchNumber.
    pub value: B256,
    /// A 1-based index representing the position of the tree entry within the Merkle tree.
    /// This index is used to help reconstruct the Merkle path during verification.
    pub index: u64,
}

/// Response type for `zks_getProof`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Proof {
    /// Account address associated with the storage proofs.
    pub address: Address,
    /// Storage proof for the requested keys.
    pub storage_proof: Vec<StorageProof>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageLog {
    pub address: Address,
    pub key: U256,
    pub written_value: U256,
}

/// A log produced by a transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Log {
    /// Address from which this log originated.
    pub address: Address,
    /// An array of 0 to 4 indexed log arguments.
    pub topics: Vec<B256>,
    /// Contains non-indexed arguments of the log.
    pub data: Bytes,
    /// Hash of the block where this log was in.
    pub block_hash: Option<B256>,
    /// Block number where this log was in.
    pub block_number: Option<U64>,
    /// L1 batch number where this log was in.
    pub l1_batch_number: Option<U64>,
    /// Hash of the transactions from which this log was created.
    pub transaction_hash: Option<B256>,
    /// Transaction index position from which the log created.
    pub transaction_index: Option<U64>,
    /// Log index position in the block.
    pub log_index: Option<U64>,
    /// Log index position in the transaction.
    pub transaction_log_index: Option<U64>,
    /// Log type.
    pub log_type: Option<String>,
    /// True when the log was removed, false if it's a valid log.
    pub removed: Option<bool>,
    /// Unix timestamp when the block was committed.
    pub block_timestamp: Option<U64>,
}

/// Response type for `zks_getL2ToL1LogProof` and `zks_getL2ToL1MsgProof`.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct L2ToL1LogProof {
    /// The Merkle proof for the message.
    pub proof: Vec<B256>,
    /// The position of the leaf in the Merkle tree of L2 to L1 messages for the block.
    pub id: u32,
    /// The root hash representing the Merkle tree root at the time the log was generated.
    pub root: B256,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Execute {
    pub contract_address: Option<Address>,
    pub calldata: Bytes,
    pub value: U256,
    /// Factory dependencies: list of contract bytecodes associated with the deploy transaction.
    pub factory_deps: Vec<Bytes>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct InputData {
    pub hash: B256,
    pub data: Bytes,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[repr(u8)]
pub enum OpProcessingType {
    Common = 0,
    OnlyRollup = 1,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq)]
#[repr(u8)]
pub enum PriorityQueueType {
    #[default]
    Deque = 0,
    HeapBuffer = 1,
    Heap = 2,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct L1TxCommonData {
    /// Sender of the transaction.
    pub sender: Address,
    /// Unique ID of the priority operation.
    pub serial_id: u64,
    /// Additional payment to the operator as an incentive to perform the operation. The contract uses a value of 192 bits.
    pub layer_2_tip_fee: U256,
    /// The total cost the sender paid for the transaction.
    pub full_fee: U256,
    /// The maximal fee per gas to be used for L1->L2 transaction
    pub max_fee_per_gas: U256,
    /// The maximum number of gas that a transaction can spend at a price of gas equals 1.
    pub gas_limit: U256,
    /// The maximum number of gas per 1 byte of pubdata.
    pub gas_per_pubdata_limit: U256,
    /// Indicator that the operation can interact with Rollup and Porter trees, or only with Rollup.
    pub op_processing_type: OpProcessingType,
    /// Priority operations queue type.
    pub priority_queue_type: PriorityQueueType,
    /// Tx hash of the transaction in the ZKsync network. Calculated as the encoded transaction data hash.
    pub canonical_tx_hash: B256,
    /// The amount of ETH that should be minted with this transaction
    pub to_mint: U256,
    /// The recipient of the refund of the transaction
    pub refund_recipient: Address,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct L2TxCommonData {
    pub nonce: u32,
    pub fee: Eip712Fee,
    pub initiator_address: Address,
    pub signature: Bytes,
    pub transaction_type: String,
    pub input: Option<InputData>,
    pub paymaster_params: PaymasterParams,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ProtocolUpgradeTxCommonData {
    /// Sender of the transaction.
    pub sender: Address,
    /// ID of the upgrade.
    pub upgrade_id: String,
    /// The maximal fee per gas to be used for L1->L2 transaction
    pub max_fee_per_gas: U256,
    /// The maximum number of gas that a transaction can spend at a price of gas equals 1.
    pub gas_limit: U256,
    /// The maximum number of gas per 1 byte of pubdata.
    pub gas_per_pubdata_limit: U256,
    /// Block in which Ethereum transaction was included.
    pub eth_block: u64,
    /// Tx hash of the transaction in the ZKsync network. Calculated as the encoded transaction data hash.
    pub canonical_tx_hash: B256,
    /// The amount of ETH that should be minted with this transaction
    pub to_mint: U256,
    /// The recipient of the refund of the transaction
    pub refund_recipient: Address,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ExecuteTransactionCommon {
    L1(L1TxCommonData),
    L2(L2TxCommonData),
    ProtocolUpgrade(ProtocolUpgradeTxCommonData),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Transaction {
    /// Common information about the transaction.
    pub common_data: ExecuteTransactionCommon,
    /// Details regarding the execution of the transaction.
    pub execute: Execute,
    /// Timestamp when the transaction was received, in milliseconds.
    pub received_timestamp_ms: u64,
    /// Raw bytes of the transaction.
    pub raw_bytes: Option<Bytes>,
}
