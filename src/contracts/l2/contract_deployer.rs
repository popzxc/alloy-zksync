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

    function create2(bytes32 salt, bytes32 bytecodeHash, bytes memory constructorInput);

    event ContractDeployed(
        address indexed deployerAddress,
        bytes32 indexed bytecodeHash,
        address indexed contractAddress
    );
}

pub(crate) fn encode_create_calldata(bytecode_hash: B256, constructor_input: Bytes) -> Bytes {
    // The salt parameter is required as per signature but is not used during create
    // See: https://github.com/matter-labs/era-contracts/blob/main/system-contracts/contracts/interfaces/IContractDeployer.sol#L65
    let call = createCall {
        salt: Default::default(),
        bytecodeHash: bytecode_hash,
        constructorInput: constructor_input,
    };

    call.abi_encode().into()
}

pub(crate) fn encode_create2_calldata(
    salt: B256,
    bytecode_hash: B256,
    constructor_input: Bytes,
) -> Bytes {
    let call = createCall {
        salt,
        bytecodeHash: bytecode_hash,
        constructorInput: constructor_input,
    };

    call.abi_encode().into()
}
