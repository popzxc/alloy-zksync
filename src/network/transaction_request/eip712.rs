use alloy_network::{Network, TransactionBuilder, TransactionBuilderError};
use serde::{Deserialize, Serialize};

use crate::network::Zksync;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Eip712Tx {}

impl TransactionBuilder<Zksync> for Eip712Tx {
    fn chain_id(&self) -> Option<alloy_primitives::ChainId> {
        todo!()
    }

    fn set_chain_id(&mut self, chain_id: alloy_primitives::ChainId) {
        todo!()
    }

    fn nonce(&self) -> Option<u64> {
        todo!()
    }

    fn set_nonce(&mut self, nonce: u64) {
        todo!()
    }

    fn input(&self) -> Option<&alloy_primitives::Bytes> {
        todo!()
    }

    fn set_input<T: Into<alloy_primitives::Bytes>>(&mut self, input: T) {
        todo!()
    }

    fn from(&self) -> Option<alloy_primitives::Address> {
        todo!()
    }

    fn set_from(&mut self, from: alloy_primitives::Address) {
        todo!()
    }

    fn kind(&self) -> Option<alloy_primitives::TxKind> {
        todo!()
    }

    fn clear_kind(&mut self) {
        todo!()
    }

    fn set_kind(&mut self, kind: alloy_primitives::TxKind) {
        todo!()
    }

    fn value(&self) -> Option<alloy_primitives::U256> {
        todo!()
    }

    fn set_value(&mut self, value: alloy_primitives::U256) {
        todo!()
    }

    fn gas_price(&self) -> Option<u128> {
        todo!()
    }

    fn set_gas_price(&mut self, gas_price: u128) {
        todo!()
    }

    fn max_fee_per_gas(&self) -> Option<u128> {
        todo!()
    }

    fn set_max_fee_per_gas(&mut self, max_fee_per_gas: u128) {
        todo!()
    }

    fn max_priority_fee_per_gas(&self) -> Option<u128> {
        todo!()
    }

    fn set_max_priority_fee_per_gas(&mut self, max_priority_fee_per_gas: u128) {
        todo!()
    }

    fn max_fee_per_blob_gas(&self) -> Option<u128> {
        todo!()
    }

    fn set_max_fee_per_blob_gas(&mut self, max_fee_per_blob_gas: u128) {
        todo!()
    }

    fn gas_limit(&self) -> Option<u128> {
        todo!()
    }

    fn set_gas_limit(&mut self, gas_limit: u128) {
        todo!()
    }

    fn access_list(&self) -> Option<&alloy_rpc_types_eth::AccessList> {
        todo!()
    }

    fn set_access_list(&mut self, access_list: alloy_rpc_types_eth::AccessList) {
        todo!()
    }

    fn blob_sidecar(&self) -> Option<&alloy_consensus::BlobTransactionSidecar> {
        todo!()
    }

    fn set_blob_sidecar(&mut self, sidecar: alloy_consensus::BlobTransactionSidecar) {
        todo!()
    }

    fn complete_type(
        &self,
        ty: <Zksync as alloy_network::Network>::TxType,
    ) -> Result<(), Vec<&'static str>> {
        todo!()
    }

    fn can_submit(&self) -> bool {
        todo!()
    }

    fn can_build(&self) -> bool {
        todo!()
    }

    fn output_tx_type(&self) -> <Zksync as alloy_network::Network>::TxType {
        todo!()
    }

    fn output_tx_type_checked(&self) -> Option<<Zksync as alloy_network::Network>::TxType> {
        todo!()
    }

    fn prep_for_submission(&mut self) {
        todo!()
    }

    fn build_unsigned(
        self,
    ) -> alloy_network::BuildResult<<Zksync as alloy_network::Network>::UnsignedTx, Zksync> {
        todo!()
    }

    async fn build<W: alloy_network::NetworkWallet<Zksync>>(
        self,
        wallet: &W,
    ) -> Result<<Zksync as Network>::TxEnvelope, TransactionBuilderError<Zksync>> {
        todo!()
    }
}
