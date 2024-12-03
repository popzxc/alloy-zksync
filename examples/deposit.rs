use alloy::primitives::address;
use alloy::providers::{Provider, ProviderBuilder};
use alloy::{
    network::EthereumWallet,
    network::TransactionBuilder,
    primitives::{Bytes, U256},
    rpc::types::TransactionRequest as L1TransactionRequest,
    signers::local::PrivateKeySigner,
};
use alloy_zksync::provider::ZksyncProvider;
use alloy_zksync::{
    network::transaction_request::TransactionRequest, provider::zksync_provider,
    wallet::ZksyncWallet,
};
use anyhow::Result;
use std::str::FromStr;

use alloy_zksync::contracts::l1::bridge_hub::{
    encode_request_l2_tx_direct_calldata, Bridgehub, L2TransactionRequestDirectParams,
};

pub const RECOMMENDED_DEPOSIT_L1_GAS_LIMIT: u64 = 10000000;
pub const RECOMMENDED_DEPOSIT_L2_GAS_LIMIT: u64 = 10000000;
pub const REQUIRED_L1_TO_L2_GAS_PER_PUBDATA_LIMIT: u64 = 800;
pub const MAX_FEE_PER_GAS_L1: u128 = 1000000001;
pub const MAX_PRIORITY_FEE_PER_GAS_L1: u128 = 1000000000;

#[tokio::main]
async fn main() -> Result<()> {
    let l1_rpc_url = "http://127.0.0.1:8545".parse()?;
    let l2_rpc_url = "http://127.0.0.1:3050".parse()?;
    let signer: PrivateKeySigner =
        "0x7726827caac94a7f9e1b160f7ea819f172f7b6f9d2a97f992c38edeab82d4110"
            .parse()
            .expect("should parse private key");
    let wallet = EthereumWallet::from(signer.clone());

    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(l1_rpc_url);
    let zksync_wallet: ZksyncWallet = ZksyncWallet::from(signer.clone());
    let zksync_provider = zksync_provider()
        .with_recommended_fillers()
        .wallet(zksync_wallet)
        .on_http(l2_rpc_url);

    let l2_chain_id = zksync_provider.get_chain_id().await?;
    let sender = address!("36615Cf349d7F6344891B1e7CA7C72883F5dc049");
    let receiver = address!("d754Ff5e8a6f257E162F72578A4bB0493c0681d8");
    let deposit_value = U256::from(70000000000000_u64);
    let gas_per_pubdata_limit = U256::from(REQUIRED_L1_TO_L2_GAS_PER_PUBDATA_LIMIT);

    let estimate_l1_to_l2_tx = TransactionRequest::default()
        .with_from(sender)
        .with_to(receiver)
        .with_value(deposit_value)
        .with_gas_per_pubdata(gas_per_pubdata_limit)
        .with_input(Bytes::from("0x"));

    let estimate_gas_l1_to_l2 = zksync_provider
        .estimate_gas_l1_to_l2(estimate_l1_to_l2_tx)
        .await?;

    let l1_gas_price = provider.get_gas_price().await?;
    let bridge_hub_contract_address = zksync_provider.get_bridgehub_contract().await?.unwrap();
    let bridge_hub_contract = Bridgehub::new(bridge_hub_contract_address, &provider);

    let l2_base_cost = bridge_hub_contract
        .l2TransactionBaseCost(
            U256::from(l2_chain_id),
            U256::from(l1_gas_price),
            estimate_gas_l1_to_l2,
            gas_per_pubdata_limit,
        )
        .call()
        .await?
        ._0;

    let l1_value = l2_base_cost + deposit_value;
    let l2_tx_request_params = L2TransactionRequestDirectParams {
        chain_id: U256::from(l2_chain_id),
        mint_value: l1_value,
        l2_contract: receiver,
        l2_value: deposit_value,
        l2_calldata: Bytes::from_str("0x").unwrap(),
        l2_gas_limit: estimate_gas_l1_to_l2,
        l2_gas_per_pubdata_byte_limit: gas_per_pubdata_limit,
        factory_deps: vec![],
        refund_recipient: sender,
    };

    let deposit_input = encode_request_l2_tx_direct_calldata(l2_tx_request_params);

    let tx = L1TransactionRequest::default()
        .from(sender)
        .to(bridge_hub_contract_address)
        .input(deposit_input.into())
        .gas_limit(RECOMMENDED_DEPOSIT_L1_GAS_LIMIT)
        .max_fee_per_gas(MAX_FEE_PER_GAS_L1)
        .max_priority_fee_per_gas(MAX_PRIORITY_FEE_PER_GAS_L1)
        .value(l1_value);
    let receipt = provider.send_transaction(tx).await?.get_receipt().await?;
    println!("Tx receipt {:#?}", receipt);
    Ok(())
}
