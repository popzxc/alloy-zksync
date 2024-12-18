use alloy::{
    dyn_abi::DynSolValue,
    primitives::{Address, Bytes, U256},
};
alloy::sol! {
    #[sol(rpc)]
    contract L1Bridge {
        function l2BridgeAddress(uint256 _chainId) external view returns (address);
    }
}

pub(crate) fn encode_deposit_token_calldata(
    token: Address,
    amount: U256,
    receiver: Address,
) -> Bytes {
    Bytes::from(
        DynSolValue::Tuple(vec![
            DynSolValue::Address(token),
            DynSolValue::Uint(amount, 256),
            DynSolValue::Address(receiver),
        ])
        .abi_encode_params(),
    )
}
