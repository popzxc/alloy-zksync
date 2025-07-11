//! Example of using a local wallet to sign and send a transaction.

use alloy::{
    network::TransactionBuilder,
    primitives::{U256, address},
    providers::Provider,
};
use alloy_zksync::{
    network::transaction_request::TransactionRequest,
    provider::{ProviderBuilderExt, zksync_provider},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create a provider with the wallet.
    let provider = zksync_provider()
        .with_recommended_fillers()
        // set anvil port to 0 to let it choose a random available port
        .on_anvil_zksync_with_wallet_and_config(|anvil| anvil.port(0_u16));

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
