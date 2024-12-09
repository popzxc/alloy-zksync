use alloy::{
    network::EthereumWallet,
    primitives::{address, U256},
    providers::ProviderBuilder,
    signers::local::PrivateKeySigner,
};
use alloy_zksync::{
    l1_provider::{zksync_provider, ZksyncProvider, ZksyncProviderWithWallet, ETHER_L1_ADDRESS},
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
    let l1_token_address = ETHER_L1_ADDRESS;
    let l1_tx_receipt = zksync_provider
        .deposit(l1_token_address, receiver, deposit_amount, &l1_provider)
        .await
        .unwrap();

    let l2_tx_receipt = zksync_provider
        .wait_for_l1_tx(l1_tx_receipt, None, None)
        .await?;

    println!("L2 deposit transaction receipt: {:#?}", l2_tx_receipt);
    Ok(())
}
