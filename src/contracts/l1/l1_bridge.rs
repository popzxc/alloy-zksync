use alloy::{
    dyn_abi::DynSolValue,
    primitives::{Address, Bytes, U256},
};

alloy::sol! {
     /// L1Bridge contract for interacting with Layer 2 bridges.
    #[sol(rpc)]
    contract L1Bridge {
        /// Retrieves the address of the L2 bridge for a given chain ID.
        ///
        /// # Arguments
        ///
        /// * `_chainId` - The chain ID.
        ///
        /// # Returns
        ///
        /// The address of the L2 bridge.
        function l2BridgeAddress(uint256 _chainId) external view returns (address);
    }
}

/// Encodes the calldata for depositing a token.
///
/// This function encodes the token address, amount, and receiver address into a `Bytes` object
/// for use in deposit operations.
///
/// # Arguments
///
/// * `token` - The address of the token to deposit.
/// * `amount` - The amount of the token to deposit.
/// * `receiver` - The address of the receiver.
///
/// # Returns
///
/// The encoded calldata as `Bytes`.
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
