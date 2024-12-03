use alloy::{
    primitives::{Address, Bytes, U256},
    sol_types::SolCall,
};

#[derive(Debug)]
pub struct L2TransactionRequestDirectParams {
    pub chain_id: U256,
    pub mint_value: U256,
    pub l2_contract: Address,
    pub l2_value: U256,
    pub l2_calldata: Bytes,
    pub l2_gas_limit: U256,
    pub l2_gas_per_pubdata_byte_limit: U256,
    pub factory_deps: Vec<Bytes>,
    pub refund_recipient: Address,
}

alloy::sol! {
    #[allow(missing_docs)]
    struct L2TransactionRequestDirect {
        uint256 chainId;
        uint256 mintValue;
        address l2Contract;
        uint256 l2Value;
        bytes l2Calldata;
        uint256 l2GasLimit;
        uint256 l2GasPerPubdataByteLimit;
        bytes[] factoryDeps;
        address refundRecipient;
    }

    #[allow(missing_docs)]
    #[sol(rpc)]
    contract Bridgehub {
        function requestL2TransactionDirect(
            L2TransactionRequestDirect memory request
        ) external payable returns (bytes32 canonicalTxHash);

        function l2TransactionBaseCost(
            uint256 _chainId,
            uint256 _gasPrice,
            uint256 _l2GasLimit,
            uint256 _l2GasPerPubdataByteLimit
        ) external view returns (uint256);
    }
}

pub fn encode_request_l2_tx_direct_calldata(
    request_params: L2TransactionRequestDirectParams,
) -> Bytes {
    let request = L2TransactionRequestDirect {
        chainId: request_params.chain_id,
        mintValue: request_params.mint_value,
        l2Contract: request_params.l2_contract,
        l2Value: request_params.l2_value,
        l2Calldata: request_params.l2_calldata,
        l2GasLimit: request_params.l2_gas_limit,
        l2GasPerPubdataByteLimit: request_params.l2_gas_per_pubdata_byte_limit,
        factoryDeps: request_params.factory_deps,
        refundRecipient: request_params.refund_recipient,
    };
    let call = Bridgehub::requestL2TransactionDirectCall { request };
    call.abi_encode().into()
}
