use alloy::{
    primitives::{Address, Bytes, B256},
    sol_types::SolCall,
};

/// The address of the contract deployer.
pub const CONTRACT_DEPLOYER_ADDRESS: Address = Address::new([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x80, 0x06,
]);

alloy::sol! {
    /// Function to create a contract.
    function create(bytes32 salt, bytes32 bytecodeHash, bytes memory constructorInput);

    /// Function to create a contract using create2.
    function create2(bytes32 salt, bytes32 bytecodeHash, bytes memory constructorInput);

    /// Event emitted when a contract is deployed.
    event ContractDeployed(
        address indexed deployerAddress,
        bytes32 indexed bytecodeHash,
        address indexed contractAddress
    );
}

/// Encodes the calldata for creating a contract.
///
/// This function encodes the bytecode hash and constructor input into a `Bytes` object
/// for use in contract creation operations using ContractDeployer contract.
///
/// # Arguments
///
/// * `bytecode_hash` - The hash of the contract bytecode.
/// * `constructor_input` - The constructor input data.
///
/// # Returns
///
/// The encoded calldata as `Bytes`.
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

/// Encodes the calldata for creating a contract using create2.
///
/// This function encodes the salt, bytecode hash, and constructor input into a `Bytes` object
/// for use in contract creation operations using ContractDeployer contract.
///
/// # Arguments
///
/// * `salt` - The salt value.
/// * `bytecode_hash` - The hash of the contract bytecode.
/// * `constructor_input` - The constructor input data.
///
/// # Returns
///
/// The encoded calldata as `Bytes`.
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
