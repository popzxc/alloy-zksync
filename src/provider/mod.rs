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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use alloy::primitives::address;
    use alloy::primitives::{Address, U256};
    use std::net::SocketAddr;

    use chrono::{DateTime, Utc};
    use jsonrpsee::core::RpcResult;
    use jsonrpsee::server::{RpcModule, Server};

    fn str_to_utc(date_utc_str: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(date_utc_str)
            .unwrap()
            .with_timezone(&Utc)
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_testnet_paymaster_when_its_not_set() {
        let server = Server::builder()
            .build("127.0.0.1:0".parse::<SocketAddr>().unwrap())
            .await
            .unwrap();
        let mut module = RpcModule::new(());
        module
            .register_method::<RpcResult<Option<Address>>, _>(
                "zks_getTestnetPaymaster",
                move |_, _, _| Ok(None),
            )
            .unwrap();

        let server_addr: SocketAddr = server.local_addr().unwrap();
        let handle = server.start(module);
        let full_addr = format!("http://{}", server_addr);
        tokio::spawn(handle.stopped());

        let provider = zksync_provider()
            .with_recommended_fillers()
            .on_http(full_addr.parse().unwrap());

        let received_paymaster_address = provider.get_testnet_paymaster().await.unwrap();
        assert_eq!(received_paymaster_address, None);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_testnet_paymaster_when_its_set() {
        let server = Server::builder()
            .build("127.0.0.1:0".parse::<SocketAddr>().unwrap())
            .await
            .unwrap();
        let mut module = RpcModule::new(());
        let network_testnet_address = address!("3cb2b87d10ac01736a65688f3e0fb1b070b3eea3");
        module
            .register_method::<RpcResult<Option<Address>>, _>(
                "zks_getTestnetPaymaster",
                move |_, _, _| Ok(Some(network_testnet_address)),
            )
            .unwrap();

        let server_addr: SocketAddr = server.local_addr().unwrap();
        let handle = server.start(module);
        let full_addr = format!("http://{}", server_addr);
        tokio::spawn(handle.stopped());

        let provider = zksync_provider()
            .with_recommended_fillers()
            .on_http(full_addr.parse().unwrap());

        let received_paymaster_address = provider.get_testnet_paymaster().await.unwrap();
        assert_eq!(received_paymaster_address.unwrap(), network_testnet_address);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_block_details_when_its_available() {
        let server = Server::builder()
            .build("127.0.0.1:0".parse::<SocketAddr>().unwrap())
            .await
            .unwrap();
        let mut module: RpcModule<()> = RpcModule::new(());
        let network_block_details = BlockDetails {
            number: 140599,
            l1_batch_number: 1617,
            l1_tx_count: 0,
            l2_tx_count: 20,
            timestamp: 1679815038,
            fair_pubdata_price: Some(U256::from(7069038)),
            root_hash: Some(
                B256::from_str(
                    "0xf1adac176fc939313eea4b72055db0622a10bbd9b7a83097286e84e471d2e7df",
                )
                .unwrap(),
            ),
            status: BlockStatus::Verified,
            commit_tx_hash: Some(
                B256::from_str(
                    "0xd045e3698f018cb233c3817eb53a41a4c5b28784ffe659da246aa33bda34350c",
                )
                .unwrap(),
            ),
            committed_at: Some(str_to_utc("2023-03-26T07:21:21.046817Z")),
            prove_tx_hash: Some(
                B256::from_str(
                    "0x1591e9b16ff6eb029cc865614094b2e6dd872c8be40b15cc56164941ed723a1a",
                )
                .unwrap(),
            ),
            proven_at: Some(str_to_utc("2023-03-26T19:48:35.200565Z")),
            execute_tx_hash: Some(
                B256::from_str(
                    "0xbb66aa75f437bb4255cf751badfc6b142e8d4d3a4e531c7b2e737a22870ff19e",
                )
                .unwrap(),
            ),
            executed_at: Some(str_to_utc("2023-03-27T07:44:52.187764Z")),
            l1_gas_price: U256::from(2069038),
            l2_fair_gas_price: U256::from(250000000),
            base_system_contracts_hashes: BaseSystemContractsHashes {
                bootloader: B256::from_str(
                    "0x010007793a328ef16cc7086708f7f3292ff9b5eed9e7e539c184228f461bf4ef",
                )
                .unwrap(),
                default_aa: B256::from_str(
                    "0x0100067d861e2f5717a12c3e869cfb657793b86bbb0caa05cc1421f16c5217bc",
                )
                .unwrap(),
                evm_emulator: Some(
                    B256::from_str(
                        "0x0100057d861e2f5717a12c3e869cfb657793b86bbb0caa05cc1421f16c5217bc",
                    )
                    .unwrap(),
                ),
            },
            operator_address: address!("feee860e7aae671124e9a4e61139f3a5085dfeee"),
            protocol_version: Some("Version5".to_string()),
        };
        let network_block_details_rpc_response = network_block_details.clone();
        module
            .register_method::<RpcResult<Option<BlockDetails>>, _>(
                "zks_getBlockDetails",
                move |params, _, _| {
                    let (block_number,) = params.parse::<(u64,)>().unwrap();
                    assert_eq!(block_number, 100);
                    Ok(Some(network_block_details_rpc_response.clone()))
                },
            )
            .unwrap();

        let server_addr: SocketAddr = server.local_addr().unwrap();
        let handle = server.start(module);
        let full_addr = format!("http://{}", server_addr);
        tokio::spawn(handle.stopped());

        let provider = zksync_provider()
            .with_recommended_fillers()
            .on_http(full_addr.parse().unwrap());

        let received_block_details = provider.get_block_details(100).await.unwrap();
        assert_eq!(received_block_details.unwrap(), network_block_details);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_block_details_when_its_not_available() {
        let server = Server::builder()
            .build("127.0.0.1:0".parse::<SocketAddr>().unwrap())
            .await
            .unwrap();
        let mut module = RpcModule::new(());
        module
            .register_method::<RpcResult<Option<BlockDetails>>, _>(
                "zks_getBlockDetails",
                move |_, _, _| Ok(None),
            )
            .unwrap();

        let server_addr: SocketAddr = server.local_addr().unwrap();
        let handle = server.start(module);
        let full_addr = format!("http://{}", server_addr);
        tokio::spawn(handle.stopped());

        let provider = zksync_provider()
            .with_recommended_fillers()
            .on_http(full_addr.parse().unwrap());

        let received_block_details = provider.get_block_details(100).await.unwrap();
        assert_eq!(received_block_details, None);
    }
}
