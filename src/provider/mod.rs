use alloy::primitives::{Address, TxHash, U256, U64};
use alloy::providers::fillers::{ChainIdFiller, JoinFill, NonceFiller, RecommendedFillers};
use alloy::providers::{Identity, Provider, ProviderBuilder, ProviderCall};
use alloy::rpc::client::NoParams;
use alloy::transports::{BoxTransport, Transport};
use fillers::Eip712FeeFiller;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use zksync_types::{
    api::{BridgeAddresses, TransactionDetails},
    Transaction,
};
use zksync_web3_decl::types::Token;

pub use self::provider_builder_ext::ProviderBuilderExt;
use crate::network::transaction_request::TransactionRequest;
use crate::network::Zksync;

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
    fn get_testnet_paymaster(&self) -> ProviderCall<T, NoParams, Address> {
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

    /// Estimate transaction gas for EIP712 transactions.
    fn estimate_fee(
        &self,
        tx: TransactionRequest,
    ) -> ProviderCall<T, (TransactionRequest,), Eip712Fee> {
        self.client().request("zks_estimateFee", (tx,)).into()
    }

    /// Retrieves the L1 base token address.
    fn get_base_token_l1_address(&self) -> ProviderCall<T, NoParams, Address> {
        self.client()
            .request_noparams("zks_getBaseTokenL1Address")
            .into()
    }

    /// Retrieves details for a given transaction.
    fn get_transaction_details(
        &self,
        hash: TxHash,
    ) -> ProviderCall<T, (TxHash,), TransactionDetails> {
        self.client()
            .request("zks_getTransactionDetails", (hash,))
            .into()
    }

    /// Lists raw transactions in a specified block without processing them.
    fn get_raw_block_transactions(
        &self,
        block_number: u32,
    ) -> ProviderCall<T, (u32,), Vec<Transaction>> {
        self.client()
            .request("zks_getRawBlockTransactions", (block_number,))
            .into()
    }

    /// Retrieves the addresses of canonical bridge contracts.
    fn get_bridge_contracts(&self) -> ProviderCall<T, NoParams, BridgeAddresses> {
        self.client()
            .request_noparams("zks_getBridgeContracts")
            .into()
    }

    /// Lists confirmed tokens that were bridged to zkSync Era via the official bridge.
    fn get_confirmed_tokens(
        &self,
        start_id: u32,
        limit: u8,
    ) -> ProviderCall<T, (u32, u8), Vec<Token>> {
        self.client()
            .request("zks_getConfirmedTokens", (start_id, limit))
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
