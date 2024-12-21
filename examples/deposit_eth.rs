//! Example of depositing ETH token.

use alloy::{
    network::EthereumWallet,
    primitives::{address, U256},
    providers::ProviderBuilder,
    signers::local::PrivateKeySigner,
};
use alloy_zksync::{
    provider::{zksync_provider, DepositRequest, ZksyncProviderWithWallet},
    wallet::ZksyncWallet,
};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // standard RPC urls for the L1 and L2 local nodes spun up by ZKSync CLI:
    // More general info on the local setup can be found here:
    // https://docs.zksync.io/zksync-era/tooling/local-setup/dockerized-l1-l2-nodes
    // and how to spin it up locally:
    // https://docs.zksync.io/zksync-era/tooling/zksync-cli/running-a-node
    let l1_rpc_url = "http://127.0.0.1:8545".parse()?;
    let l2_rpc_url = "http://127.0.0.1:3050".parse()?;
    // one of the test rich wallets created by the local setup
    // https://github.com/matter-labs/local-setup/blob/main/rich-wallets.json
    let signer: PrivateKeySigner =
        "0x7726827caac94a7f9e1b160f7ea819f172f7b6f9d2a97f992c38edeab82d4110"
            .parse()
            .expect("should parse private key");
    let wallet = EthereumWallet::from(signer.clone());

    let l1_provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(l1_rpc_url);

    let zksync_wallet: ZksyncWallet = ZksyncWallet::from(signer.clone());
    let zksync_provider = zksync_provider()
        .with_recommended_fillers()
        .wallet(zksync_wallet)
        .on_http(l2_rpc_url);

    // use another test rich wallet as a receiver
    // https://github.com/matter-labs/local-setup/blob/main/rich-wallets.json
    let receiver = address!("a61464658AfeAf65CccaaFD3a512b69A83B77618");
    // 0.00007 ETH
    let deposit_amount = U256::from(70000000000000_u64);
    let deposit_l1_receipt = zksync_provider
        .deposit(
            &DepositRequest::new(deposit_amount).with_receiver(receiver),
            &l1_provider,
        )
        .await
        .unwrap();

    let deposit_l2_receipt = deposit_l1_receipt
        .get_l2_tx()?
        .with_required_confirmations(1)
        .with_timeout(Some(std::time::Duration::from_secs(60 * 5)))
        .get_receipt()
        .await?;

    println!("L2 deposit transaction receipt: {:#?}", deposit_l2_receipt);
    Ok(())
}
