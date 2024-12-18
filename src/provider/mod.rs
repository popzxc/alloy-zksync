pub use self::provider_builder_ext::ProviderBuilderExt;
use crate::{
    contracts::{
        common::erc20::{encode_token_data_for_bridge, ERC20},
        l1::{
            bridge_hub::{
                Bridgehub::{self},
                L2TransactionRequestDirect, L2TransactionRequestTwoBridges,
            },
            l1_bridge::{encode_deposit_token_calldata, L1Bridge},
        },
        l2::l2_bridge::encode_finalize_deposit_calldata,
    },
    network::{transaction_request::TransactionRequest, Zksync},
    types::*,
    utils::apply_l1_to_l2_alias,
};
use alloy::{
    contract::{CallBuilder, CallDecoder},
    network::{Ethereum, NetworkWallet, TransactionBuilder},
    primitives::{Address, Bytes, B256, U256, U64},
    providers::{
        fillers::{ChainIdFiller, JoinFill, NonceFiller, RecommendedFillers},
        utils::Eip1559Estimation,
        Identity, PendingTransactionBuilder, Provider, ProviderBuilder, ProviderCall, RootProvider,
        WalletProvider,
    },
    rpc::{client::NoParams, types::eth::TransactionReceipt},
    transports::{BoxTransport, Transport},
};
use fillers::Eip712FeeFiller;
use std::{collections::HashMap, str::FromStr};

pub mod fillers;
pub mod layers;
mod provider_builder_ext;

pub const ETHER_L1_ADDRESS: Address = Address::new([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
]);

pub const REQUIRED_L1_TO_L2_GAS_PER_PUBDATA_LIMIT: u64 = 800;

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

    /// Gets the L1 Chain ID.
    fn get_l1_chain_id(&self) -> ProviderCall<T, NoParams, U64> {
        self.client().request_noparams("zks_L1ChainId").into()
    }

    /// Gets the latest L1 batch number.
    fn get_l1_batch_number(&self) -> ProviderCall<T, NoParams, U64> {
        self.client().request_noparams("zks_L1BatchNumber").into()
    }

    /// Estimates transaction gas for a transaction.
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
    ///
    /// ## Parameters
    ///
    /// - `address`: an account address.
    ///
    /// ## Returns
    ///
    /// A hashmap with token addresses as keys and their corresponding balances as values.
    /// Each key-value pair represents the balance of a specific token held by the account.
    fn get_all_account_balances(
        &self,
        address: Address,
    ) -> ProviderCall<T, (Address,), HashMap<Address, U256>> {
        self.client()
            .request("zks_getAllAccountBalances", (address,))
            .into()
    }

    /// Retrieves the proof for an L2 to L1 message.
    ///
    /// ## Parameters
    ///
    /// - `block_number`: the block number where the message was emitted.
    /// - `sender`: The sender of the message.
    /// - `msg`: The keccak256 hash of the sent message.
    /// - `l2_log_position`:  Optional: The index in the block of the event that was emitted by the L1Messenger when submitting this message.
    /// If it is omitted, the proof for the first message is returned.
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
    ///
    /// # Parameters
    ///
    /// - `tx_hash`: hash of the L2 transaction the L2 to L1 log was produced in.
    /// - `l2_to_l1_log_index`: Optional: The index of the L2 to L1 log in the transaction.
    fn get_l2_to_l1_log_proof(
        &self,
        tx_hash: B256,
        l2_to_l1_log_index: Option<usize>,
    ) -> ProviderCall<T, (B256, Option<usize>), Option<L2ToL1LogProof>> {
        self.client()
            .request("zks_getL2ToL1LogProof", (tx_hash, l2_to_l1_log_index))
            .into()
    }

    /// Retrieves details for a given L2 block.
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

    /// Gets the protocol version.
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
    ///
    /// ## Parameters
    ///
    /// - `address`: account address to fetch storage values and proofs for.
    /// - `keys`: the keys in the account.
    /// - `l1_batch_number`: number of the L1 batch specifying the point in time at which the requested values are returned.
    ///
    /// ## Returns
    ///
    /// The account details and proofs for storage keys.
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

/// Scales the gas limit to ensure the transaction will be accepted.
// Gas limit scaling logic is taken from the JS SDK:
// https://github.com/zksync-sdk/zksync-ethers/blob/64763688d1bb5cee4a4c220c3841b803c74b0d05/src/utils.ts#L1451
pub fn scale_l1_gas_limit(l1_gas_limit: u64) -> u64 {
    /// Numerator used in scaling the gas limit to ensure acceptance of `L1->L2` transactions.
    /// This constant is part of a coefficient calculation to adjust the gas limit to account for variations
    /// in the SDK estimation, ensuring the transaction will be accepted.
    const L1_FEE_ESTIMATION_COEF_NUMERATOR: u64 = 12;

    /// Denominator used in scaling the gas limit to ensure acceptance of `L1->L2` transactions.
    /// This constant is part of a coefficient calculation to adjust the gas limit to account for variations
    /// in the SDK estimation, ensuring the transaction will be accepted.
    const L1_FEE_ESTIMATION_COEF_DENOMINATOR: u64 = 10;
    l1_gas_limit * L1_FEE_ESTIMATION_COEF_NUMERATOR / L1_FEE_ESTIMATION_COEF_DENOMINATOR
}

async fn get_gas_limit_for_l1_tx<Tr, P, D, Pr>(
    tx_builder: &CallBuilder<Tr, P, D, Ethereum>,
    l1_provider: &Pr,
) -> Result<u64, L1CommunicationError>
where
    Tr: Transport + Clone,
    Pr: alloy::providers::Provider<Tr, Ethereum>,
    D: CallDecoder,
    CallBuilder<Tr, P, D, Ethereum>: Clone,
{
    let tx_request = tx_builder.clone().into_transaction_request();
    let l1_tx_gas_estimation = l1_provider.estimate_gas(&tx_request).await.map_err(|_| {
        L1CommunicationError::Custom(
            "Error occurred while estimating gas limit for the L1 transaction.",
        )
    })?;
    let l1_gas_limit = scale_l1_gas_limit(l1_tx_gas_estimation);
    Ok(l1_gas_limit)
}

/// Enum to describe errors that might occur during L1 -> L2 communication.
#[derive(Debug, thiserror::Error)]
pub enum L1CommunicationError {
    #[error("NewPriorityRequest event log was not found in L1 -> L2 transaction.")]
    NewPriorityRequestLogNotFound,
    #[error("Custom L1 -> L2 communication error.")]
    Custom(&'static str),
}

/// A wrapper struct to hold L1 transaction receipt and L2 provider
/// which is used by the associated functions.
pub struct L1TransactionReceipt<T> {
    /// Ethereum transaction receipt.
    inner: TransactionReceipt,
    /// A reference to the L2 provider.
    l2_provider: RootProvider<T, Zksync>,
}

impl<T> L1TransactionReceipt<T>
where
    T: Transport + Clone,
{
    pub fn new(tx_receipt: TransactionReceipt, l2_provider: RootProvider<T, Zksync>) -> Self {
        Self {
            inner: tx_receipt,
            l2_provider,
        }
    }

    pub fn get_receipt(&self) -> &TransactionReceipt {
        &self.inner
    }

    pub fn get_l2_tx(&self) -> Result<PendingTransactionBuilder<T, Zksync>, L1CommunicationError> {
        let l1_to_l2_tx_log = self
            .inner
            .inner
            .logs()
            .iter()
            .filter_map(|log| log.log_decode::<Bridgehub::NewPriorityRequest>().ok())
            .next()
            .ok_or(L1CommunicationError::NewPriorityRequestLogNotFound)?;

        let l2_tx_hash = l1_to_l2_tx_log.inner.txHash;

        Ok(PendingTransactionBuilder::new(
            self.l2_provider.clone(),
            l2_tx_hash,
        ))
    }
}

/// Type for deposit request
#[derive(Clone, Debug)]
pub struct DepositRequest {
    /// Amount to deposit in Wei.
    pub amount: U256,
    /// Receiver of deposited assets. If None, the sender address will be used as a receiver.
    pub receiver: Option<Address>,
    /// L1 token address to deposit.
    pub token: Address,
    /// Bridge address for the deposit. If None, default shared bridge will be used.
    pub bridge_address: Option<Address>,
    /// Gas per pubdata limit to use in initiated transactions. If None,
    /// REQUIRED_L1_TO_L2_GAS_PER_PUBDATA_LIMIT will be used.
    pub gas_per_pubdata_limit: U256,
}

impl DepositRequest {
    pub fn new(amount: U256) -> Self {
        Self {
            amount,
            receiver: None,
            token: ETHER_L1_ADDRESS,
            bridge_address: None,
            gas_per_pubdata_limit: U256::from(REQUIRED_L1_TO_L2_GAS_PER_PUBDATA_LIMIT),
        }
    }

    pub fn with_amount(&self) -> &U256 {
        &self.amount
    }

    pub fn with_receiver(mut self, address: Address) -> Self {
        self.receiver = Some(address);
        self
    }

    pub fn with_token(mut self, token: Address) -> Self {
        self.token = token;
        self
    }

    pub fn with_gas_per_pubdata_limit(mut self, value: U256) -> Self {
        self.gas_per_pubdata_limit = value;
        self
    }

    pub fn with_bridge_address(mut self, bridge_address: Address) -> Self {
        self.bridge_address = Some(bridge_address);
        self
    }
}

/// Type to represent gas limit and transaction base cost for the bridge L2 transaction.
#[derive(Clone, Debug)]
pub struct BridgeL2TxGasData {
    pub gas_limit: U256,
    pub tx_base_cost: U256,
}
/// Trait for ZKsync provider with populated wallet
/// Contains provider methods that need a wallet
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
pub trait ZksyncProviderWithWallet<T = BoxTransport>:
    ZksyncProvider<T> + WalletProvider<Zksync>
where
    T: Transport + Clone,
{
    /// Returns l1 and l2 bridge addresses for the deposit request.
    /// If deposit_request.bridge_address is None, takes both addresses using get_bridge_contracts method.
    /// If deposit_request.bridge_address is defined, calls bridge.l2BridgeAddress
    /// to get corresponding l2 bridge address and returns both.
    ///
    /// ## Parameters
    ///
    /// - `deposit_request`: deposit request to get bridge addresses for.
    /// - `l2_chain_id`: L2 chain id.
    /// - `l1_provider`: L1 provider.
    ///
    /// ## Returns
    ///
    /// A tuple with l1 and l2 bridge addresses for the deposit request.
    async fn get_bridge_addresses_for_deposit<Tr, P>(
        &self,
        deposit_request: &DepositRequest,
        l2_chain_id: U256,
        l1_provider: &P,
    ) -> Result<(Address, Address), L1CommunicationError>
    where
        Tr: Transport + Clone,
        P: alloy::providers::Provider<Tr, Ethereum>,
    {
        let (l1_bridge_address, l2_bridge_address) = match deposit_request.bridge_address {
            Some(l1_bridge_address) => {
                let l1_bridge = L1Bridge::new(l1_bridge_address, l1_provider);
                let l2_bridge_address = l1_bridge
                    .l2BridgeAddress(l2_chain_id)
                    .call()
                    .await
                    .map_err(|_| {
                        L1CommunicationError::Custom("Error while getting L2 bridge address.")
                    })?
                    ._0;
                (l1_bridge_address, l2_bridge_address)
            }
            None => {
                let bridge_addresses = self.get_bridge_contracts().await.map_err(|_| {
                    L1CommunicationError::Custom("Error occurred while fetching bridge contracts.")
                })?;
                (
                    bridge_addresses.l1_shared_default_bridge.ok_or(
                        L1CommunicationError::Custom(
                            "L1 shared default bridge is not defined for the chain and bridge address is not specified in the deposit request.",
                        ),
                    )?,
                    bridge_addresses.l2_shared_default_bridge.ok_or(
                        L1CommunicationError::Custom(
                            "L2 shared default bridge is not defined for the chain.",
                        ),
                    )?,
                )
            }
        };
        Ok((l1_bridge_address, l2_bridge_address))
    }

    /// Returns gas data for the bridge L2 transaction.
    ///
    /// ## Parameters
    ///
    /// - `bridge_hub_contract`: an instance of the bridge hub contract.
    /// - `l1_to_l2_tx`: transaction request, to calculate gas limit and base cost for.
    /// - `l2_chain_id`: L2 chain id.
    /// - `max_fee_per_gas`: max fee per gas.
    /// - `gas_per_pubdata_limit` - gas per pubdata limit.
    ///
    /// ## Returns
    ///
    /// Gas data for the  L2 bridge transaction which contains gas limit and tx base cost.
    async fn get_bridge_l2_tx_gas_data<Tr, P>(
        &self,
        bridge_hub_contract: &Bridgehub::BridgehubInstance<Tr, P, Ethereum>,
        l1_to_l2_tx: TransactionRequest,
        l2_chain_id: U256,
        max_fee_per_gas: U256,
        gas_per_pubdata_limit: U256,
    ) -> Result<BridgeL2TxGasData, L1CommunicationError>
    where
        Tr: Transport + Clone,
        P: alloy::providers::Provider<Tr, Ethereum>,
    {
        let gas_limit = self.estimate_gas_l1_to_l2(l1_to_l2_tx).await.map_err(|_| {
            L1CommunicationError::Custom(
                "Error occurred while estimating gas for L1 -> L2 transaction.",
            )
        })?;

        let tx_base_cost = bridge_hub_contract
            .l2TransactionBaseCost(
                l2_chain_id,
                U256::from(max_fee_per_gas),
                gas_limit,
                gas_per_pubdata_limit,
            )
            .call()
            .await
            .map_err(|_| {
                L1CommunicationError::Custom(
                    "Error occurred while estimating L2 transaction base cost.",
                )
            })?
            ._0;
        Ok(BridgeL2TxGasData {
            gas_limit,
            tx_base_cost,
        })
    }

    /// Deposits specified L1 token to the L2 address.
    ///
    /// ## Parameters
    ///
    /// - `deposit_request`: deposit request which contains deposit params including amount, token to deposit etc.
    /// - `l1_provider`: reference to the L1 provider.
    ///
    /// ## Returns
    ///
    /// L1TransactionReceipt.
    /// Hint: use returned L1 transaction receipt to get corresponding L2 transaction and wait for its receipt
    /// E.g.: deposit_l1_receipt.get_l2_tx()?.with_required_confirmations(1).with_timeout(Some(std::time::Duration::from_secs(60 * 5))).get_receipt()
    async fn deposit<Tr, P>(
        &self,
        deposit_request: &DepositRequest,
        l1_provider: &P,
    ) -> Result<L1TransactionReceipt<T>, L1CommunicationError>
    where
        Tr: Transport + Clone,
        P: alloy::providers::Provider<Tr, Ethereum>,
    {
        let l2_chain_id = U256::from(self.get_chain_id().await.map_err(|_| {
            L1CommunicationError::Custom("Error occurred while fetching L2 chain id.")
        })?);
        let sender = self.wallet().default_signer_address();
        let receiver = deposit_request.receiver.unwrap_or(sender);
        // fees adjustment is taken from the JS SDK:
        // https://github.com/zksync-sdk/zksync-ethers/blob/64763688d1bb5cee4a4c220c3841b803c74b0d05/src/adapters.ts#L2069
        let max_priority_fee_per_gas =
            l1_provider
                .get_max_priority_fee_per_gas()
                .await
                .map_err(|_| {
                    L1CommunicationError::Custom(
                        "Error occurred while fetching L1 max_priority_fee_per_gas.",
                    )
                })?;
        let base_l1_fees_data = l1_provider
            .estimate_eip1559_fees(Some(|base_fee_per_gas, _| Eip1559Estimation {
                max_fee_per_gas: base_fee_per_gas * 3 / 2,
                max_priority_fee_per_gas: 0,
            }))
            .await
            .map_err(|_| {
                L1CommunicationError::Custom("Error occurred while estimating L1 base fees.")
            })?;
        let max_fee_per_gas = base_l1_fees_data.max_fee_per_gas + max_priority_fee_per_gas;

        let bridge_hub_contract_address = self
            .get_bridgehub_contract()
            .await
            .map_err(|_| {
                L1CommunicationError::Custom(
                    "Error occurred while fetching the bridge hub contract address.",
                )
            })?
            .unwrap();
        let bridge_hub_contract = Bridgehub::new(bridge_hub_contract_address, l1_provider);

        if deposit_request.token != ETHER_L1_ADDRESS {
            let (l1_bridge_address, l2_bridge_address) = self
                .get_bridge_addresses_for_deposit(deposit_request, l2_chain_id, l1_provider)
                .await?;

            let erc20_contract = ERC20::new(deposit_request.token, l1_provider);
            let token_data = encode_token_data_for_bridge(&erc20_contract)
                .await
                .map_err(|_| {
                    L1CommunicationError::Custom("Error while encoding ERC20 token data.")
                })?;
            let l2_finalize_deposit_calldata = encode_finalize_deposit_calldata(
                sender,
                receiver,
                deposit_request.token,
                deposit_request.amount,
                token_data,
            );

            let l2_tx_fee = self
                .get_bridge_l2_tx_gas_data(
                    &bridge_hub_contract,
                    TransactionRequest::default()
                        .with_from(apply_l1_to_l2_alias(l1_bridge_address))
                        .with_to(l2_bridge_address)
                        .with_gas_per_pubdata(deposit_request.gas_per_pubdata_limit)
                        .with_input(l2_finalize_deposit_calldata),
                    l2_chain_id,
                    U256::from(max_fee_per_gas),
                    deposit_request.gas_per_pubdata_limit,
                )
                .await?;

            let token_allowance = erc20_contract
                .allowance(sender, l1_bridge_address)
                .call()
                .await
                .map_err(|_| {
                    L1CommunicationError::Custom(
                        "Error occurred while fetching token allowance for the bridge.",
                    )
                })?
                ._0;

            let allowance_deficit = deposit_request.amount - token_allowance;
            if allowance_deficit > U256::from(0) {
                let approve_tx_builder = erc20_contract
                    .approve(l1_bridge_address, allowance_deficit)
                    .from(sender);

                let approve_tx_gas_limit =
                    get_gas_limit_for_l1_tx(&approve_tx_builder, l1_provider).await?;

                approve_tx_builder
                    .max_fee_per_gas(max_fee_per_gas)
                    .max_priority_fee_per_gas(max_priority_fee_per_gas)
                    .gas(approve_tx_gas_limit)
                    .send()
                    .await
                    .map_err(|_| {
                        L1CommunicationError::Custom(
                            "Error occurred while approving tokens for the bridge address",
                        )
                    })?
                    .watch()
                    .await
                    .map_err(|_| {
                        L1CommunicationError::Custom(
                                 "Error occurred while approving tokens for the bridge address. Approve transaction has failed.",
                    )
                    })?;
            }

            let bridge_calldata = encode_deposit_token_calldata(
                deposit_request.token,
                deposit_request.amount,
                receiver,
            );
            let l2_tx_request_builder = bridge_hub_contract
                .requestL2TransactionTwoBridges(L2TransactionRequestTwoBridges {
                    chainId: l2_chain_id,
                    mintValue: l2_tx_fee.tx_base_cost,
                    l2Value: U256::from(0),
                    l2GasLimit: l2_tx_fee.gas_limit,
                    l2GasPerPubdataByteLimit: deposit_request.gas_per_pubdata_limit,
                    refundRecipient: sender,
                    secondBridgeAddress: l1_bridge_address,
                    secondBridgeValue: U256::from(0),
                    secondBridgeCalldata: bridge_calldata,
                })
                .from(sender)
                .value(l2_tx_fee.tx_base_cost);
            let l1_gas_limit = get_gas_limit_for_l1_tx(&l2_tx_request_builder, l1_provider).await?;
            let l1_tx_receipt = l2_tx_request_builder
                .max_fee_per_gas(max_fee_per_gas)
                .max_priority_fee_per_gas(max_priority_fee_per_gas)
                .gas(l1_gas_limit)
                .send()
                .await
                .map_err(|_| {
                    L1CommunicationError::Custom(
                        "Error occurred while sending the L1 -> L2 deposit transaction.",
                    )
                })?
                .get_receipt()
                .await
                .map_err(|_| {
                    L1CommunicationError::Custom(
                        "Error occurred while sending the L1 -> L2 deposit transaction receipt.",
                    )
                })?;

            return Ok(L1TransactionReceipt::new(
                l1_tx_receipt,
                self.root().clone(),
            ));
        }

        let l2_tx_fee = self
            .get_bridge_l2_tx_gas_data(
                &bridge_hub_contract,
                TransactionRequest::default()
                    .with_from(sender)
                    .with_to(receiver)
                    .with_value(deposit_request.amount)
                    .with_gas_per_pubdata(deposit_request.gas_per_pubdata_limit)
                    .with_input(Bytes::from("0x")),
                l2_chain_id,
                U256::from(max_fee_per_gas),
                deposit_request.gas_per_pubdata_limit,
            )
            .await?;

        let l1_value = l2_tx_fee.tx_base_cost + deposit_request.amount;
        let l2_tx_request_builder = bridge_hub_contract
            .requestL2TransactionDirect(L2TransactionRequestDirect {
                chainId: l2_chain_id,
                mintValue: l1_value,
                l2Contract: receiver,
                l2Value: deposit_request.amount,
                l2Calldata: Bytes::from_str("0x").unwrap(),
                l2GasLimit: l2_tx_fee.gas_limit,
                l2GasPerPubdataByteLimit: deposit_request.gas_per_pubdata_limit,
                factoryDeps: vec![],
                refundRecipient: sender,
            })
            .value(l1_value);
        let l1_gas_limit = get_gas_limit_for_l1_tx(&l2_tx_request_builder, l1_provider).await?;
        let l1_tx_receipt = l2_tx_request_builder
            .max_fee_per_gas(max_fee_per_gas)
            .max_priority_fee_per_gas(max_priority_fee_per_gas)
            .gas(l1_gas_limit)
            .send()
            .await
            .map_err(|_| {
                L1CommunicationError::Custom(
                    "Error occurred while sending the L1 -> L2 deposit transaction.",
                )
            })?
            .get_receipt()
            .await
            .map_err(|_| {
                L1CommunicationError::Custom(
                    "Error occurred while sending the L1 -> L2 deposit transaction receipt.",
                )
            })?;

        Ok(L1TransactionReceipt::new(
            l1_tx_receipt,
            self.root().clone(),
        ))
    }
}

impl<P, T> ZksyncProviderWithWallet<T> for P
where
    T: Transport + Clone,
    P: WalletProvider<Zksync> + Provider<T, Zksync>,
{
}

impl<P, T> ZksyncProvider<T> for P
where
    T: Transport + Clone,
    P: Provider<T, Zksync>,
{
}

impl RecommendedFillers for Zksync {
    type RecommendedFillers = JoinFill<Eip712FeeFiller, JoinFill<NonceFiller, ChainIdFiller>>;

    fn recommended_fillers() -> Self::RecommendedFillers {
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
                let received_gas_estimation =
                    provider.estimate_gas_l1_to_l2(tx_request).await.unwrap();
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
