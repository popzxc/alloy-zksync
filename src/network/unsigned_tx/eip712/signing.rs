use super::{utils::hash_bytecode, TxEip712};
use alloy::primitives::{Address, FixedBytes, U256};
use alloy::{
    sol,
    sol_types::{eip712_domain, Eip712Domain, SolStruct},
};

impl TxEip712 {
    fn as_sol_tx(&self) -> Transaction {
        let paymaster = self
            .eip712_meta
            .paymaster_params
            .as_ref()
            .map(|p| p.paymaster)
            .unwrap_or(Address::ZERO);
        let paymaster_input = self
            .eip712_meta
            .paymaster_params
            .as_ref()
            .map(|p| p.paymaster_input.clone())
            .unwrap_or_default();
        let mut factory_deps_hashes = Vec::with_capacity(self.eip712_meta.factory_deps.len() * 32);
        for dep in &self.eip712_meta.factory_deps {
            // TODO: Unwrap should be safe here?
            let hash = hash_bytecode(dep).unwrap();
            factory_deps_hashes.push(hash.into());
        }

        Transaction {
            txType: U256::from(self.tx_type() as u8),
            from: address_to_u256(&self.from),
            to: address_to_u256(&self.to),
            gasLimit: U256::from(self.gas_limit),
            gasPerPubdataByteLimit: self.eip712_meta.gas_per_pubdata,
            maxFeePerGas: U256::from(self.max_fee_per_gas),
            maxPriorityFeePerGas: U256::from(self.max_priority_fee_per_gas),
            paymaster: address_to_u256(&paymaster),
            nonce: self.nonce,
            value: self.value,
            data: self.input.clone(),
            factoryDeps: factory_deps_hashes,
            paymasterInput: paymaster_input,
        }
    }

    pub(super) fn domain_hash(&self) -> FixedBytes<32> {
        let domain = zksync_eip712_domain(self.chain_id);
        domain.hash_struct()
    }

    pub(super) fn eip712_hash_struct(&self) -> FixedBytes<32> {
        self.as_sol_tx().eip712_hash_struct()
    }
}

fn address_to_u256(address: &Address) -> U256 {
    let mut bytes = [0u8; 32];
    bytes[12..].copy_from_slice(address.as_slice());
    U256::from_be_slice(&bytes)
}

fn zksync_eip712_domain(chain_id: u64) -> Eip712Domain {
    eip712_domain! {
        name: "zkSync",
        version: "2",
        chain_id: chain_id,
    }
}

sol! {
    struct Transaction {
        uint256 txType;
        uint256 from;
        uint256 to;
        uint256 gasLimit;
        uint256 gasPerPubdataByteLimit;
        uint256 maxFeePerGas;
        uint256 maxPriorityFeePerGas;
        uint256 paymaster;
        uint256 nonce;
        uint256 value;
        bytes data;
        bytes32[] factoryDeps;
        bytes paymasterInput;
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_address_to_u256() {
        let test_vector = [
            (Address::ZERO, U256::ZERO),
            (
                Address::repeat_byte(0x01),
                U256::from_str(
                    "0x0000000000000000000000000101010101010101010101010101010101010101",
                )
                .unwrap(),
            ),
        ];
        for (address, expected) in test_vector.iter() {
            assert_eq!(address_to_u256(address), *expected);
        }
    }
}
