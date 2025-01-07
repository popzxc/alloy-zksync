use alloy::{
    network::EthereumWallet,
    primitives::{address, U256},
    providers::{Provider, ProviderBuilder},
    signers::local::PrivateKeySigner,
    sol,
};
use alloy_zksync::{
    provider::{zksync_provider, DepositRequest, ZksyncProviderWithWallet},
    wallet::ZksyncWallet,
};
use anyhow::Result;

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    ERC20Example,
    // ERC20 example taken from the alloy-rs repo:
    // https://github.com/alloy-rs/examples/blob/main/examples/transactions/examples/artifacts/ERC20Example.json
    "examples/artifacts/ERC20Example.json"
);

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

    let l1_gas_price = l1_provider.get_gas_price().await?;
    // Deploy a test ERC20 token to be used for the deposit.
    let erc20_token_address = ERC20Example::deploy_builder(&l1_provider)
        .gas_price(l1_gas_price)
        .deploy()
        .await?;
    println!("L1 ERC20 token address: {}", erc20_token_address);

    let zksync_wallet: ZksyncWallet = ZksyncWallet::from(signer.clone());
    let zksync_provider = zksync_provider()
        .with_recommended_fillers()
        .wallet(zksync_wallet)
        .on_http(l2_rpc_url);

    // use another test rich wallet as a receiver
    // https://github.com/matter-labs/local-setup/blob/main/rich-wallets.json
    let receiver = address!("a61464658AfeAf65CccaaFD3a512b69A83B77618");
    // 0.00007 tokens
    let deposit_amount = U256::from(70000000000000_u64);
    let deposit_l1_receipt = zksync_provider
        .deposit(
            &DepositRequest::new(deposit_amount)
                .with_receiver(receiver)
                .with_token(erc20_token_address),
            // use with_bridge_address to specify custom bridge address for the deposit
            //.with_bridge_address(address!("785185bbac3a09d447c679cf3420b206ea90be88")),
            // disable tokens auto approval if you plan to manage tokens allowance manually
            //.with_auto_approval(false),
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
