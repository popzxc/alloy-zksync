//! Example of deploying a contract at runtime from Solidity bytecode to anvil-zksync and
//! interacting with it.
//! Based on the example from the `alloy` book.

use alloy::{network::ReceiptResponse, primitives::U256, providers::Provider as _, sol};
use alloy_zksync::{
    network::transaction_request::TransactionRequest,
    provider::{zksync_provider, ProviderBuilderExt as _},
};
use anyhow::Result;

sol! {
    #[allow(missing_docs)]
    #[sol(rpc)]
    contract Counter {
        uint256 public number;

        function setNumber(uint256 newNumber) public {
            number = newNumber;
        }

        function increment() public {
            number++;
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Create provider connected to the `anvil-zksync`.
    // Note that you can manually spawn and configure the `anvil-zksync`, e.g. if you don't want the instance
    // to be spawned implicitly and/or you don't want it to be attached to a single provider.
    let provider = zksync_provider()
        .with_recommended_fillers()
        // set anvil port to 0 to let it choose a random available port
        .on_anvil_zksync_with_wallet_and_config(|anvil| anvil.port(0_u16));

    // Manually deploy contract.
    let bytecode = hex::decode("0000008003000039000000400030043f0000000100200190000000180000c13d00000060021002700000000f02200197000000040020008c000000330000413d000000000301043b000000e003300270000000110030009c000000270000613d000000120030009c000000200000613d000000130030009c000000330000c13d000000240020008c000000330000413d0000000002000416000000000002004b000000330000c13d0000000401100370000000000101043b000000350000013d0000000001000416000000000001004b000000330000c13d0000002001000039000001000010044300000120000004430000001001000041000000390001042e0000000001000416000000000001004b000000330000c13d000000000100041a000000800010043f0000001601000041000000390001042e0000000001000416000000000001004b000000330000c13d000000000100041a000000010110003a000000350000c13d0000001401000041000000000010043f0000001101000039000000040010043f00000015010000410000003a0001043000000000010000190000003a00010430000000000010041b0000000001000019000000390001042e0000003800000432000000390001042e0000003a00010430000000000000000000000000000000000000000000000000000000000000000000000000ffffffff000000020000000000000000000000000000004000000100000000000000000000000000000000000000000000000000000000000000000000000000d09de08a000000000000000000000000000000000000000000000000000000008381f58a000000000000000000000000000000000000000000000000000000003fb5c1cb4e487b7100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002400000000000000000000000000000000000000000000000000000000000000200000008000000000000000000000000000000000000000000000000000000000000000000000000000000000907744cfcba9c9276da62757037b7ad91caa65b463857bae5ffcd6ceb985e728").unwrap();
    let tx = TransactionRequest::default().with_create_params(bytecode, Vec::new(), Vec::new())?;
    let receipt = provider.send_transaction(tx).await?.get_receipt().await?;

    let contract_address = receipt
        .contract_address()
        .expect("Failed to get contract address");
    let contract = Counter::new(contract_address, &provider);

    println!("Deployed contract at address: {}", contract.address());

    let builder = contract.setNumber(U256::from(42));
    let tx_hash = builder.send().await?.watch().await?;

    println!("Set number to 42: {tx_hash}");

    // Increment the number to 43.
    let builder = contract.increment();
    let tx_hash = builder.send().await?.watch().await?;

    println!("Incremented number: {tx_hash}");

    // Retrieve the number, which should be 43.
    let builder = contract.number();
    let number = builder.call().await?.to_string();

    println!("Retrieved number: {number}");

    Ok(())
}
