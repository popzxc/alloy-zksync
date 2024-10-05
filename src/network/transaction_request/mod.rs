use alloy_network::{
    Network, TransactionBuilder, TransactionBuilderError, UnbuiltTransactionError,
};

use super::{unsigned_tx::eip712::Eip712Meta, Zksync};

pub mod eip712;

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct TransactionRequest {
    #[serde(flatten)]
    base: alloy_rpc_types_eth::transaction::TransactionRequest,
    #[serde(skip_serializing_if = "Option::is_none")]
    eip_712_meta: Option<Eip712Meta>,
}

impl From<crate::network::unsigned_tx::TypedTransaction> for TransactionRequest {
    fn from(value: crate::network::unsigned_tx::TypedTransaction) -> Self {
        match value {
            crate::network::unsigned_tx::TypedTransaction::Native(inner) => Self {
                base: inner.into(),
                eip_712_meta: None,
            },
            crate::network::unsigned_tx::TypedTransaction::Eip712(inner) => Self {
                base: inner.clone().into(),
                eip_712_meta: Some(inner.eip712_meta),
            },
        }
    }
}

impl From<crate::network::tx_envelope::TxEnvelope> for TransactionRequest {
    fn from(value: crate::network::tx_envelope::TxEnvelope) -> Self {
        match value {
            crate::network::tx_envelope::TxEnvelope::Native(inner) => Self {
                base: inner.into(),
                eip_712_meta: None,
            },
            crate::network::tx_envelope::TxEnvelope::Eip712(signed) => Self {
                base: signed.tx().clone().into(),
                eip_712_meta: Some(signed.tx().clone().eip712_meta),
            },
        }
    }
}

/// Macro that delegates a method call to the inner variant implementation.
// TODO: not necessary, to be removed.
macro_rules! delegate {
    ($_self:ident.$inner:ident.$method:ident($($args:expr),*)) => {
        TransactionBuilder::$method($_self.$inner, $($args),*)
    };
}
impl TransactionBuilder<Zksync> for TransactionRequest {
    fn chain_id(&self) -> Option<alloy_primitives::ChainId> {
        TransactionBuilder::chain_id(&self.base)
    }

    fn set_chain_id(&mut self, chain_id: alloy_primitives::ChainId) {
        TransactionBuilder::set_chain_id(&mut self.base, chain_id)
    }

    fn nonce(&self) -> Option<u64> {
        TransactionBuilder::nonce(&self.base)
    }

    fn set_nonce(&mut self, nonce: u64) {
        TransactionBuilder::set_nonce(&mut self.base, nonce)
    }

    fn input(&self) -> Option<&alloy_primitives::Bytes> {
        TransactionBuilder::input(&self.base)
    }

    fn set_input<T: Into<alloy_primitives::Bytes>>(&mut self, input: T) {
        TransactionBuilder::set_input(&mut self.base, input.into())
    }

    fn from(&self) -> Option<alloy_primitives::Address> {
        TransactionBuilder::from(&self.base)
    }

    fn set_from(&mut self, from: alloy_primitives::Address) {
        TransactionBuilder::set_from(&mut self.base, from)
    }

    fn kind(&self) -> Option<alloy_primitives::TxKind> {
        TransactionBuilder::kind(&self.base)
    }

    fn clear_kind(&mut self) {
        TransactionBuilder::clear_kind(&mut self.base)
    }

    fn set_kind(&mut self, kind: alloy_primitives::TxKind) {
        TransactionBuilder::set_kind(&mut self.base, kind)
    }

    fn value(&self) -> Option<alloy_primitives::U256> {
        TransactionBuilder::value(&self.base)
    }

    fn set_value(&mut self, value: alloy_primitives::U256) {
        TransactionBuilder::set_value(&mut self.base, value)
    }

    fn gas_price(&self) -> Option<u128> {
        TransactionBuilder::gas_price(&self.base)
    }

    fn set_gas_price(&mut self, gas_price: u128) {
        TransactionBuilder::set_gas_price(&mut self.base, gas_price)
    }

    fn max_fee_per_gas(&self) -> Option<u128> {
        TransactionBuilder::max_fee_per_gas(&self.base)
    }

    fn set_max_fee_per_gas(&mut self, max_fee_per_gas: u128) {
        TransactionBuilder::set_max_fee_per_gas(&mut self.base, max_fee_per_gas)
    }

    fn max_priority_fee_per_gas(&self) -> Option<u128> {
        TransactionBuilder::max_priority_fee_per_gas(&self.base)
    }

    fn set_max_priority_fee_per_gas(&mut self, max_priority_fee_per_gas: u128) {
        TransactionBuilder::set_max_priority_fee_per_gas(&mut self.base, max_priority_fee_per_gas)
    }

    fn max_fee_per_blob_gas(&self) -> Option<u128> {
        TransactionBuilder::max_fee_per_blob_gas(&self.base)
    }

    fn set_max_fee_per_blob_gas(&mut self, max_fee_per_blob_gas: u128) {
        TransactionBuilder::set_max_fee_per_blob_gas(&mut self.base, max_fee_per_blob_gas)
    }

    fn gas_limit(&self) -> Option<u128> {
        TransactionBuilder::gas_limit(&self.base)
    }

    fn set_gas_limit(&mut self, gas_limit: u128) {
        TransactionBuilder::set_gas_limit(&mut self.base, gas_limit)
    }

    fn access_list(&self) -> Option<&alloy_rpc_types_eth::AccessList> {
        TransactionBuilder::access_list(&self.base)
    }

    fn set_access_list(&mut self, access_list: alloy_rpc_types_eth::AccessList) {
        TransactionBuilder::set_access_list(&mut self.base, access_list)
    }

    fn blob_sidecar(&self) -> Option<&alloy_rpc_types_eth::BlobTransactionSidecar> {
        TransactionBuilder::blob_sidecar(&self.base)
    }

    fn set_blob_sidecar(&mut self, sidecar: alloy_rpc_types_eth::BlobTransactionSidecar) {
        TransactionBuilder::set_blob_sidecar(&mut self.base, sidecar)
    }

    fn complete_type(&self, ty: <Zksync as Network>::TxType) -> Result<(), Vec<&'static str>> {
        // TODO: cover era-specific types.
        let eth_ty = ty
            .as_eth_type()
            .expect("Era-specific types are not supported yet");
        TransactionBuilder::complete_type(&self.base, eth_ty)
    }

    fn can_submit(&self) -> bool {
        TransactionBuilder::can_submit(&self.base)
    }

    fn can_build(&self) -> bool {
        TransactionBuilder::can_build(&self.base)
    }

    fn output_tx_type(&self) -> <Zksync as Network>::TxType {
        TransactionBuilder::output_tx_type(&self.base).into()
    }

    fn output_tx_type_checked(&self) -> Option<<Zksync as Network>::TxType> {
        TransactionBuilder::output_tx_type_checked(&self.base).map(Into::into)
    }

    fn prep_for_submission(&mut self) {
        TransactionBuilder::prep_for_submission(&mut self.base)
    }

    fn build_unsigned(
        self,
    ) -> alloy_network::BuildResult<crate::network::unsigned_tx::TypedTransaction, Zksync> {
        // TODO: Support era-specific
        if self.eip_712_meta.is_some() {
            todo!("Era-specific transactions are not supported yet");
        }

        use TransactionBuilderError::*;
        let inner = self.base;

        let result = TransactionBuilder::build_unsigned(inner);
        match result {
            Ok(tx) => Ok(crate::network::unsigned_tx::TypedTransaction::Native(tx)),
            Err(err) => {
                let UnbuiltTransactionError { request, error } = err;
                let wrapped_request = Self {
                    base: request,
                    eip_712_meta: None,
                };
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

    async fn build<W: alloy_network::NetworkWallet<Zksync>>(
        self,
        wallet: &W,
    ) -> Result<<Zksync as Network>::TxEnvelope, TransactionBuilderError<Zksync>> {
        Ok(wallet.sign_request(self).await?)
    }
}
