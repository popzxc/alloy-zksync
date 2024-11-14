use crate::network::transaction_request::TransactionRequest;
use crate::network::Zksync;
use crate::types::*;
use alloy::primitives::{Address, Bytes, B256, U256, U64};
use alloy::providers::fillers::{ChainIdFiller, JoinFill, NonceFiller, RecommendedFillers};
use alloy::providers::{Identity, Provider, ProviderBuilder, ProviderCall};
use alloy::rpc::client::NoParams;
use alloy::transports::{BoxTransport, Transport};
use fillers::Eip712FeeFiller;
use std::collections::HashMap;

pub use self::provider_builder_ext::ProviderBuilderExt;

pub mod fillers;
pub mod layers;
mod provider_builder_ext;

type GetMsgProofRequest = (u64, Address, B256, Option<usize>);

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
    ) -> ProviderCall<T, GetMsgProofRequest, Option<L2ToL1LogProof>> {
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

    /// Lists transactions in a native encoding (e.g. that has more details, but does not
    /// adhere to the "common" Web3 Transaction interface).
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
