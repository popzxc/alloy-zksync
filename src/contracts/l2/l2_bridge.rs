use alloy::{
    primitives::{Address, Bytes, U256},
    sol_types::SolCall,
};
use L2Bridge::finalizeDepositCall;

alloy::sol! {
    /// L2Bridge contract for finalizing deposits from Layer 1.
    #[sol(rpc)]
    contract L2Bridge {
        /// Finalizes a deposit from Layer 1 to Layer 2.
        ///
        /// # Arguments
        ///
        /// * `_l1Sender` - The address of the sender on Layer 1.
        /// * `_l2Receiver` - The address of the receiver on Layer 2.
        /// * `_l1Token` - The address of the token on Layer 1.
        /// * `_amount` - The amount of the token to deposit.
        /// * `_data` - Encoded deposit token data.
        function finalizeDeposit(
            address _l1Sender,
            address _l2Receiver,
            address _l1Token,
            uint256 _amount,
            bytes calldata _data
        );
    }
}

/// Encodes the calldata for finalizing a deposit.
///
/// This function encodes the sender address, receiver address, Layer 1 token address,
/// amount, and token data into a `Bytes` object for use in deposit finalization operations.
///
/// # Arguments
///
/// * `sender` - The address of the sender on Layer 1.
/// * `receiver` - The address of the receiver on Layer 2.
/// * `l1_token_address` - The address of the token on Layer 1.
/// * `amount` - The amount of the token to deposit.
/// * `token_data` - Encoded deposit token data.
///
/// # Returns
///
/// The encoded calldata as `Bytes`.
pub(crate) fn encode_finalize_deposit_calldata(
    sender: Address,
    receiver: Address,
    l1_token_address: Address,
    amount: U256,
    token_data: Bytes,
) -> Bytes {
    let call = finalizeDepositCall {
        _l1Sender: sender,
        _l2Receiver: receiver,
        _l1Token: l1_token_address,
        _amount: amount,
        _data: token_data,
    };

    call.abi_encode().into()
}
