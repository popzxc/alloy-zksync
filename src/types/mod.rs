use crate::network::unsigned_tx::eip712::PaymasterParams;
use alloy::primitives::{Address, Bytes, B256, U256, U64};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Response type for `zks_estimateFee`.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeAddresses {
    pub l1_shared_default_bridge: Option<Address>,
    pub l2_shared_default_bridge: Option<Address>,
    pub l1_erc20_default_bridge: Option<Address>,
    pub l2_erc20_default_bridge: Option<Address>,
    pub l1_weth_bridge: Option<Address>,
    pub l2_weth_bridge: Option<Address>,
    pub l2_legacy_shared_bridge: Option<Address>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseSystemContractsHashes {
    pub bootloader: B256,
    pub default_aa: B256,
    pub evm_emulator: Option<B256>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BlockStatus {
    Sealed,
    Verified,
}

/// Response type for `zks_getBlockDetails`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockDetails {
    pub number: u64,
    pub l1_batch_number: u64,
    pub operator_address: Address,
    pub protocol_version: Option<String>,
    pub timestamp: u64,
    pub l1_tx_count: u64,
    pub l2_tx_count: u64,
    pub root_hash: Option<B256>,
    pub status: BlockStatus,
    pub commit_tx_hash: Option<B256>,
    pub committed_at: Option<DateTime<Utc>>,
    pub prove_tx_hash: Option<B256>,
    pub proven_at: Option<DateTime<Utc>>,
    pub execute_tx_hash: Option<B256>,
    pub executed_at: Option<DateTime<Utc>>,
    pub l1_gas_price: U256,
    pub l2_fair_gas_price: U256,
    pub fair_pubdata_price: Option<U256>,
    pub base_system_contracts_hashes: BaseSystemContractsHashes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TransactionStatus {
    Pending,
    Included,
    Verified,
    Failed,
}

/// Response type for `zks_getTransactionDetails`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionDetails {
    pub is_l1_originated: bool,
    pub status: TransactionStatus,
    pub fee: U256,
    pub gas_per_pubdata: U256,
    pub initiator_address: Address,
    pub received_at: String,
    pub eth_commit_tx_hash: Option<B256>,
    pub eth_prove_tx_hash: Option<B256>,
    pub eth_execute_tx_hash: Option<B256>,
}

/// Response type for `zks_getL1BatchDetails`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct L1BatchDetails {
    pub number: u64,
    pub timestamp: u64,
    pub l1_tx_count: u64,
    pub l2_tx_count: u64,
    pub root_hash: Option<B256>,
    pub status: BlockStatus,
    pub commit_tx_hash: Option<B256>,
    pub committed_at: Option<DateTime<Utc>>,
    pub prove_tx_hash: Option<B256>,
    pub proven_at: Option<DateTime<Utc>>,
    pub execute_tx_hash: Option<B256>,
    pub executed_at: Option<DateTime<Utc>>,
    pub l1_gas_price: U256,
    pub l2_fair_gas_price: U256,
    pub fair_pubdata_price: Option<U256>,
    pub base_system_contracts_hashes: BaseSystemContractsHashes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeModelConfigV2 {
    pub minimal_l2_gas_price: U256,
    pub compute_overhead_part: f64,
    pub pubdata_overhead_part: f64,
    pub batch_overhead_l1_gas: U256,
    pub max_gas_per_batch: U256,
    pub max_pubdata_per_batch: U256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseTokenConversionRatio {
    pub numerator: u64,
    pub denominator: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeParamsV2 {
    config: FeeModelConfigV2,
    l1_gas_price: U256,
    l1_pubdata_price: U256,
    conversion_ratio: BaseTokenConversionRatio,
}

/// Response type for `zks_getFeeParams`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeeParams {
    V2(FeeParamsV2),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L1VerifierConfig {
    pub recursion_scheduler_level_vk_hash: B256,
}

/// Response type for `zks_getProtocolVersion`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolVersion {
    #[serde(rename = "minorVersion")]
    pub minor_version: Option<u16>,
    pub timestamp: u64,
    pub verification_keys_hashes: Option<L1VerifierConfig>,
    pub base_system_contracts: Option<BaseSystemContractsHashes>,
    #[serde(rename = "bootloaderCodeHash")]
    pub bootloader_code_hash: Option<B256>,
    #[serde(rename = "defaultAccountCodeHash")]
    pub default_account_code_hash: Option<B256>,
    #[serde(rename = "evmSimulatorCodeHash")]
    pub evm_emulator_code_hash: Option<B256>,
    #[serde(rename = "l2SystemUpgradeTxHash")]
    pub l2_system_upgrade_tx_hash: Option<B256>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageProof {
    pub key: B256,
    pub proof: Vec<B256>,
    pub value: B256,
    pub index: u64,
}

/// Response type for `zks_getProof`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Proof {
    pub address: Address,
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
    pub address: Address,
    pub topics: Vec<B256>,
    pub data: Bytes,
    pub block_hash: Option<B256>,
    pub block_number: Option<U64>,
    pub l1_batch_number: Option<U64>,
    pub transaction_hash: Option<B256>,
    pub transaction_index: Option<U64>,
    pub log_index: Option<U64>,
    pub transaction_log_index: Option<U64>,
    pub log_type: Option<String>,
    pub removed: Option<bool>,
    pub block_timestamp: Option<U64>,
}

/// Response type for `zks_getL2ToL1LogProof`.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct L2ToL1LogProof {
    /// The merkle path for the leaf.
    pub proof: Vec<B256>,
    /// The id of the leaf in a tree.
    pub id: u32,
    /// The root of the tree.
    pub root: B256,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Execute {
    pub contract_address: Option<Address>,
    pub calldata: Bytes,
    pub value: U256,
    /// Factory dependencies: list of contract bytecodes associated with the deploy transaction.
    pub factory_deps: Vec<Bytes>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InputData {
    pub hash: B256,
    pub data: Bytes,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[repr(u8)]
pub enum OpProcessingType {
    Common = 0,
    OnlyRollup = 1,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[repr(u8)]
pub enum PriorityQueueType {
    #[default]
    Deque = 0,
    HeapBuffer = 1,
    Heap = 2,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ExecuteTransactionCommon {
    L1(L1TxCommonData),
    L2(L2TxCommonData),
    ProtocolUpgrade(ProtocolUpgradeTxCommonData),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub common_data: ExecuteTransactionCommon,
    pub execute: Execute,
    pub received_timestamp_ms: u64,
    pub raw_bytes: Option<Bytes>,
}
