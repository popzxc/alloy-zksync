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
    use alloy::hex::FromHex;
    use alloy::primitives::address;
    use alloy::primitives::{Address, Bytes, U256};
    use alloy::providers::{fillers::FillProvider, RootProvider};
    use alloy::transports::http::Http;
    use reqwest::Client;
    use std::net::SocketAddr;

    use crate::network::unsigned_tx::eip712::PaymasterParams;
    use alloy::network::TransactionBuilder;
    use chrono::{DateTime, Utc};
    use jsonrpsee::core::RpcResult;
    use jsonrpsee::server::{RpcModule, Server};
    use std::future::Future;

    fn str_to_utc(date_utc_str: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(date_utc_str)
            .unwrap()
            .with_timezone(&Utc)
    }
    type ZKsyncTestProvider = FillProvider<
        JoinFill<Identity, JoinFill<Eip712FeeFiller, JoinFill<NonceFiller, ChainIdFiller>>>,
        RootProvider<Http<Client>, Zksync>,
        Http<Client>,
        Zksync,
    >;
    async fn run_server_and_test<Fut>(
        register_rpc_module_fn: impl FnOnce(&mut RpcModule<()>),
        test_fn: impl FnOnce(ZKsyncTestProvider) -> Fut,
    ) where
        Fut: Future<Output = ()>,
    {
        let server = Server::builder()
            .build("127.0.0.1:0".parse::<SocketAddr>().unwrap())
            .await
            .unwrap();
        let mut module = RpcModule::new(());
        register_rpc_module_fn(&mut module);

        let server_addr: SocketAddr = server.local_addr().unwrap();
        let handle = server.start(module);
        let full_addr = format!("http://{}", server_addr);
        tokio::spawn(handle.stopped());

        let provider = zksync_provider()
            .with_recommended_fillers()
            .on_http(full_addr.parse().unwrap());
        test_fn(provider).await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_main_contract_test() {
        let network_main_contract_address = address!("32400084c286cf3e17e7b677ea9583e60a000324");
        run_server_and_test(
            |module| {
                module
                    .register_method::<RpcResult<Address>, _>(
                        "zks_getMainContract",
                        move |_, _, _| Ok(network_main_contract_address),
                    )
                    .unwrap();
            },
            |provider: ZKsyncTestProvider| async move {
                let received_main_contract_address = provider.get_main_contract().await.unwrap();
                assert_eq!(
                    network_main_contract_address,
                    received_main_contract_address
                );
            },
        )
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_testnet_paymaster_when_its_not_set() {
        run_server_and_test(
            |module| {
                module
                    .register_method::<RpcResult<Option<Address>>, _>(
                        "zks_getTestnetPaymaster",
                        move |_, _, _| Ok(None),
                    )
                    .unwrap();
            },
            |provider: ZKsyncTestProvider| async move {
                let received_paymaster_address = provider.get_testnet_paymaster().await.unwrap();
                assert_eq!(received_paymaster_address, None);
            },
        )
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_testnet_paymaster_when_its_set() {
        let network_testnet_address = address!("3cb2b87d10ac01736a65688f3e0fb1b070b3eea3");
        run_server_and_test(
            |module| {
                module
                    .register_method::<RpcResult<Option<Address>>, _>(
                        "zks_getTestnetPaymaster",
                        move |_, _, _| Ok(Some(network_testnet_address)),
                    )
                    .unwrap();
            },
            |provider: ZKsyncTestProvider| async move {
                let received_paymaster_address = provider.get_testnet_paymaster().await.unwrap();
                assert_eq!(received_paymaster_address.unwrap(), network_testnet_address);
            },
        )
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_l1_chain_id_test() {
        let network_l1_chain_id = U64::from(1);
        run_server_and_test(
            |module| {
                module
                    .register_method::<RpcResult<U64>, _>("zks_L1ChainId", move |_, _, _| {
                        Ok(network_l1_chain_id)
                    })
                    .unwrap();
            },
            |provider: ZKsyncTestProvider| async move {
                let received_l1_chain_id = provider.get_l1_chain_id().await.unwrap();
                assert_eq!(network_l1_chain_id, received_l1_chain_id);
            },
        )
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_l1_batch_number_test() {
        let network_l1_batch_number = U64::from(12345);
        run_server_and_test(
            |module| {
                module
                    .register_method::<RpcResult<U64>, _>("zks_L1BatchNumber", move |_, _, _| {
                        Ok(network_l1_batch_number)
                    })
                    .unwrap();
            },
            |provider: ZKsyncTestProvider| async move {
                let received_l1_batch_number = provider.get_l1_batch_number().await.unwrap();
                assert_eq!(network_l1_batch_number, received_l1_batch_number);
            },
        )
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn estimate_fee_test() {
        let network_fee = Eip712Fee {
            gas_limit: 40000,
            gas_per_pubdata_limit: U256::from(90000),
            max_fee_per_gas: 60000,
            max_priority_fee_per_gas: 70000,
        };

        let tx_request = TransactionRequest::default()
            .with_to(address!("1111111111111111111111111111111111111111"))
            .with_from(address!("2222222222222222222222222222222222222222"));
        let network_fee_rpc_response = network_fee.clone();

        run_server_and_test(
            move |module| {
                module
                    .register_method::<RpcResult<Eip712Fee>, _>(
                        "zks_estimateFee",
                        move |params, _, _| {
                            let (tx_request_param,) =
                                params.parse::<(TransactionRequest,)>().unwrap();
                            assert_eq!(
                                tx_request_param.to().unwrap(),
                                address!("1111111111111111111111111111111111111111")
                            );
                            assert_eq!(
                                tx_request_param.from().unwrap(),
                                address!("2222222222222222222222222222222222222222")
                            );
                            Ok(network_fee_rpc_response.clone())
                        },
                    )
                    .unwrap();
            },
            |provider: ZKsyncTestProvider| async move {
                let received_fee = provider.estimate_fee(tx_request).await.unwrap();
                assert_eq!(network_fee, received_fee);
            },
        )
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn estimate_gas_l1_to_l2_test() {
        let network_gas_estimation = U256::from(6789);

        let tx_request = TransactionRequest::default()
            .with_to(address!("1111111111111111111111111111111111111111"))
            .with_from(address!("2222222222222222222222222222222222222222"));

        run_server_and_test(
            |module| {
                module
                    .register_method::<RpcResult<U256>, _>(
                        "zks_estimateGasL1ToL2",
                        move |params, _, _| {
                            let (tx_request,) = params.parse::<(TransactionRequest,)>().unwrap();
                            assert_eq!(
                                tx_request.to().unwrap(),
                                address!("1111111111111111111111111111111111111111")
                            );
                            assert_eq!(
                                tx_request.from().unwrap(),
                                address!("2222222222222222222222222222222222222222")
                            );
                            Ok(network_gas_estimation)
                        },
                    )
                    .unwrap();
            },
            |provider: ZKsyncTestProvider| async move {
                let received_gas_estimation = provider
                    .estimate_gas_l1_to_l2(tx_request)
                    .await
                    .unwrap();
                assert_eq!(network_gas_estimation, received_gas_estimation);
            },
        )
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_bridgehub_contract_when_its_not_set() {
        run_server_and_test(
            |module| {
                module
                    .register_method::<RpcResult<Option<Address>>, _>(
                        "zks_getBridgehubContract",
                        move |_, _, _| Ok(None),
                    )
                    .unwrap();
            },
            |provider: ZKsyncTestProvider| async move {
                let received_bridge_hub_address = provider.get_bridgehub_contract().await.unwrap();
                assert_eq!(received_bridge_hub_address, None);
            },
        )
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_bridgehub_contract_when_its_set() {
        let network_bridge_hub_address = address!("3cb2b87d10ac01736a65688f3e0fb1b070b3eea3");
        run_server_and_test(
            |module| {
                module
                    .register_method::<RpcResult<Option<Address>>, _>(
                        "zks_getBridgehubContract",
                        move |_, _, _| Ok(Some(network_bridge_hub_address)),
                    )
                    .unwrap();
            },
            |provider: ZKsyncTestProvider| async move {
                let received_bridge_hub_address = provider.get_bridgehub_contract().await.unwrap();
                assert_eq!(
                    received_bridge_hub_address.unwrap(),
                    network_bridge_hub_address
                );
            },
        )
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_bridge_contracts_test() {
        let network_bridge_addresses = BridgeAddresses {
            l1_shared_default_bridge: Some(address!("1111111111111111111111111111111111111111")),
            l2_shared_default_bridge: Some(address!("2222222222222222222222222222222222222222")),
            l1_erc20_default_bridge: Some(address!("3333333333333333333333333333333333333333")),
            l2_erc20_default_bridge: Some(address!("4444444444444444444444444444444444444444")),
            l1_weth_bridge: Some(address!("5555555555555555555555555555555555555555")),
            l2_weth_bridge: Some(address!("6666666666666666666666666666666666666666")),
            l2_legacy_shared_bridge: Some(address!("7777777777777777777777777777777777777777")),
        };

        let network_bridge_addresses_rpc_response = network_bridge_addresses.clone();
        run_server_and_test(
            |module| {
                module
                    .register_method::<RpcResult<BridgeAddresses>, _>(
                        "zks_getBridgeContracts",
                        move |_, _, _| Ok(network_bridge_addresses_rpc_response.clone()),
                    )
                    .unwrap();
            },
            |provider: ZKsyncTestProvider| async move {
                let received_bridge_addresses = provider.get_bridge_contracts().await.unwrap();
                assert_eq!(received_bridge_addresses, network_bridge_addresses);
            },
        )
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_base_token_l1_address_test() {
        let network_base_token_l1_address = address!("7777777777777777777777777777777777777777");
        run_server_and_test(
            |module| {
                module
                    .register_method::<RpcResult<Address>, _>(
                        "zks_getBaseTokenL1Address",
                        move |_, _, _| Ok(network_base_token_l1_address),
                    )
                    .unwrap();
            },
            |provider: ZKsyncTestProvider| async move {
                let received_base_token_l1_address =
                    provider.get_base_token_l1_address().await.unwrap();
                assert_eq!(
                    network_base_token_l1_address,
                    received_base_token_l1_address
                );
            },
        )
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_all_account_balances_test() {
        let address = address!("7777777777777777777777777777777777777777");
        let address_balances: HashMap<Address, U256> = vec![
            (
                address!("1111111111111111111111111111111111111111"),
                U256::from(11111),
            ),
            (
                address!("2222222222222222222222222222222222222222"),
                U256::from(22222),
            ),
            (
                address!("3333333333333333333333333333333333333333"),
                U256::from(33333),
            ),
        ]
        .into_iter()
        .collect();

        let address_balances_rpc_response = address_balances.clone();
        run_server_and_test(
            |module| {
                module
                    .register_method::<RpcResult<HashMap<Address, U256>>, _>(
                        "zks_getAllAccountBalances",
                        move |params, _, _| {
                            let (address,) = params.parse::<(Address,)>().unwrap();
                            assert_eq!(
                                address,
                                address!("7777777777777777777777777777777777777777")
                            );
                            Ok(address_balances_rpc_response.clone())
                        },
                    )
                    .unwrap();
            },
            |provider: ZKsyncTestProvider| async move {
                let received_address_balances =
                    provider.get_all_account_balances(address).await.unwrap();
                assert_eq!(address_balances, received_address_balances);
            },
        )
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_l2_to_l1_msg_proof_when_it_exists() {
        let block_number = 10000_u64;
        let sender = address!("3333333333333333333333333333333333333333");
        let msg =
            B256::from_str("0x22de7debaa98758afdaee89f447ff43bab5da3de6acca7528b281cc2f1be2ee9")
                .unwrap();
        let l2_log_position = Some(10);
        let network_msg_proof = L2ToL1LogProof {
            proof: vec![
                B256::from_str(
                    "0x2a1c6c74b184965c0cb015aae9ea134fd96215d2e4f4979cfec12563295f610e",
                )
                .unwrap(),
                B256::from_str(
                    "0x0100067d861e2f5717a12c3e869cfb657793b86bbb0caa05cc1421f16c5217bc",
                )
                .unwrap(),
            ],
            id: 3000,
            root: B256::from_str(
                "0x8b65c0cf1012ea9f393197eb24619fd814379b298b238285649e14f936a5eb12",
            )
            .unwrap(),
        };
        let network_msg_proof_rpc_response = network_msg_proof.clone();
        run_server_and_test(
            |module| {
                module
                    .register_method::<RpcResult<Option<L2ToL1LogProof>>, _>(
                        "zks_getL2ToL1MsgProof",
                        move |params, _, _| {
                            let (block_num_param, sender_param, msg_param, log_position_param) =
                                params.parse::<GetMsgProofRequest>().unwrap();
                            assert_eq!(block_num_param, block_number);
                            assert_eq!(sender_param, sender);
                            assert_eq!(msg_param, msg);
                            assert_eq!(log_position_param, l2_log_position);
                            Ok(Some(network_msg_proof_rpc_response.clone()))
                        },
                    )
                    .unwrap();
            },
            |provider: ZKsyncTestProvider| async move {
                let received_msg_proof = provider
                    .get_l2_to_l1_msg_proof(block_number, sender, msg, l2_log_position)
                    .await
                    .unwrap();
                assert_eq!(network_msg_proof, received_msg_proof.unwrap());
            },
        )
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_l2_to_l1_msg_proof_when_it_does_not_exist() {
        let block_number = 10000_u64;
        let sender = address!("3333333333333333333333333333333333333333");
        let msg =
            B256::from_str("0x22de7debaa98758afdaee89f447ff43bab5da3de6acca7528b281cc2f1be2ee9")
                .unwrap();
        let l2_log_position = Some(10);

        run_server_and_test(
            |module| {
                module
                    .register_method::<RpcResult<Option<L2ToL1LogProof>>, _>(
                        "zks_getL2ToL1MsgProof",
                        move |params, _, _| {
                            let (block_num_param, sender_param, msg_param, log_position_param) =
                                params.parse::<GetMsgProofRequest>().unwrap();
                            assert_eq!(block_num_param, block_number);
                            assert_eq!(sender_param, sender);
                            assert_eq!(msg_param, msg);
                            assert_eq!(log_position_param, l2_log_position);
                            Ok(None)
                        },
                    )
                    .unwrap();
            },
            |provider: ZKsyncTestProvider| async move {
                let received_msg_proof = provider
                    .get_l2_to_l1_msg_proof(block_number, sender, msg, l2_log_position)
                    .await
                    .unwrap();
                assert_eq!(received_msg_proof, None);
            },
        )
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_l2_to_l1_log_proof_when_it_exists() {
        let tx_hash =
            B256::from_str("0x22de7debaa98758afdaee89f447ff43bab5da3de6acca7528b281cc2f1be2ee9")
                .unwrap();
        let index = Some(10);
        let network_log_proof = L2ToL1LogProof {
            proof: vec![
                B256::from_str(
                    "0x2a1c6c74b184965c0cb015aae9ea134fd96215d2e4f4979cfec12563295f610e",
                )
                .unwrap(),
                B256::from_str(
                    "0x0100067d861e2f5717a12c3e869cfb657793b86bbb0caa05cc1421f16c5217bc",
                )
                .unwrap(),
            ],
            id: 3000,
            root: B256::from_str(
                "0x8b65c0cf1012ea9f393197eb24619fd814379b298b238285649e14f936a5eb12",
            )
            .unwrap(),
        };
        let network_log_proof_rpc_response = network_log_proof.clone();
        run_server_and_test(
            |module| {
                module
                    .register_method::<RpcResult<Option<L2ToL1LogProof>>, _>(
                        "zks_getL2ToL1LogProof",
                        move |params, _, _| {
                            let (tx_hash_param, index_param) =
                                params.parse::<(B256, Option<usize>)>().unwrap();
                            assert_eq!(tx_hash_param, tx_hash);
                            assert_eq!(index_param, index);
                            Ok(Some(network_log_proof_rpc_response.clone()))
                        },
                    )
                    .unwrap();
            },
            |provider: ZKsyncTestProvider| async move {
                let received_log_proof = provider
                    .get_l2_to_l1_log_proof(tx_hash, index)
                    .await
                    .unwrap();
                assert_eq!(network_log_proof, received_log_proof.unwrap());
            },
        )
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_l2_to_l1_log_proof_when_it_does_not_exist() {
        let tx_hash =
            B256::from_str("0x22de7debaa98758afdaee89f447ff43bab5da3de6acca7528b281cc2f1be2ee9")
                .unwrap();
        let index = Some(10);

        run_server_and_test(
            |module| {
                module
                    .register_method::<RpcResult<Option<L2ToL1LogProof>>, _>(
                        "zks_getL2ToL1LogProof",
                        move |params, _, _| {
                            let (tx_hash_param, index_param) =
                                params.parse::<(B256, Option<usize>)>().unwrap();
                            assert_eq!(tx_hash_param, tx_hash);
                            assert_eq!(index_param, index);
                            Ok(None)
                        },
                    )
                    .unwrap();
            },
            |provider: ZKsyncTestProvider| async move {
                let received_log_proof = provider
                    .get_l2_to_l1_log_proof(tx_hash, index)
                    .await
                    .unwrap();
                assert_eq!(received_log_proof, None);
            },
        )
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_block_details_when_exist() {
        let network_block_details = Some(BlockDetails {
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
        });

        let network_block_details_rpc_response = network_block_details.clone();
        run_server_and_test(
            |module| {
                module
                    .register_method::<RpcResult<Option<BlockDetails>>, _>(
                        "zks_getBlockDetails",
                        move |params, _, _| {
                            let (block_number,) = params.parse::<(u64,)>().unwrap();
                            assert_eq!(block_number, 100);
                            Ok(network_block_details_rpc_response.clone())
                        },
                    )
                    .unwrap();
            },
            |provider: ZKsyncTestProvider| async move {
                let received_block_details = provider.get_block_details(100).await.unwrap();
                assert_eq!(received_block_details, network_block_details);
            },
        )
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_block_details_when_do_not_exist() {
        run_server_and_test(
            |module| {
                module
                    .register_method::<RpcResult<Option<BlockDetails>>, _>(
                        "zks_getBlockDetails",
                        move |params, _, _| {
                            let (block_number,) = params.parse::<(u64,)>().unwrap();
                            assert_eq!(block_number, 100);
                            Ok(None)
                        },
                    )
                    .unwrap();
            },
            |provider: ZKsyncTestProvider| async move {
                let received_block_details = provider.get_block_details(100).await.unwrap();
                assert_eq!(None, received_block_details);
            },
        )
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_transaction_details_when_exist() {
        let tx_hash =
            B256::from_str("0xf1adac176fc939313eea4b72055db0622a10bbd9b7a83097286e84e471d2e7df")
                .unwrap();

        let tx_details = Some(TransactionDetails {
            is_l1_originated: false,
            status: TransactionStatus::Included,
            fee: U256::from(10000),
            gas_per_pubdata: U256::from(20000),
            initiator_address: address!("3333333333333333333333333333333333333333"),
            received_at: str_to_utc("2023-03-03T23:52:24.169Z"),
            eth_commit_tx_hash: Some(
                B256::from_str(
                    "0x8b65c0cf1012ea9f393197eb24619fd814379b298b238285649e14f936a5eb12",
                )
                .unwrap(),
            ),
            eth_prove_tx_hash: Some(
                B256::from_str(
                    "0xd045e3698f018cb233c3817eb53a41a4c5b28784ffe659da246aa33bda34350c",
                )
                .unwrap(),
            ),
            eth_execute_tx_hash: Some(
                B256::from_str(
                    "0x22de7debaa98758afdaee89f447ff43bab5da3de6acca7528b281cc2f1be2ee9",
                )
                .unwrap(),
            ),
        });
        let tx_details_rpc_response = tx_details.clone();
        run_server_and_test(
            |module| {
                module
                    .register_method::<RpcResult<Option<TransactionDetails>>, _>(
                        "zks_getTransactionDetails",
                        move |params, _, _| {
                            let (tx_hash_param,) = params.parse::<(B256,)>().unwrap();
                            assert_eq!(tx_hash_param, tx_hash);
                            Ok(tx_details_rpc_response.clone())
                        },
                    )
                    .unwrap();
            },
            |provider: ZKsyncTestProvider| async move {
                let received_tx_details = provider.get_transaction_details(tx_hash).await.unwrap();
                assert_eq!(tx_details, received_tx_details);
            },
        )
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_transaction_details_when_do_not_exist() {
        let tx_hash =
            B256::from_str("0xf1adac176fc939313eea4b72055db0622a10bbd9b7a83097286e84e471d2e7df")
                .unwrap();
        run_server_and_test(
            |module| {
                module
                    .register_method::<RpcResult<Option<TransactionDetails>>, _>(
                        "zks_getTransactionDetails",
                        move |params, _, _| {
                            let (tx_hash_param,) = params.parse::<(B256,)>().unwrap();
                            assert_eq!(tx_hash_param, tx_hash);
                            Ok(None)
                        },
                    )
                    .unwrap();
            },
            |provider: ZKsyncTestProvider| async move {
                let received_tx_details = provider.get_transaction_details(tx_hash).await.unwrap();
                assert_eq!(received_tx_details, None);
            },
        )
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_raw_block_transactions_test() {
        let block_number = 10000;
        let block_txs = vec![
            Transaction {
                common_data: ExecuteTransactionCommon::L2(L2TxCommonData {
                    nonce: 1,
                    fee: Eip712Fee {
                        gas_limit: 1111111,
                        gas_per_pubdata_limit: U256::from(1111112),
                        max_fee_per_gas: 1111113,
                        max_priority_fee_per_gas: 1111114,
                    },
                    initiator_address: address!("a111111111111111111111111111111111111111"),
                    signature: Bytes::from_str(
                        "0x22de7debaa98758afdaee89f447ff43bab5da3de6acca7528b281cc2f1be2ee9",
                    )
                    .unwrap(),
                    transaction_type: "L2 tx".to_string(),
                    input: Some(InputData {
                        hash: B256::from_str(
                            "0x0100067d861e2f5717a12c3e869cfb657793b86bbb0caa05cc1421f16c5217bc",
                        )
                        .unwrap(),
                        data: Bytes::from_str(
                            "0x8b65c0cf1012ea9f393197eb24619fd814379b298b238285649e14f936a5eb12",
                        )
                        .unwrap(),
                    }),
                    paymaster_params: PaymasterParams {
                        paymaster: address!("b111111111111111111111111111111111111111"),
                        paymaster_input: Bytes::from_str(
                            "0x22de7debaa98758afdaee89f447ff43bab5da3de6acca7528b281cc2f1be2ee9",
                        )
                        .unwrap(),
                    },
                }),
                execute: Execute {
                    contract_address: Some(address!("1111111111111111111111111111111111111111")),
                    calldata: Bytes::from_hex(
                        "0x2a1c6c74b184965c0cb015aae9ea134fd96215d2e4f4979cfec12563295f610e",
                    )
                    .unwrap(),
                    value: U256::from(11111),
                    factory_deps: vec![
                        Bytes::from_hex(
                            "0x22de7debaa98758afdaee89f447ff43bab5da3de6acca7528b281cc2f1be2ee9",
                        )
                        .unwrap(),
                        Bytes::from_hex(
                            "0x0100067d861e2f5717a12c3e869cfb657793b86bbb0caa05cc1421f16c5217bc",
                        )
                        .unwrap(),
                        Bytes::from_hex(
                            "0x8b65c0cf1012ea9f393197eb24619fd814379b298b238285649e14f936a5eb12",
                        )
                        .unwrap(),
                    ],
                },
                received_timestamp_ms: 20000,
                raw_bytes: Some(
                    Bytes::from_str(
                        "0xf1adac176fc939313eea4b72055db0622a10bbd9b7a83097286e84e471d2e7df",
                    )
                    .unwrap(),
                ),
            },
            Transaction {
                common_data: ExecuteTransactionCommon::L1(L1TxCommonData {
                    sender: address!("a222222222222222222222222222222222222222"),
                    serial_id: 123,
                    layer_2_tip_fee: U256::from(2222222),
                    full_fee: U256::from(2222223),
                    max_fee_per_gas: U256::from(2222224),
                    gas_limit: U256::from(2222225),
                    gas_per_pubdata_limit: U256::from(2222226),
                    op_processing_type: OpProcessingType::Common,
                    priority_queue_type: PriorityQueueType::Heap,
                    canonical_tx_hash: B256::from_str(
                        "0x22de7debaa98758afdaee89f447ff43bab5da3de6acca7528b281cc2f1be2ee9",
                    )
                    .unwrap(),
                    to_mint: U256::from(222226),
                    refund_recipient: address!("b222222222222222222222222222222222222222"),
                }),
                execute: Execute {
                    contract_address: Some(address!("2222222222222222222222222222222222222222")),
                    calldata: Bytes::from_hex("0x2222222222222222222222222222222222222222")
                        .unwrap(),
                    value: U256::from(22222),
                    factory_deps: vec![
                        Bytes::from_hex(
                            "0x0100067d861e2f5717a12c3e869cfb657793b86bbb0caa05cc1421f16c5217bc",
                        )
                        .unwrap(),
                        Bytes::from_hex(
                            "0x8b65c0cf1012ea9f393197eb24619fd814379b298b238285649e14f936a5eb12",
                        )
                        .unwrap(),
                        Bytes::from_hex(
                            "0x22de7debaa98758afdaee89f447ff43bab5da3de6acca7528b281cc2f1be2ee9",
                        )
                        .unwrap(),
                    ],
                },
                received_timestamp_ms: 30000,
                raw_bytes: Some(
                    Bytes::from_hex(
                        "0x0100067d861e2f5717a12c3e869cfb657793b86bbb0caa05cc1421f16c5217bc",
                    )
                    .unwrap(),
                ),
            },
            Transaction {
                common_data: ExecuteTransactionCommon::ProtocolUpgrade(
                    ProtocolUpgradeTxCommonData {
                        sender: address!("a333333333333333333333333333333333333333"),
                        upgrade_id: "upgrade id".to_string(),
                        max_fee_per_gas: U256::from(33333334),
                        gas_limit: U256::from(33333335),
                        gas_per_pubdata_limit: U256::from(33333336),
                        eth_block: 345,
                        canonical_tx_hash: B256::from_str(
                            "0x8b65c0cf1012ea9f393197eb24619fd814379b298b238285649e14f936a5eb12",
                        )
                        .unwrap(),
                        to_mint: U256::from(33333337),
                        refund_recipient: address!("b333333333333333333333333333333333333333"),
                    },
                ),
                execute: Execute {
                    contract_address: Some(address!("3333333333333333333333333333333333333333")),
                    calldata: Bytes::from_hex(
                        "0x22de7debaa98758afdaee89f447ff43bab5da3de6acca7528b281cc2f1be2ee9",
                    )
                    .unwrap(),
                    value: U256::from(22222),
                    factory_deps: vec![
                        Bytes::from_hex(
                            "0x0100067d861e2f5717a12c3e869cfb657793b86bbb0caa05cc1421f16c5217bc",
                        )
                        .unwrap(),
                        Bytes::from_hex(
                            "0x8b65c0cf1012ea9f393197eb24619fd814379b298b238285649e14f936a5eb12",
                        )
                        .unwrap(),
                        Bytes::from_hex(
                            "0x22de7debaa98758afdaee89f447ff43bab5da3de6acca7528b281cc2f1be2ee9",
                        )
                        .unwrap(),
                    ],
                },
                received_timestamp_ms: 50000,
                raw_bytes: Some(
                    Bytes::from_hex(
                        "0x0100067d861e2f5717a12c3e869cfb657793b86bbb0caa05cc1421f16c5217bc",
                    )
                    .unwrap(),
                ),
            },
        ];
        let block_txs_rpc_response = block_txs.clone();
        run_server_and_test(
            |module| {
                module
                    .register_method::<RpcResult<Vec<Transaction>>, _>(
                        "zks_getRawBlockTransactions",
                        move |params, _, _| {
                            let (block_number_param,) = params.parse::<(u64,)>().unwrap();
                            assert_eq!(block_number_param, block_number);
                            Ok(block_txs_rpc_response.clone())
                        },
                    )
                    .unwrap();
            },
            |provider: ZKsyncTestProvider| async move {
                let received_block_txs = provider
                    .get_raw_block_transactions(block_number)
                    .await
                    .unwrap();
                assert_eq!(block_txs, received_block_txs);
            },
        )
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_l1_batch_details_when_exist() {
        let batch_number = 6578_u64;
        let network_batch_details = Some(L1BatchDetails {
            number: 468355,
            timestamp: 1711649164,
            l1_tx_count: 1,
            l2_tx_count: 2363,
            root_hash: Some(
                B256::from_str(
                    "0x7b31ef880f09238f13b71a0f6bfea340b9c76d01bba0712af6aa0a4f224be167",
                )
                .unwrap(),
            ),
            status: BlockStatus::Verified,
            commit_tx_hash: Some(
                B256::from_str(
                    "0x5b2598bf1260d498c1c6a05326f7416ef2a602b8a1ac0f75b583cd6e08ae83cb",
                )
                .unwrap(),
            ),
            committed_at: Some(str_to_utc("2024-03-28T18:24:49.713730Z")),
            prove_tx_hash: Some(
                B256::from_str(
                    "0xc02563331d0a83d634bc4190750e920fc26b57096ec72dd100af2ab037b43912",
                )
                .unwrap(),
            ),
            proven_at: Some(str_to_utc("2024-03-29T03:09:19.634524Z")),
            execute_tx_hash: Some(
                B256::from_str(
                    "0xbe1ba1fdd17c2421cf2dabe2908fafa26ff4fa2190a7724d16295dd9df72b144",
                )
                .unwrap(),
            ),
            executed_at: Some(str_to_utc("2024-03-29T18:18:04.204270Z")),
            l1_gas_price: U256::from(47875552051_u64),
            l2_fair_gas_price: U256::from(25000000),
            fair_pubdata_price: Some(U256::from(725000000)),
            base_system_contracts_hashes: BaseSystemContractsHashes {
                bootloader: B256::from_str(
                    "0x010007ede999d096c84553fb514d3d6ca76fbf39789dda76bfeda9f3ae06236e",
                )
                .unwrap(),
                default_aa: B256::from_str(
                    "0x0100055b041eb28aff6e3a6e0f37c31fd053fc9ef142683b05e5f0aee6934066",
                )
                .unwrap(),
                evm_emulator: Some(
                    B256::from_str(
                        "0x0100067d861e2f5717a12c3e869cfb657793b86bbb0caa05cc1421f16c5217bc",
                    )
                    .unwrap(),
                ),
            },
        });
        let network_batch_details_rpc_response = network_batch_details.clone();
        run_server_and_test(
            |module| {
                module
                    .register_method::<RpcResult<Option<L1BatchDetails>>, _>(
                        "zks_getL1BatchDetails",
                        move |params, _, _| {
                            let (batch_number_param,) = params.parse::<(u64,)>().unwrap();
                            assert_eq!(batch_number_param, batch_number);
                            Ok(network_batch_details_rpc_response.clone())
                        },
                    )
                    .unwrap();
            },
            |provider: ZKsyncTestProvider| async move {
                let received_batch_details =
                    provider.get_l1_batch_details(batch_number).await.unwrap();
                assert_eq!(network_batch_details, received_batch_details);
            },
        )
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_l1_batch_details_when_do_not_exist() {
        let batch_number = 6578_u64;
        run_server_and_test(
            |module| {
                module
                    .register_method::<RpcResult<Option<L1BatchDetails>>, _>(
                        "zks_getL1BatchDetails",
                        move |params, _, _| {
                            let (batch_number_param,) = params.parse::<(u64,)>().unwrap();
                            assert_eq!(batch_number_param, batch_number);
                            Ok(None)
                        },
                    )
                    .unwrap();
            },
            |provider: ZKsyncTestProvider| async move {
                let received_batch_details =
                    provider.get_l1_batch_details(batch_number).await.unwrap();
                assert_eq!(received_batch_details, None);
            },
        )
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_bytecode_by_hash_when_exists() {
        let tx_hash =
            B256::from_str("0x0100067d861e2f5717a12c3e869cfb657793b86bbb0caa05cc1421f16c5217bc")
                .unwrap();
        let network_tx_bytecode = Some(
            Bytes::from_str("0x0100067d861e2f5717a12c3e869cfb657793b86bbb0caa05cc1421f16c5217bc")
                .unwrap(),
        );
        let network_tx_bytecode_rpc_response = network_tx_bytecode.clone();
        run_server_and_test(
            |module| {
                module
                    .register_method::<RpcResult<Option<Bytes>>, _>(
                        "zks_getBytecodeByHash",
                        move |params, _, _| {
                            let (tx_hash_param,) = params.parse::<(B256,)>().unwrap();
                            assert_eq!(tx_hash_param, tx_hash);
                            Ok(network_tx_bytecode_rpc_response.clone())
                        },
                    )
                    .unwrap();
            },
            |provider: ZKsyncTestProvider| async move {
                let received_tx_bytecode = provider.get_bytecode_by_hash(tx_hash).await.unwrap();
                assert_eq!(network_tx_bytecode, received_tx_bytecode);
            },
        )
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_bytecode_by_hash_when_does_not_exist() {
        let tx_hash =
            B256::from_str("0x0100067d861e2f5717a12c3e869cfb657793b86bbb0caa05cc1421f16c5217bc")
                .unwrap();
        run_server_and_test(
            |module| {
                module
                    .register_method::<RpcResult<Option<Bytes>>, _>(
                        "zks_getBytecodeByHash",
                        move |params, _, _| {
                            let (tx_hash_param,) = params.parse::<(B256,)>().unwrap();
                            assert_eq!(tx_hash_param, tx_hash);
                            Ok(None)
                        },
                    )
                    .unwrap();
            },
            |provider: ZKsyncTestProvider| async move {
                let received_tx_bytecode = provider.get_bytecode_by_hash(tx_hash).await.unwrap();
                assert_eq!(received_tx_bytecode, None);
            },
        )
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_l1_batch_block_range_when_exists() {
        let l1_batch_number = 123_u64;
        let block_range = Some((U64::from(1000), U64::from(2000)));

        run_server_and_test(
            |module| {
                module
                    .register_method::<RpcResult<Option<(U64, U64)>>, _>(
                        "zks_getL1BatchBlockRange",
                        move |params, _, _| {
                            let (batch_number_param,) = params.parse::<(u64,)>().unwrap();
                            assert_eq!(batch_number_param, l1_batch_number);
                            Ok(block_range)
                        },
                    )
                    .unwrap();
            },
            |provider: ZKsyncTestProvider| async move {
                let received_blocks_range = provider
                    .get_l1_batch_block_range(l1_batch_number)
                    .await
                    .unwrap();
                assert_eq!(block_range, received_blocks_range);
            },
        )
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_l1_batch_block_range_when_does_not_exist() {
        let l1_batch_number = 123_u64;

        run_server_and_test(
            |module| {
                module
                    .register_method::<RpcResult<Option<(U64, U64)>>, _>(
                        "zks_getL1BatchBlockRange",
                        move |params, _, _| {
                            let (batch_number_param,) = params.parse::<(u64,)>().unwrap();
                            assert_eq!(batch_number_param, l1_batch_number);
                            Ok(None)
                        },
                    )
                    .unwrap();
            },
            |provider: ZKsyncTestProvider| async move {
                let received_blocks_range = provider
                    .get_l1_batch_block_range(l1_batch_number)
                    .await
                    .unwrap();
                assert_eq!(received_blocks_range, None);
            },
        )
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_l1_gas_price_test() {
        let network_l1_gas_price = U256::from(13456);
        run_server_and_test(
            |module| {
                module
                    .register_method::<RpcResult<U256>, _>("zks_getL1GasPrice", move |_, _, _| {
                        Ok(network_l1_gas_price)
                    })
                    .unwrap();
            },
            |provider: ZKsyncTestProvider| async move {
                let received_l1_gas_price = provider.get_l1_gas_price().await.unwrap();
                assert_eq!(network_l1_gas_price, received_l1_gas_price);
            },
        )
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_fee_params_test() {
        let network_fee_params = FeeParams::V2(FeeParamsV2 {
            config: FeeModelConfigV2 {
                minimal_l2_gas_price: U256::from(111111),
                compute_overhead_part: 12345_f64,
                pubdata_overhead_part: 23456_f64,
                batch_overhead_l1_gas: U256::from(222222),
                max_gas_per_batch: U256::from(3333333),
                max_pubdata_per_batch: U256::from(44444),
            },
            l1_gas_price: U256::from(555555),
            l1_pubdata_price: U256::from(66666),
            conversion_ratio: BaseTokenConversionRatio {
                numerator: 3456345_u64,
                denominator: 234344_u64,
            },
        });
        let network_fee_params_rpc_response = network_fee_params.clone();
        run_server_and_test(
            |module| {
                
                module
                    .register_method::<RpcResult<FeeParams>, _>(
                        "zks_getFeeParams",
                        move |_, _, _| Ok(network_fee_params_rpc_response.clone()),
                    )
                    .unwrap();
            },
            |provider: ZKsyncTestProvider| async move {
                let received_fee_params = provider.get_fee_params().await.unwrap();
                assert_eq!(network_fee_params, received_fee_params);
            },
        )
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_protocol_version_when_available() {
        let protocol_id = Some(123_u16);
        let network_protocol_version = Some(ProtocolVersion {
            minor_version: Some(123_u16),
            timestamp: 456778_u64,
            verification_keys_hashes: Some(L1VerifierConfig {
                recursion_scheduler_level_vk_hash: B256::from_str(
                    "0x063c6fb5c70404c2867f413a8e35563ad3d040b1ad8c11786231bfdba7b472c7",
                )
                .unwrap(),
            }),
            base_system_contracts: Some(BaseSystemContractsHashes {
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
            }),
            bootloader_code_hash: Some(
                B256::from_str(
                    "0x010007ede999d096c84553fb514d3d6ca76fbf39789dda76bfeda9f3ae06236e",
                )
                .unwrap(),
            ),
            default_account_code_hash: Some(
                B256::from_str(
                    "0x8b65c0cf1012ea9f393197eb24619fd814379b298b238285649e14f936a5eb12",
                )
                .unwrap(),
            ),
            evm_emulator_code_hash: Some(
                B256::from_str(
                    "0x22de7debaa98758afdaee89f447ff43bab5da3de6acca7528b281cc2f1be2ee9",
                )
                .unwrap(),
            ),
            l2_system_upgrade_tx_hash: Some(
                B256::from_str(
                    "0x8b65c0cf1012ea9f393197eb24619fd814379b298b238285649e14f936a5eb12",
                )
                .unwrap(),
            ),
        });
        let network_protocol_version_rpc_response = network_protocol_version.clone();
        run_server_and_test(
            |module| {
                module
                    .register_method::<RpcResult<Option<ProtocolVersion>>, _>(
                        "zks_getProtocolVersion",
                        move |params, _, _| {
                            let (protocol_id_param,) = params.parse::<(Option<u16>,)>().unwrap();
                            assert_eq!(protocol_id_param, protocol_id);
                            Ok(network_protocol_version_rpc_response.clone())
                        },
                    )
                    .unwrap();
            },
            |provider: ZKsyncTestProvider| async move {
                let received_protocol_version =
                    provider.get_protocol_version(protocol_id).await.unwrap();
                assert_eq!(network_protocol_version, received_protocol_version);
            },
        )
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_protocol_version_when_not_available() {
        let protocol_id = Some(123_u16);
        run_server_and_test(
            |module| {
                module
                    .register_method::<RpcResult<Option<ProtocolVersion>>, _>(
                        "zks_getProtocolVersion",
                        move |_, _, _| Ok(None),
                    )
                    .unwrap();
            },
            |provider: ZKsyncTestProvider| async move {
                let received_protocol_version =
                    provider.get_protocol_version(protocol_id).await.unwrap();
                assert_eq!(received_protocol_version, None);
            },
        )
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_proof_when_available() {
        let address = address!("0000000000000000000000000000000000008003");
        let keys = vec![B256::from_str(
            "0x8b65c0cf1012ea9f393197eb24619fd814379b298b238285649e14f936a5eb12",
        )
        .unwrap()];
        let l1_batch_number = 354895_u64;
        let proof = Some(Proof {
            address: address!("0000000000000000000000000000000000008003"),
            storage_proof: vec![StorageProof {
                key: B256::from_str(
                    "0x8b65c0cf1012ea9f393197eb24619fd814379b298b238285649e14f936a5eb12",
                )
                .unwrap(),
                proof: vec![B256::from_str(
                    "0xe3e8e49a998b3abf8926f62a5a832d829aadc1b7e059f1ea59ffbab8e11edfb7",
                )
                .unwrap()],
                value: B256::from_str(
                    "0x0000000000000000000000000000000000000000000000000000000000000060",
                )
                .unwrap(),
                index: 27900957_u64,
            }],
        });
        let proof_rpc_response = proof.clone();
        let keys_rpc_request = keys.clone();
        run_server_and_test(
            |module| {
                module
                    .register_method::<RpcResult<Option<Proof>>, _>(
                        "zks_getProof",
                        move |params, _, _| {
                            let (address_param, keys_param, batch_num_param) =
                                params.parse::<(Address, Vec<B256>, u64)>().unwrap();
                            assert_eq!(address_param, address);
                            assert_eq!(keys_param, keys_rpc_request);
                            assert_eq!(batch_num_param, l1_batch_number);
                            Ok(proof_rpc_response.clone())
                        },
                    )
                    .unwrap();
            },
            |provider: ZKsyncTestProvider| async move {
                let received_proof =
                    ZksyncProvider::get_proof(&provider, address, keys, l1_batch_number)
                        .await
                        .unwrap();
                assert_eq!(proof, received_proof);
            },
        )
        .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_proof_when_not_available() {
        let address = address!("0000000000000000000000000000000000008003");
        let keys = vec![B256::from_str(
            "0x8b65c0cf1012ea9f393197eb24619fd814379b298b238285649e14f936a5eb12",
        )
        .unwrap()];
        let l1_batch_number = 354895_u64;
        let keys_rpc_request = keys.clone();
        run_server_and_test(
            |module| {
                module
                    .register_method::<RpcResult<Option<Proof>>, _>(
                        "zks_getProof",
                        move |params, _, _| {
                            let (address_param, keys_param, batch_num_param) =
                                params.parse::<(Address, Vec<B256>, u64)>().unwrap();
                            assert_eq!(address_param, address);
                            assert_eq!(keys_param, keys_rpc_request);
                            assert_eq!(batch_num_param, l1_batch_number);
                            Ok(None)
                        },
                    )
                    .unwrap();
            },
            |provider: ZKsyncTestProvider| async move {
                let received_proof =
                    ZksyncProvider::get_proof(&provider, address, keys, l1_batch_number)
                        .await
                        .unwrap();
                assert_eq!(received_proof, None);
            },
        )
        .await;
    }
}
