//! Helper utilities.

use alloy::{
    hex::FromHex,
    primitives::{Address, U256},
};
use std::str::FromStr;

/// ETH address on L1.
pub const ETHER_L1_ADDRESS: Address = Address::new([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
]);

///  Utility function that converts an L1 address to the L2 address.
pub fn apply_l1_to_l2_alias(l1_address: Address) -> Address {
    let address_modulo: U256 = U256::from(2).pow(U256::from(160));
    let contract_address = U256::from_str(&l1_address.to_string()).unwrap();
    let l1_to_l2_alias_offset =
        U256::from_str("0x1111000000000000000000000000000000001111").unwrap();

    let l2_address = (contract_address + l1_to_l2_alias_offset) % address_modulo;
    Address::from_hex(format!("{l2_address:x}")).unwrap()
}

#[cfg(test)]
mod tests {
    use super::apply_l1_to_l2_alias;
    use alloy::primitives::address;

    #[tokio::test(flavor = "multi_thread")]
    async fn get_main_contract_test() {
        let l2_alias = apply_l1_to_l2_alias(address!("702942B8205E5dEdCD3374E5f4419843adA76Eeb"));
        assert_eq!(
            l2_alias,
            address!("813A42B8205E5DedCd3374e5f4419843ADa77FFC")
        );
    }
}
