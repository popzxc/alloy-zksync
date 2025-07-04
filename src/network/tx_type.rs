use std::fmt;

use alloy::network::eip2718::Eip2718Error;

/// Transaction types supported by the Era network.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[doc(alias = "TransactionType")]
pub enum TxType {
    /// Legacy transaction type.
    Legacy = 0,
    /// EIP-2930 transaction type.
    Eip2930 = 1,
    /// EIP-1559 transaction type.
    Eip1559 = 2,
    /// EIP-4844 transaction type.
    Eip4844 = 3,
    /// EIP-7702 transaction type.
    Eip7702 = 4,
    /// ZKsync-specific EIP712-based transaction type.
    Eip712 = 0x71,
    // TODO: L1-based transaction type
}

impl TxType {
    /// Tries to represent transaction as an Ethereum transaction type.
    /// Will return `None` for ZKsync-specific transactions.
    pub fn as_eth_type(self) -> Option<alloy::consensus::TxType> {
        Some(match self {
            TxType::Legacy => alloy::consensus::TxType::Legacy,
            TxType::Eip2930 => alloy::consensus::TxType::Eip2930,
            TxType::Eip1559 => alloy::consensus::TxType::Eip1559,
            TxType::Eip4844 => alloy::consensus::TxType::Eip4844,
            TxType::Eip7702 => alloy::consensus::TxType::Eip7702,
            TxType::Eip712 => return None,
        })
    }
}

impl alloy::consensus::Typed2718 for TxType {
    fn ty(&self) -> u8 {
        u8::from(*self)
    }
}

impl From<alloy::consensus::TxType> for TxType {
    fn from(value: alloy::consensus::TxType) -> Self {
        let raw_value = value as u8;
        Self::try_from(raw_value).expect("Era supports all Eth tx types")
    }
}

impl From<TxType> for u8 {
    fn from(value: TxType) -> Self {
        value as Self
    }
}

impl fmt::Display for TxType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Legacy => write!(f, "Legacy"),
            Self::Eip2930 => write!(f, "EIP-2930"),
            Self::Eip1559 => write!(f, "EIP-1559"),
            Self::Eip4844 => write!(f, "EIP-4844"),
            Self::Eip7702 => write!(f, "EIP-7702"),
            Self::Eip712 => write!(f, "Era EIP-712"),
        }
    }
}

impl TryFrom<u8> for TxType {
    type Error = Eip2718Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Legacy,
            1 => Self::Eip2930,
            2 => Self::Eip1559,
            3 => Self::Eip4844,
            4 => Self::Eip7702,
            0x71 => Self::Eip712,
            _ => return Err(Eip2718Error::UnexpectedType(value)),
        })
    }
}
