//! Example of using a local wallet to sign and send a transaction.

use alloy::{
    network::TransactionBuilder,
    primitives::{address, U256},
    providers::{Provider, ProviderBuilder},
    signers::local::PrivateKeySigner,
};
use alloy_zksync::{
    network::{transaction_request::TransactionRequest, Zksync},
    node_bindings::EraTestNode,
    wallet::ZksyncWallet,
};

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Spin up a local era-test-node node.
    // Ensure `era_test_node` is available in $PATH.
    let era_test_node = EraTestNode::new().try_spawn()?;

    // Set up signer from the first default era-test-node account (Alice).
    // [RISK WARNING! Writing a private key in the code file is insecure behavior.]
    // The following code is for testing only. Set up signer from private key, be aware of danger.
    // let signer: PrivateKeySigner = "<PRIVATE_KEY>".parse().expect("should parse private key");
    let signer: PrivateKeySigner = era_test_node.keys()[0].clone().into();
    let wallet = ZksyncWallet::from(signer);

    // Create a provider with the wallet.
    let rpc_url = era_test_node.endpoint().parse()?;
    let provider = ProviderBuilder::<_, _, Zksync>::default()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(rpc_url);

    // Build a transaction to send 100 wei from Alice to Vitalik.
    // The `from` field is automatically filled to the first signer's address (Alice).
    let tx = TransactionRequest::default()
        .with_to(address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045"))
        .with_value(U256::from(100));

    // Send the transaction and wait for inclusion.
    let receipt = provider.send_transaction(tx).await?.get_receipt().await?;

    println!("Got receipt: {receipt:#?}");

    Ok(())
}
