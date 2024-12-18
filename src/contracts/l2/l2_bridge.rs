use alloy::{
    primitives::{Address, Bytes, U256},
    sol_types::SolCall,
};
use L2Bridge::finalizeDepositCall;

alloy::sol! {
    #[sol(rpc)]
    contract L2Bridge {
        function finalizeDeposit(
            address _l1Sender,
            address _l2Receiver,
            address _l1Token,
            uint256 _amount,
            bytes calldata _data
        );
    }
}

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
