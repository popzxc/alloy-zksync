use alloy_network::{
    Network, TransactionBuilder, TransactionBuilderError, UnbuiltTransactionError,
};

use super::Era;

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

impl TransactionBuilder<Era> for TransactionRequest {
    fn chain_id(&self) -> Option<alloy_primitives::ChainId> {
        self.inner.chain_id
    }

    fn set_chain_id(&mut self, chain_id: alloy_primitives::ChainId) {
        self.inner.set_chain_id(chain_id)
    }

    fn nonce(&self) -> Option<u64> {
        TransactionBuilder::nonce(&self.inner)
    }

    fn set_nonce(&mut self, nonce: u64) {
        self.inner.set_nonce(nonce)
    }

    fn input(&self) -> Option<&alloy_primitives::Bytes> {
        TransactionBuilder::input(&self.inner)
    }

    fn set_input<T: Into<alloy_primitives::Bytes>>(&mut self, input: T) {
        self.inner.set_input(input)
    }

    fn from(&self) -> Option<alloy_primitives::Address> {
        TransactionBuilder::from(&self.inner)
    }

    fn set_from(&mut self, from: alloy_primitives::Address) {
        self.inner.set_from(from)
    }

    fn kind(&self) -> Option<alloy_primitives::TxKind> {
        self.inner.kind()
    }

    fn clear_kind(&mut self) {
        self.inner.clear_kind()
    }

    fn set_kind(&mut self, kind: alloy_primitives::TxKind) {
        self.inner.set_kind(kind)
    }

    fn value(&self) -> Option<alloy_primitives::U256> {
        TransactionBuilder::value(&self.inner)
    }

    fn set_value(&mut self, value: alloy_primitives::U256) {
        self.inner.set_value(value)
    }

    fn gas_price(&self) -> Option<u128> {
        self.inner.gas_price()
    }

    fn set_gas_price(&mut self, gas_price: u128) {
        self.inner.set_gas_price(gas_price)
    }

    fn max_fee_per_gas(&self) -> Option<u128> {
        self.inner.max_fee_per_gas
    }

    fn set_max_fee_per_gas(&mut self, max_fee_per_gas: u128) {
        self.inner.set_max_fee_per_gas(max_fee_per_gas)
    }

    fn max_priority_fee_per_gas(&self) -> Option<u128> {
        TransactionBuilder::max_priority_fee_per_gas(&self.inner)
    }

    fn set_max_priority_fee_per_gas(&mut self, max_priority_fee_per_gas: u128) {
        self.inner
            .set_max_priority_fee_per_gas(max_priority_fee_per_gas)
    }

    fn max_fee_per_blob_gas(&self) -> Option<u128> {
        self.inner.max_fee_per_blob_gas()
    }

    fn set_max_fee_per_blob_gas(&mut self, max_fee_per_blob_gas: u128) {
        self.inner.set_max_fee_per_blob_gas(max_fee_per_blob_gas)
    }

    fn gas_limit(&self) -> Option<u128> {
        TransactionBuilder::gas_limit(&self.inner)
    }

    fn set_gas_limit(&mut self, gas_limit: u128) {
        self.inner.set_gas_limit(gas_limit)
    }

    fn access_list(&self) -> Option<&alloy_rpc_types_eth::AccessList> {
        TransactionBuilder::access_list(&self.inner)
    }

    fn set_access_list(&mut self, access_list: alloy_rpc_types_eth::AccessList) {
        self.inner.set_access_list(access_list)
    }

    fn blob_sidecar(&self) -> Option<&alloy_rpc_types_eth::BlobTransactionSidecar> {
        TransactionBuilder::blob_sidecar(&self.inner)
    }

    fn set_blob_sidecar(&mut self, sidecar: alloy_rpc_types_eth::BlobTransactionSidecar) {
        self.inner.set_blob_sidecar(sidecar)
    }

    fn complete_type(&self, ty: <Era as Network>::TxType) -> Result<(), Vec<&'static str>> {
        // TODO: cover era-specific types.
        let eth_ty = ty
            .as_eth_type()
            .expect("Era-specific types are not supported yet");
        TransactionBuilder::complete_type(&self.inner, eth_ty)
    }

    fn can_submit(&self) -> bool {
        TransactionBuilder::can_submit(&self.inner)
    }

    fn can_build(&self) -> bool {
        TransactionBuilder::can_build(&self.inner)
    }

    fn output_tx_type(&self) -> <Era as Network>::TxType {
        TransactionBuilder::output_tx_type(&self.inner).into()
    }

    fn output_tx_type_checked(&self) -> Option<<Era as Network>::TxType> {
        TransactionBuilder::output_tx_type_checked(&self.inner).map(Into::into)
    }

    fn prep_for_submission(&mut self) {
        TransactionBuilder::prep_for_submission(&mut self.inner)
    }

    fn build_unsigned(
        self,
    ) -> alloy_network::BuildResult<crate::network::unsigned_tx::TypedTransaction, Era> {
        use TransactionBuilderError::*;

        let result = TransactionBuilder::build_unsigned(self.inner);
        match result {
            Ok(tx) => Ok(crate::network::unsigned_tx::TypedTransaction { inner: tx }),
            Err(err) => {
                let UnbuiltTransactionError { request, error } = err;
                let wrapped_request = Self { inner: request };
                let error = match error {
                    InvalidTransactionRequest(tx, fields) => {
                        InvalidTransactionRequest(tx.into(), fields)
                    }
                    UnsupportedSignatureType => UnsupportedSignatureType,
                    Signer(s) => Signer(s),
                    Custom(c) => Custom(c),
                };

                Err(UnbuiltTransactionError {
                    request: wrapped_request,
                    error,
                })
            }
        }
    }

    async fn build<W: alloy_network::NetworkWallet<Era>>(
        self,
        wallet: &W,
    ) -> Result<<Era as Network>::TxEnvelope, TransactionBuilderError<Era>> {
        Ok(wallet.sign_request(self).await?)
    }
}
