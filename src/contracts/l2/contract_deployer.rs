use alloy::{
    primitives::{Address, Bytes, B256},
    sol_types::SolCall,
};

pub const CONTRACT_DEPLOYER_ADDRESS: Address = Address::new([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x80, 0x06,
]);

alloy::sol! {
    function create(bytes32 salt, bytes32 bytecodeHash, bytes memory constructorInput);

    event ContractDeployed(
        address indexed deployerAddress,
        bytes32 indexed bytecodeHash,
        address indexed contractAddress
    );
}

pub(crate) fn encode_create_calldata(
    salt: Option<B256>,
    bytecode_hash: B256,
    constructor_input: Bytes,
) -> Bytes {
    let call = createCall {
        salt: salt.unwrap_or_default(),
        bytecodeHash: bytecode_hash,
        constructorInput: constructor_input,
    };

    call.abi_encode().into()
}
