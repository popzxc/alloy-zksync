use alloy::{
    contract::Error,
    dyn_abi::DynSolValue,
    network::Network,
    primitives::{Bytes, U256},
    transports::Transport,
};
use ERC20::ERC20Instance;

alloy::sol! {
    #[sol(rpc)]
    contract ERC20 {
        function allowance(address owner, address spender) external view returns (uint256);
        function approve(address spender, uint256 value) external returns (bool);

        function name() public view virtual returns (string memory);
        function symbol() public view virtual returns (string memory);
        function decimals() public view virtual returns (uint8);
    }
}

pub(crate) async fn encode_token_data_for_bridge<T, P, N>(
    erc20_contract: &ERC20Instance<T, P, N>,
) -> Result<Bytes, Error>
where
    T: Transport + Clone,
    P: alloy::providers::Provider<T, N>,
    N: Network,
{
    let erc20_name = erc20_contract.name().call().await?._0;
    let erc20_symbol = erc20_contract.symbol().call().await?._0;
    let erc20_decimals = erc20_contract.decimals().call().await?._0;

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
