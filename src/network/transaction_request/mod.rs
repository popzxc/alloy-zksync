use alloy::network::{
    Network, TransactionBuilder, TransactionBuilderError, UnbuiltTransactionError,
};
use alloy::primitives::{Bytes, TxKind, B256, U256};

use crate::contracts::l2::contract_deployer::CONTRACT_DEPLOYER_ADDRESS;
use crate::network::{tx_type::TxType, unsigned_tx::eip712::TxEip712};

use super::unsigned_tx::eip712::{hash_bytecode, BytecodeHashError, PaymasterParams};
use super::{unsigned_tx::eip712::Eip712Meta, Zksync};

#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionRequest {
    #[serde(flatten)]
    base: alloy::rpc::types::transaction::TransactionRequest,
    #[serde(skip_serializing_if = "Option::is_none")]
    eip_712_meta: Option<Eip712Meta>,
}

// TODO: Extension trait for `TransactionBuilder`?
impl TransactionRequest {
    pub fn gas_per_pubdata(&self) -> Option<U256> {
        self.eip_712_meta.as_ref().map(|meta| meta.gas_per_pubdata)
    }

    pub fn set_gas_per_pubdata(&mut self, gas_per_pubdata: U256) {
        self.eip_712_meta
            .get_or_insert_with(Eip712Meta::default)
            .gas_per_pubdata = gas_per_pubdata;
    }

    pub fn with_gas_per_pubdata(mut self, gas_per_pubdata: U256) -> Self {
        self.set_gas_per_pubdata(gas_per_pubdata);
        self
    }

    pub fn set_paymaster(&mut self, paymaster_params: PaymasterParams) {
        self.eip_712_meta
            .get_or_insert_with(Eip712Meta::default)
            .paymaster_params = Some(paymaster_params);
    }

    pub fn with_paymaster(mut self, paymaster_params: PaymasterParams) -> Self {
        self.set_paymaster(paymaster_params);
        self
    }

    pub fn set_factory_deps(&mut self, factory_deps: Vec<Bytes>) {
        self.eip_712_meta
            .get_or_insert_with(Eip712Meta::default)
            .factory_deps = factory_deps;
    }

    pub fn with_factory_deps(mut self, factory_deps: Vec<Bytes>) -> Self {
        self.set_factory_deps(factory_deps);
        self
    }

    pub fn set_custom_signature(&mut self, custom_signature: Bytes) {
        self.eip_712_meta
            .get_or_insert_with(Eip712Meta::default)
            .custom_signature = Some(custom_signature);
    }

    pub fn with_custom_signature(mut self, custom_signature: Bytes) -> Self {
        self.set_custom_signature(custom_signature);
        self
    }

    pub fn zksync_deploy(
        self,
        code: Vec<u8>,
        constructor_data: Vec<u8>,
        factory_deps: Vec<Vec<u8>>,
    ) -> Result<Self, BytecodeHashError> {
        self.zksync_deploy_inner(None, code, constructor_data, factory_deps)
    }

    pub fn zksync_deploy_with_salt(
        self,
        salt: B256,
        code: Vec<u8>,
        constructor_data: Vec<u8>,
        factory_deps: Vec<Vec<u8>>,
    ) -> Result<Self, BytecodeHashError> {
        self.zksync_deploy_inner(Some(salt), code, constructor_data, factory_deps)
    }

    fn zksync_deploy_inner(
        mut self,
        salt: Option<B256>,
        code: Vec<u8>,
        constructor_data: Vec<u8>,
        mut factory_deps: Vec<Vec<u8>>,
    ) -> Result<Self, BytecodeHashError> {
        self.base.transaction_type = Some(TxType::Eip712 as u8);

        let bytecode_hash = hash_bytecode(&code)?;
        factory_deps.push(code);
        self.base.to = Some(CONTRACT_DEPLOYER_ADDRESS.into());
        self.base.input = match salt {
            Some(salt) => crate::contracts::l2::contract_deployer::encode_create2_calldata(
                salt,
                bytecode_hash.into(),
                constructor_data.into(),
            ),
            None => crate::contracts::l2::contract_deployer::encode_create_calldata(
                bytecode_hash.into(),
                constructor_data.into(),
            ),
        }
        .into();
        self.eip_712_meta
            .get_or_insert_with(Eip712Meta::default)
            .factory_deps = factory_deps.into_iter().map(Into::into).collect();
        Ok(self)
    }
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

// TODO: transactionbuilder for other networks

impl TransactionBuilder<Zksync> for TransactionRequest {
    fn chain_id(&self) -> Option<alloy::primitives::ChainId> {
        TransactionBuilder::chain_id(&self.base)
    }

    fn set_chain_id(&mut self, chain_id: alloy::primitives::ChainId) {
        TransactionBuilder::set_chain_id(&mut self.base, chain_id)
    }

    fn nonce(&self) -> Option<u64> {
        TransactionBuilder::nonce(&self.base)
    }

    fn set_nonce(&mut self, nonce: u64) {
        TransactionBuilder::set_nonce(&mut self.base, nonce)
    }

    fn input(&self) -> Option<&alloy::primitives::Bytes> {
        TransactionBuilder::input(&self.base)
    }

    fn set_input<T: Into<alloy::primitives::Bytes>>(&mut self, input: T) {
        TransactionBuilder::set_input(&mut self.base, input.into())
    }

    fn from(&self) -> Option<alloy::primitives::Address> {
        TransactionBuilder::from(&self.base)
    }

    fn set_from(&mut self, from: alloy::primitives::Address) {
        TransactionBuilder::set_from(&mut self.base, from)
    }

    fn kind(&self) -> Option<alloy::primitives::TxKind> {
        TransactionBuilder::kind(&self.base)
    }

    fn clear_kind(&mut self) {
        TransactionBuilder::clear_kind(&mut self.base)
    }

    fn set_kind(&mut self, kind: alloy::primitives::TxKind) {
        TransactionBuilder::set_kind(&mut self.base, kind)
    }

    fn value(&self) -> Option<alloy::primitives::U256> {
        TransactionBuilder::value(&self.base)
    }

    fn set_value(&mut self, value: alloy::primitives::U256) {
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

    fn gas_limit(&self) -> Option<u64> {
        TransactionBuilder::gas_limit(&self.base)
    }

    fn set_gas_limit(&mut self, gas_limit: u64) {
        TransactionBuilder::set_gas_limit(&mut self.base, gas_limit)
    }

    fn access_list(&self) -> Option<&alloy::rpc::types::AccessList> {
        TransactionBuilder::access_list(&self.base)
    }

    fn set_access_list(&mut self, access_list: alloy::rpc::types::AccessList) {
        TransactionBuilder::set_access_list(&mut self.base, access_list)
    }

    fn complete_type(&self, ty: <Zksync as Network>::TxType) -> Result<(), Vec<&'static str>> {
        // TODO: cover era-specific types.
        match ty {
            TxType::Eip712 => {
                // TODO: Should check gas per pubdata?
                TransactionBuilder::complete_type(&self.base, alloy::consensus::TxType::Eip1559)
            }
            _ if ty.as_eth_type().is_some() => {
                TransactionBuilder::complete_type(&self.base, ty.as_eth_type().unwrap())
            }
            _ => Err(vec!["Unsupported transaction type"]),
        }
    }

    fn can_submit(&self) -> bool {
        TransactionBuilder::can_submit(&self.base)
    }

    fn can_build(&self) -> bool {
        if self.eip_712_meta.is_some() {
            let common = self.base.gas.is_some() && self.base.nonce.is_some();
            let eip1559 =
                self.base.max_fee_per_gas.is_some() && self.base.max_priority_fee_per_gas.is_some();
            // TODO: Should check gas per pubdata?
            return common && eip1559;
        }

        TransactionBuilder::can_build(&self.base)
    }

    fn output_tx_type(&self) -> <Zksync as Network>::TxType {
        if self.eip_712_meta.is_some() {
            return TxType::Eip712;
        }

        TransactionBuilder::output_tx_type(&self.base).into()
    }

    fn output_tx_type_checked(&self) -> Option<<Zksync as Network>::TxType> {
        if self.eip_712_meta.is_some() {
            if !self.can_build() {
                return None;
            }
            return Some(TxType::Eip712);
        }

        TransactionBuilder::output_tx_type_checked(&self.base).map(Into::into)
    }

    fn prep_for_submission(&mut self) {
        // This has to go first, as it overwrites the transaction type.
        TransactionBuilder::prep_for_submission(&mut self.base);

        if self.eip_712_meta.is_some() {
            self.base.transaction_type = Some(TxType::Eip712 as u8);
            self.base.gas_price = None;
            self.base.blob_versioned_hashes = None;
            self.base.sidecar = None;
        }
    }

    fn build_unsigned(
        self,
    ) -> alloy::network::BuildResult<crate::network::unsigned_tx::TypedTransaction, Zksync> {
        if self.eip_712_meta.is_some() {
            let mut missing = Vec::new();
            // TODO: Copy-paste, can be avoided?
            if self.base.max_fee_per_gas.is_none() {
                missing.push("max_fee_per_gas");
            }
            if self.base.max_priority_fee_per_gas.is_none() {
                missing.push("max_priority_fee_per_gas");
            }

            if !missing.is_empty() {
                return Err(TransactionBuilderError::InvalidTransactionRequest(
                    TxType::Eip712,
                    missing,
                )
                .into_unbuilt(self));
            }

            let TxKind::Call(to) = self.base.to.unwrap_or_default() else {
                return Err(TransactionBuilderError::InvalidTransactionRequest(
                    TxType::Eip712,
                    vec!["to (recipient) must be specified for EIP-712 transactions"],
                )
                .into_unbuilt(self));
            };

            // TODO: Are unwraps safe?
            let tx = TxEip712 {
                chain_id: self.base.chain_id.unwrap(),
                nonce: U256::from(self.base.nonce.unwrap()), // TODO: Deployment nonce?
                gas_limit: self.base.gas.unwrap(),
                max_fee_per_gas: self.base.max_fee_per_gas.unwrap(),
                max_priority_fee_per_gas: self.base.max_priority_fee_per_gas.unwrap(),
                eip712_meta: self.eip_712_meta.unwrap(),
                from: self.base.from.unwrap(),
                to,
                value: self.base.value.unwrap_or_default(),
                input: self.base.input.into_input().unwrap_or_default(),
            };
            return Ok(crate::network::unsigned_tx::TypedTransaction::Eip712(tx));
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

    async fn build<W: alloy::network::NetworkWallet<Zksync>>(
        self,
        wallet: &W,
    ) -> Result<<Zksync as Network>::TxEnvelope, TransactionBuilderError<Zksync>> {
        Ok(wallet.sign_request(self).await?)
    }
}

impl From<alloy::rpc::types::transaction::TransactionRequest> for TransactionRequest {
    fn from(value: alloy::rpc::types::transaction::TransactionRequest) -> Self {
        Self {
            base: value,
            ..Default::default()
        }
    }
}
