use alloy_network::Ethereum;
use alloy_network::Network;

pub mod header;
pub mod header_response;
pub mod receipt_envelope;
pub mod receipt_response;
pub mod transaction_request;
pub mod transaction_response;
pub mod tx_envelope;
pub mod tx_type;
pub mod unsigned_tx;

#[derive(Debug, Clone, Copy)]
pub struct Era {
    _private: (),
}

impl Network for Era {
    type TxType = self::tx_type::TxType;

    type TxEnvelope = self::tx_envelope::TxEnvelope;

    type UnsignedTx = self::unsigned_tx::TypedTransaction;

    type ReceiptEnvelope = <Ethereum as Network>::ReceiptEnvelope;

    type Header = <Ethereum as Network>::Header;

    type TransactionRequest = self::transaction_request::TransactionRequest;

    type TransactionResponse = <Ethereum as Network>::TransactionResponse;

    type ReceiptResponse = <Ethereum as Network>::ReceiptResponse;

    type HeaderResponse = <Ethereum as Network>::HeaderResponse;
}
