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
    provider::ZksyncProvider,
    wallet::ZksyncWallet,
};

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Spin up a local era-test-node node.
    // Ensure `era_test_node` is available in $PATH.
    let era_test_node = EraTestNode::new().try_spawn()?;

    // Set up signer from the first default era-test-node account (Alice).
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
        .with_value(U256::from(100))
        .with_gas_per_pubdata(U256::from(1_000_000));

    // Send the transaction and wait for inclusion.
    let receipt = provider.send_transaction(tx).await?.get_receipt().await?;
    println!("Got receipt: {receipt:#?}");

    // Call `zks` namespace RPC.
    let l1_chain_id = provider.get_l1_chain_id().await?;
    println!("L1 chain ID is: {l1_chain_id}");

    // Manually deploy contract.
    let bytecode = hex::decode("0000008003000039000000400030043f0000000100200190000000120000c13d000000000201001900000009002001980000001a0000613d000000000101043b0000000a011001970000000b0010009c0000001a0000c13d0000000001000416000000000001004b0000001a0000c13d0000002a01000039000000800010043f0000000c010000410000001d0001042e0000000001000416000000000001004b0000001a0000c13d00000020010000390000010000100443000001200000044300000008010000410000001d0001042e00000000010000190000001e000104300000001c000004320000001d0001042e0000001e000104300000000000000000000000020000000000000000000000000000004000000100000000000000000000000000000000000000000000000000fffffffc000000000000000000000000ffffffff0000000000000000000000000000000000000000000000000000000026121ff000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002000000080000000000000000000000000000000000000000000000000000000000000000000000000000000007c0a65e1f13adb4dd4cc7c561a9505c55567c29e53e1576cfddf58b4218e7a9e").unwrap();
    let tx = TransactionRequest::default()
        .with_gas_limit(100_000_000)
        .with_gas_per_pubdata(U256::from(1_000_000)) // Error: server returned an error response: error code 3: Failed to serialize transaction: gas per pub data limit is zero
        .zksync_deploy(bytecode, Vec::new(), Vec::new())?;
    let receipt = provider.send_transaction(tx).await?.get_receipt().await?;
    println!("Got receipt: {receipt:#?}");

    Ok(())
}
