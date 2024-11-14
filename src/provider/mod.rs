use alloy::primitives::{Address, Bytes, B256, U256, U64};
use alloy::providers::fillers::{ChainIdFiller, JoinFill, NonceFiller, RecommendedFillers};
use alloy::providers::{Identity, Provider, ProviderBuilder, ProviderCall};
use alloy::rpc::client::NoParams;
use alloy::transports::{BoxTransport, Transport};
use chrono::{DateTime, Utc};
use fillers::Eip712FeeFiller;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::network::transaction_request::TransactionRequest;
use crate::network::unsigned_tx::eip712::PaymasterParams;
use crate::network::Zksync;

pub use self::provider_builder_ext::ProviderBuilderExt;

pub mod fillers;
pub mod layers;
mod provider_builder_ext;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockDetailsBase {
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
/// Response type for `zks_getBlockDetails`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockDetails {
    pub number: u64,
    pub l1_batch_number: u64,
    pub operator_address: Address,
    pub protocol_version: Option<String>,
    #[serde(flatten)]
    pub base: BlockDetailsBase,
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
    #[serde(flatten)]
    pub base: BlockDetailsBase,
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

/// Response type for `zks_sendRawTransactionWithDetailedOutput`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionDetailedResult {
    pub transaction_hash: B256,
    pub storage_logs: Vec<StorageLog>,
    pub events: Vec<Log>,
}

/// Response type for `zks_getConfirmedTokens`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    pub l1_address: Address,
    pub l2_address: Address,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
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
pub enum ExecuteTransactionCommon {
    L2(L2TxCommonData),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub common_data: ExecuteTransactionCommon,
    pub execute: Execute,
    pub received_timestamp_ms: u64,
    pub raw_bytes: Option<Bytes>,
}

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
pub trait ZksyncProvider<T = BoxTransport>: Provider<T, Zksync>
where
    T: Transport + Clone,
{
    /// Gets the address of the main ZKsync contract on L1.
    fn get_main_contract(&self) -> ProviderCall<T, NoParams, Address> {
        self.client().request_noparams("zks_getMainContract").into()
    }

    /// Gets the address of the testnet paymaster ZKsync contract on L2, if it's present on the network.
    fn get_testnet_paymaster(&self) -> ProviderCall<T, NoParams, Option<Address>> {
        self.client()
            .request_noparams("zks_getTestnetPaymaster")
            .into()
    }

    /// Gets the L1 Chain ID
    fn get_l1_chain_id(&self) -> ProviderCall<T, NoParams, U64> {
        self.client().request_noparams("zks_L1ChainId").into()
    }

    /// Gets the L1 batch number.
    fn get_l1_batch_number(&self) -> ProviderCall<T, NoParams, U64> {
        self.client().request_noparams("zks_L1BatchNumber").into()
    }

    /// Estimates transaction gas for EIP712 transactions.
    fn estimate_fee(
        &self,
        tx: TransactionRequest,
    ) -> ProviderCall<T, (TransactionRequest,), Eip712Fee> {
        self.client().request("zks_estimateFee", (tx,)).into()
    }

    /// Estimates the gas required for an L1 to L2 transaction.
    fn estimate_gas_l1_to_l2(
        &self,
        tx: TransactionRequest,
    ) -> ProviderCall<T, (TransactionRequest,), U256> {
        self.client().request("zks_estimateGasL1ToL2", (tx,)).into()
    }

    /// Retrieves the bridge hub contract address.
    fn get_bridgehub_contract(&self) -> ProviderCall<T, NoParams, Option<Address>> {
        self.client()
            .request_noparams("zks_getBridgehubContract")
            .into()
    }

    /// Retrieves the addresses of canonical bridge contracts for ZKsync Era.
    fn get_bridge_contracts(&self) -> ProviderCall<T, NoParams, BridgeAddresses> {
        self.client()
            .request_noparams("zks_getBridgeContracts")
            .into()
    }

    /// Retrieves the L1 base token address.
    fn get_base_token_l1_address(&self) -> ProviderCall<T, NoParams, Address> {
        self.client()
            .request_noparams("zks_getBaseTokenL1Address")
            .into()
    }

    /// Lists confirmed tokens.
    /// Confirmed in the method name means any token bridged to ZKsync Era via the official bridge.
    ///
    /// The tokens are returned in alphabetical order by their symbol.
    /// This means the token id is its position in an alphabetically sorted array of tokens.
    fn get_confirmed_tokens(&self, from: u32, limit: u8) -> ProviderCall<T, (u32, u8), Vec<Token>> {
        self.client()
            .request("zks_getConfirmedTokens", (from, limit))
            .into()
    }

    /// Gets all account balances for a given address.
    fn get_all_account_balances(
        &self,
        address: Address,
    ) -> ProviderCall<T, (Address,), HashMap<Address, U256>> {
        self.client()
            .request("zks_getAllAccountBalances", (address,))
            .into()
    }

    /// Retrieves the proof for an L2 to L1 message.
    fn get_l2_to_l1_msg_proof(
        &self,
        block_number: u64,
        sender: Address,
        msg: B256,
        l2_log_position: Option<usize>,
    ) -> ProviderCall<T, (u64, Address, B256, Option<usize>), Option<L2ToL1LogProof>> {
        self.client()
            .request(
                "zks_getL2ToL1MsgProof",
                (block_number, sender, msg, l2_log_position),
            )
            .into()
    }

    /// Retrieves the log proof for an L2 to L1 transaction.
    fn get_l2_to_l1_log_proof(
        &self,
        tx_hash: B256,
        index: Option<usize>,
    ) -> ProviderCall<T, (B256, Option<usize>), Option<L2ToL1LogProof>> {
        self.client()
            .request("zks_getL2ToL1LogProof", (tx_hash, index))
            .into()
    }

    /// Retrieves details for a given block.
    fn get_block_details(
        &self,
        block_number: u64,
    ) -> ProviderCall<T, (u64,), Option<BlockDetails>> {
        self.client()
            .request("zks_getBlockDetails", (block_number,))
            .into()
    }

    /// Retrieves details for a given transaction.
    fn get_transaction_details(
        &self,
        tx_hash: B256,
    ) -> ProviderCall<T, (B256,), Option<TransactionDetails>> {
        self.client()
            .request("zks_getTransactionDetails", (tx_hash,))
            .into()
    }

    /// Lists transactions in a block without processing them.
    fn get_raw_block_transactions(
        &self,
        block_number: u64,
    ) -> ProviderCall<T, (u64,), Vec<Transaction>> {
        self.client()
            .request("zks_getRawBlockTransactions", (block_number,))
            .into()
    }

    /// Retrieves details for a given L1 batch.
    fn get_l1_batch_details(
        &self,
        l1_batch_number: u64,
    ) -> ProviderCall<T, (u64,), Option<L1BatchDetails>> {
        self.client()
            .request("zks_getL1BatchDetails", (l1_batch_number,))
            .into()
    }

    /// Retrieves the bytecode of a transaction by its hash.
    fn get_bytecode_by_hash(&self, tx_hash: B256) -> ProviderCall<T, (B256,), Option<Bytes>> {
        self.client()
            .request("zks_getBytecodeByHash", (tx_hash,))
            .into()
    }

    /// Returns the range of blocks contained within a batch given by the batch number.
    fn get_l1_batch_block_range(
        &self,
        l1_batch_number: u64,
    ) -> ProviderCall<T, (u64,), Option<(U64, U64)>> {
        self.client()
            .request("zks_getL1BatchBlockRange", (l1_batch_number,))
            .into()
    }

    /// Retrieves the current L1 gas price.
    fn get_l1_gas_price(&self) -> ProviderCall<T, NoParams, U256> {
        self.client().request_noparams("zks_getL1GasPrice").into()
    }

    /// Retrieves the current fee parameters.
    fn get_fee_params(&self) -> ProviderCall<T, NoParams, FeeParams> {
        self.client().request_noparams("zks_getFeeParams").into()
    }

    /// Retrieves the current fee parameters.
    fn get_protocol_version(
        &self,
        version_id: Option<u16>,
    ) -> ProviderCall<T, (Option<u16>,), Option<ProtocolVersion>> {
        self.client()
            .request("zks_getProtocolVersion", (version_id,))
            .into()
    }

    /// Generates Merkle proofs for one or more storage values associated with a specific account,
    /// accompanied by a proof of their authenticity. It verifies that these values remain unaltered.
    fn get_proof(
        &self,
        address: Address,
        keys: Vec<B256>,
        l1_batch_number: u64,
    ) -> ProviderCall<T, (Address, Vec<B256>, u64), Option<Proof>> {
        self.client()
            .request("zks_getProof", (address, keys, l1_batch_number))
            .into()
    }

    /// Executes a transaction and returns its hash, storage logs, and events that
    /// would have been generated if the transaction had already been included in the block.
    /// The API has a similar behaviour to eth_sendRawTransaction
    /// but with some extra data returned from it.
    fn send_raw_transaction_with_detailed_output(
        &self,
        tx_bytes: Bytes,
    ) -> ProviderCall<T, (Bytes,), TransactionDetailedResult> {
        self.client()
            .request("zks_sendRawTransactionWithDetailedOutput", (tx_bytes,))
            .into()
    }
}

impl<P, T> ZksyncProvider<T> for P
where
    T: Transport + Clone,
    P: Provider<T, Zksync>,
{
}

impl RecommendedFillers for Zksync {
    type RecomendedFillers = JoinFill<Eip712FeeFiller, JoinFill<NonceFiller, ChainIdFiller>>;

    fn recommended_fillers() -> Self::RecomendedFillers {
        JoinFill::new(
            Eip712FeeFiller::default(),
            JoinFill::new(NonceFiller::default(), ChainIdFiller::default()),
        )
    }
}

/// Convenience function to initialize provider builder for ZKsync network.
pub fn zksync_provider() -> ProviderBuilder<Identity, Identity, Zksync> {
    ProviderBuilder::<Identity, Identity, Zksync>::default()
}
