use alloy_network::{Network, TransactionBuilderError};

use super::EraNetwork;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionRequest {
    #[serde(flatten)]
    inner: alloy_rpc_types_eth::transaction::TransactionRequest,
    // eip_712_meta: Eip712Meta,
}

#[derive(Debug, Clone, Default)]
struct Eip712Meta {
    _todo: (),
}

impl From<crate::network::unsigned_tx::TypedTransaction> for TransactionRequest {
    fn from(value: crate::network::unsigned_tx::TypedTransaction) -> Self {
        Self {
            inner: From::from(value.inner),
            // eip_712_meta: todo!(),
        }
    }
}

impl From<crate::network::tx_envelope::TxEnvelope> for TransactionRequest {
    fn from(value: crate::network::tx_envelope::TxEnvelope) -> Self {
        Self {
            inner: From::from(value.inner),
            // eip_712_meta: todo!(),
        }
    }
}

impl alloy_network::TransactionBuilder<EraNetwork> for TransactionRequest {
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

    fn blob_sidecar(&self) -> Option<&alloy_rpc_types_eth::BlobTransactionSidecar> {
        todo!()
    }

    fn set_blob_sidecar(&mut self, sidecar: alloy_rpc_types_eth::BlobTransactionSidecar) {
        todo!()
    }

    fn complete_type(&self, ty: <EraNetwork as Network>::TxType) -> Result<(), Vec<&'static str>> {
        todo!()
    }

    fn can_submit(&self) -> bool {
        todo!()
    }

    fn can_build(&self) -> bool {
        todo!()
    }

    fn output_tx_type(&self) -> <EraNetwork as Network>::TxType {
        todo!()
    }

    fn output_tx_type_checked(&self) -> Option<<EraNetwork as Network>::TxType> {
        todo!()
    }

    fn prep_for_submission(&mut self) {
        todo!()
    }

    fn build_unsigned(
        self,
    ) -> alloy_network::BuildResult<<EraNetwork as Network>::UnsignedTx, EraNetwork> {
        todo!()
    }

    async fn build<W: alloy_network::NetworkWallet<EraNetwork>>(
        self,
        wallet: &W,
    ) -> Result<<EraNetwork as Network>::TxEnvelope, TransactionBuilderError<EraNetwork>> {
        todo!()
    }
}
