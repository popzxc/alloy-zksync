//! Implementation of the deposit logic.

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
    provider::{
        l1_transaction_receipt::L1TransactionReceipt, L1CommunicationError, ZksyncProvider,
    },
    utils::{apply_l1_to_l2_alias, ETHER_L1_ADDRESS},
};
use alloy::{
    eips::eip1559::Eip1559Estimation,
    network::{Ethereum, NetworkWallet, TransactionBuilder},
    primitives::{Address, Bytes, U256},
    providers::{utils::Eip1559Estimator, WalletProvider},
    rpc::types::eth::TransactionRequest as L1TransactionRequest,
};
use std::str::FromStr;

pub const REQUIRED_L1_TO_L2_GAS_PER_PUBDATA_LIMIT: u64 = 800;

/// Type for deposit request.
/// This type only stores the required information for the deposit, while the deposit itself
/// is performed via [`DepositExecutor`].
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
    /// Enable or disable automatic submission of ERC20 approval transactions
    /// if the allowance is not sufficient.
    pub auto_approval: bool,
}

impl DepositRequest {
    /// Initiates a new deposit request.
    pub fn new(amount: U256) -> Self {
        Self {
            amount,
            receiver: None,
            token: ETHER_L1_ADDRESS,
            bridge_address: None,
            gas_per_pubdata_limit: U256::from(REQUIRED_L1_TO_L2_GAS_PER_PUBDATA_LIMIT),
            auto_approval: true,
        }
    }

    /// Returns the amount to deposit.
    pub fn amount(&self) -> &U256 {
        &self.amount
    }

    /// Sets the receiver for the deposit.
    pub fn with_receiver(mut self, address: Address) -> Self {
        self.receiver = Some(address);
        self
    }

    /// Sets the token address for the deposit.
    pub fn with_token(mut self, token: Address) -> Self {
        self.token = token;
        self
    }

    /// Sets the gas per pubdata limit for the transaction.
    pub fn with_gas_per_pubdata_limit(mut self, value: U256) -> Self {
        self.gas_per_pubdata_limit = value;
        self
    }

    /// Sets the bridge address.
    pub fn with_bridge_address(mut self, bridge_address: Address) -> Self {
        self.bridge_address = Some(bridge_address);
        self
    }

    /// Enables or disables auto-approval for ERC20 tokens.
    pub fn with_auto_approval(mut self, auto_approval: bool) -> Self {
        self.auto_approval = auto_approval;
        self
    }
}

#[derive(Clone, Debug, Copy)]
struct FeeParams {
    max_fee_per_gas: u128,
    max_priority_fee_per_gas: u128,
}

#[derive(Clone, Debug, Copy)]
struct BridgeL2TxFeeParams {
    pub gas_limit: U256,
    pub tx_base_cost: U256,
}

#[derive(Clone, Debug, Copy)]
struct BridgeAddresses {
    pub l1_bridge_address: Address,
    pub l2_bridge_address: Address,
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

/// Type that handles deposit logic for various scenarios: deposit ETH, ERC20 etc.
pub struct DepositExecutor<'a, P1, P2>
where
    P1: alloy::providers::Provider<Ethereum>,
    P2: ZksyncProvider + WalletProvider<Zksync> + ?Sized,
{
    l1_provider: &'a P1,
    l2_provider: &'a P2,
    request: &'a DepositRequest,
}

impl<'a, P1, P2> DepositExecutor<'a, P1, P2>
where
    P1: alloy::providers::Provider<Ethereum>,
    P2: ZksyncProvider + WalletProvider<Zksync> + ?Sized,
{
    /// Prepares an executor for a particular deposit request.
    pub fn new(l1_provider: &'a P1, l2_provider: &'a P2, request: &'a DepositRequest) -> Self {
        DepositExecutor {
            l1_provider,
            l2_provider,
            request,
        }
    }

    async fn get_bridge_addresses_for_deposit(
        &self,
        l2_chain_id: U256,
    ) -> Result<(Address, Address), L1CommunicationError> {
        let (l1_bridge_address, l2_bridge_address) = match self.request.bridge_address {
            Some(l1_bridge_address) => {
                let l1_bridge = L1Bridge::new(l1_bridge_address, self.l1_provider);
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
                let bridge_addresses =
                    self.l2_provider.get_bridge_contracts().await.map_err(|_| {
                        L1CommunicationError::Custom(
                            "Error occurred while fetching bridge contracts.",
                        )
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

    async fn get_l1_fee_params(&self) -> Result<FeeParams, L1CommunicationError> {
        let max_priority_fee_per_gas = self
            .l1_provider
            .get_max_priority_fee_per_gas()
            .await
            .map_err(|_| {
                L1CommunicationError::Custom(
                    "Error occurred while fetching L1 max_priority_fee_per_gas.",
                )
            })?;
        // fees adjustment is taken from the JS SDK:
        // https://github.com/zksync-sdk/zksync-ethers/blob/64763688d1bb5cee4a4c220c3841b803c74b0d05/src/adapters.ts#L2069
        let base_l1_fees_data = self
            .l1_provider
            .estimate_eip1559_fees_with(Eip1559Estimator::new(|base_fee_per_gas, _| {
                Eip1559Estimation {
                    max_fee_per_gas: base_fee_per_gas * 3 / 2,
                    max_priority_fee_per_gas: 0,
                }
            }))
            .await
            .map_err(|_| {
                L1CommunicationError::Custom("Error occurred while estimating L1 base fees.")
            })?;
        let max_fee_per_gas = base_l1_fees_data.max_fee_per_gas + max_priority_fee_per_gas;

        Ok(FeeParams {
            max_fee_per_gas,
            max_priority_fee_per_gas,
        })
    }

    async fn get_l1_tx_gas_limit(
        &self,
        tx_request: &L1TransactionRequest,
    ) -> Result<u64, L1CommunicationError> {
        let l1_tx_gas_estimation = self
            .l1_provider
            .estimate_gas(tx_request.clone())
            .await
            .map_err(|_| {
                L1CommunicationError::Custom(
                    "Error occurred while estimating gas limit for the L1 transaction.",
                )
            })?;
        let l1_gas_limit = scale_l1_gas_limit(l1_tx_gas_estimation);
        Ok(l1_gas_limit)
    }

    async fn get_bridge_l2_tx_fee_params<P>(
        &self,
        bridge_hub_contract: &Bridgehub::BridgehubInstance<(), &P>,
        l1_to_l2_tx: TransactionRequest,
        l2_chain_id: U256,
        fee_params: &FeeParams,
    ) -> Result<BridgeL2TxFeeParams, L1CommunicationError>
    where
        P: alloy::providers::Provider<Ethereum>,
    {
        let gas_limit = self
            .l2_provider
            .estimate_gas_l1_to_l2(l1_to_l2_tx)
            .await
            .map_err(|_| {
                L1CommunicationError::Custom(
                    "Error occurred while estimating gas for L1 -> L2 transaction.",
                )
            })?;

        let tx_base_cost = bridge_hub_contract
            .l2TransactionBaseCost(
                l2_chain_id,
                U256::from(fee_params.max_fee_per_gas),
                gas_limit,
                self.request.gas_per_pubdata_limit,
            )
            .call()
            .await
            .map_err(|_| {
                L1CommunicationError::Custom(
                    "Error occurred while estimating L2 transaction base cost.",
                )
            })?
            ._0;
        Ok(BridgeL2TxFeeParams {
            gas_limit,
            tx_base_cost,
        })
    }

    async fn get_l1_deposit_tx(
        &self,
        sender: Address,
        receiver: Address,
        bridge_addresses: Option<BridgeAddresses>,
        l2_chain_id: U256,
        fee_params: &FeeParams,
    ) -> Result<L1TransactionRequest, L1CommunicationError> {
        let bridge_hub_contract_address = self
            .l2_provider
            .get_bridgehub_contract()
            .await
            .map_err(|_| {
                L1CommunicationError::Custom(
                    "Error occurred while fetching the bridge hub contract address.",
                )
            })?
            .unwrap();
        let bridge_hub_contract = Bridgehub::new(bridge_hub_contract_address, self.l1_provider);

        let l1_tx_request = if self.request.token == ETHER_L1_ADDRESS {
            let l2_tx_fee = self
                .get_bridge_l2_tx_fee_params(
                    &bridge_hub_contract,
                    TransactionRequest::default()
                        .with_from(sender)
                        .with_to(receiver)
                        .with_value(self.request.amount)
                        .with_gas_per_pubdata(self.request.gas_per_pubdata_limit)
                        .with_input(Bytes::from("0x")),
                    l2_chain_id,
                    fee_params,
                )
                .await?;

            let l1_value = l2_tx_fee.tx_base_cost + self.request.amount;
            bridge_hub_contract
                .requestL2TransactionDirect(L2TransactionRequestDirect {
                    chainId: l2_chain_id,
                    mintValue: l1_value,
                    l2Contract: receiver,
                    l2Value: self.request.amount,
                    l2Calldata: Bytes::from_str("0x").unwrap(),
                    l2GasLimit: l2_tx_fee.gas_limit,
                    l2GasPerPubdataByteLimit: self.request.gas_per_pubdata_limit,
                    factoryDeps: vec![],
                    refundRecipient: sender,
                })
                .value(l1_value)
                .into_transaction_request()
        } else {
            let bridge_addresses = bridge_addresses.unwrap();
            let erc20_contract = ERC20::new(self.request.token, self.l1_provider);
            let token_data = encode_token_data_for_bridge(&erc20_contract)
                .await
                .map_err(|_| {
                    L1CommunicationError::Custom("Error while encoding ERC20 token data.")
                })?;
            let l2_finalize_deposit_calldata = encode_finalize_deposit_calldata(
                sender,
                receiver,
                self.request.token,
                self.request.amount,
                token_data,
            );

            let l2_tx_fee = self
                .get_bridge_l2_tx_fee_params(
                    &bridge_hub_contract,
                    TransactionRequest::default()
                        .with_from(apply_l1_to_l2_alias(bridge_addresses.l1_bridge_address))
                        .with_to(bridge_addresses.l2_bridge_address)
                        .with_gas_per_pubdata(self.request.gas_per_pubdata_limit)
                        .with_input(l2_finalize_deposit_calldata),
                    l2_chain_id,
                    fee_params,
                )
                .await?;

            let bridge_calldata =
                encode_deposit_token_calldata(self.request.token, self.request.amount, receiver);
            bridge_hub_contract
                .requestL2TransactionTwoBridges(L2TransactionRequestTwoBridges {
                    chainId: l2_chain_id,
                    mintValue: l2_tx_fee.tx_base_cost,
                    l2Value: U256::from(0),
                    l2GasLimit: l2_tx_fee.gas_limit,
                    l2GasPerPubdataByteLimit: self.request.gas_per_pubdata_limit,
                    refundRecipient: sender,
                    secondBridgeAddress: bridge_addresses.l1_bridge_address,
                    secondBridgeValue: U256::from(0),
                    secondBridgeCalldata: bridge_calldata,
                })
                .from(sender)
                .value(l2_tx_fee.tx_base_cost)
                .into_transaction_request()
        };
        Ok(l1_tx_request
            .max_fee_per_gas(fee_params.max_fee_per_gas)
            .max_priority_fee_per_gas(fee_params.max_priority_fee_per_gas))
    }

    async fn approve_tokens(
        &self,
        sender: Address,
        bridge_addresses: Option<BridgeAddresses>,
        fee_params: &FeeParams,
    ) -> Result<(), L1CommunicationError> {
        if self.request.token == ETHER_L1_ADDRESS {
            return Ok(());
        }
        let bridge_addresses = bridge_addresses.unwrap();
        let erc20_contract = ERC20::new(self.request.token, self.l1_provider);
        let token_allowance = erc20_contract
            .allowance(sender, bridge_addresses.l1_bridge_address)
            .call()
            .await
            .map_err(|_| {
                L1CommunicationError::Custom(
                    "Error occurred while fetching token allowance for the bridge.",
                )
            })?
            ._0;

        let allowance_deficit = self.request.amount - token_allowance;
        if allowance_deficit > U256::from(0) {
            if !self.request.auto_approval {
                return Err(L1CommunicationError::Custom(
                    "Deposit request auto_approval is disabled and the current token allowance won't cover the deposit. Consider enabling deposit request auto_approval or approving tokens manually before the deposit.",
                ));
            }
            let approve_tx_builder = erc20_contract
                .approve(bridge_addresses.l1_bridge_address, allowance_deficit)
                .from(sender);
            let approve_tx = approve_tx_builder.into_transaction_request();
            let approve_tx_gas_limit = self.get_l1_tx_gas_limit(&approve_tx).await?;

            self
                .l1_provider
                .send_transaction( approve_tx
                .max_fee_per_gas(fee_params.max_fee_per_gas)
                .max_priority_fee_per_gas(fee_params.max_priority_fee_per_gas)
                .gas_limit(approve_tx_gas_limit))
                .await.map_err(|_| {
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
        Ok(())
    }

    async fn submit(
        &self,
        tx_request: &L1TransactionRequest,
    ) -> Result<L1TransactionReceipt, L1CommunicationError> {
        let l1_gas_limit = self.get_l1_tx_gas_limit(tx_request).await?;
        let l1_tx_request = tx_request.clone().with_gas_limit(l1_gas_limit);
        let l1_tx_receipt = self
            .l1_provider
            .send_transaction(l1_tx_request)
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
            self.l2_provider.root().clone(),
        ))
    }

    /// Executes specified deposit request. This will handle:
    /// - Approving tokens if necessary.
    /// - Sending the deposit transaction.
    /// - Returning the [`L1TransactionReceipt`] of the deposit transaction.
    ///
    /// Returned receipt can be converted into a pending L2 transaction and awaited
    /// using [`PendingTransactionBuilder`](https://docs.rs/alloy/latest/alloy/providers/struct.PendingTransactionBuilder.html)
    /// interface.
    ///
    /// ## Returns
    ///
    /// L1TransactionReceipt of the deposit transaction.
    pub async fn execute(&self) -> Result<L1TransactionReceipt, L1CommunicationError> {
        let l2_chain_id = U256::from(self.l2_provider.get_chain_id().await.map_err(|_| {
            L1CommunicationError::Custom("Error occurred while fetching L2 chain id.")
        })?);

        let bridge_addresses = if self.request.token != ETHER_L1_ADDRESS {
            let (l1_bridge_address, l2_bridge_address) =
                self.get_bridge_addresses_for_deposit(l2_chain_id).await?;

            Some(BridgeAddresses {
                l1_bridge_address,
                l2_bridge_address,
            })
        } else {
            None
        };

        let sender = self.l2_provider.wallet().default_signer_address();
        let receiver = self.request.receiver.unwrap_or(sender);

        let l1_fee_params = self.get_l1_fee_params().await?;

        let l1_deposit_tx = self
            .get_l1_deposit_tx(
                sender,
                receiver,
                bridge_addresses,
                l2_chain_id,
                &l1_fee_params,
            )
            .await?;

        self.approve_tokens(sender, bridge_addresses, &l1_fee_params)
            .await?;

        self.submit(&l1_deposit_tx).await
    }
}
