//! ZKsync-specific utilities related to ERC20 contracts.

use ERC20::ERC20Instance;
use alloy::network::Ethereum;
use alloy::{
    contract::Error,
    dyn_abi::DynSolValue,
    primitives::{Bytes, U256},
};

alloy::sol! {
    /// ABI for an ERC20 contract.
    #[sol(rpc)]
    contract ERC20 {
        function allowance(address owner, address spender) external view returns (uint256);
        function approve(address spender, uint256 value) external returns (bool);

        function name() public view virtual returns (string memory);
        function symbol() public view virtual returns (string memory);
        function decimals() public view virtual returns (uint8);
    }
}

/// Encodes the token data for bridging an ERC20 token.
///
/// This function retrieves the name, symbol, and decimals of the ERC20 token
/// and encodes them into a `Bytes` object for use in bridging operations.
///
/// # Arguments
///
/// * `erc20_contract` - An instance of the ERC20 contract.
///
/// # Returns
///
/// A `Result` containing the encoded token data as `Bytes` or an `Error`.
/// ```
pub(crate) async fn encode_token_data_for_bridge<P>(
    erc20_contract: &ERC20Instance<P>,
) -> Result<Bytes, Error>
where
    P: alloy::providers::Provider<Ethereum>,
{
    let erc20_name = erc20_contract.name().call().await?;
    let erc20_symbol = erc20_contract.symbol().call().await?;
    let erc20_decimals = erc20_contract.decimals().call().await?;

    let token_data = Bytes::from(
        DynSolValue::Tuple(vec![
            DynSolValue::Bytes(DynSolValue::String(erc20_name).abi_encode()),
            DynSolValue::Bytes(DynSolValue::String(erc20_symbol).abi_encode()),
            DynSolValue::Bytes(DynSolValue::Uint(U256::from(erc20_decimals), 256).abi_encode()),
        ])
        .abi_encode_params(),
    );

    Ok(token_data)
}
