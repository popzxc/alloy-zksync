use k256::sha2::{self, Digest};

// Bytecode length in words must fit in u16.
const WORD_SIZE: usize = 32;
const MAX_BYTECODE_LENGTH: usize = WORD_SIZE * u16::MAX as usize;

#[derive(Debug, thiserror::Error)]
pub enum BytecodeHashError {
    #[error("Bytecode cannot be split into 32-byte words")]
    BytecodeNotAligned,
    #[error(
        "Bytecode length exceeds limit: {num_words} words, the maximum is {MAX_BYTECODE_LENGTH}"
    )]
    BytecodeLengthExceedsLimit { num_words: usize },
    #[error("Bytecode must have odd number of words")]
    NumberOfWordsMustBeOdd,
}

/// The 32-byte hash of the bytecode of a zkSync contract is calculated in the following way:
///
/// * The first 2 bytes denote the version of bytecode hash format and are currently equal to `[1,0]`.
/// * The second 2 bytes denote the length of the bytecode in 32-byte words.
/// * The rest of the 28-byte (i.e. 28 low big-endian bytes) are equal to the last 28 bytes of the sha256 hash of the contract's bytecode.
///
/// This function performs validity checks for bytecode:
/// - The bytecode must be aligned to 32-byte words.
/// - The bytecode length must not exceed the maximum allowed value.
/// - The number of words must be odd.
pub fn hash_bytecode(bytecode: &[u8]) -> Result<[u8; 32], BytecodeHashError> {
    if bytecode.len() % WORD_SIZE != 0 {
        return Err(BytecodeHashError::BytecodeNotAligned);
    }

    let bytecode_length = bytecode.len() / WORD_SIZE;
    let bytecode_length = u16::try_from(bytecode_length).map_err(|_| {
        BytecodeHashError::BytecodeLengthExceedsLimit {
            num_words: bytecode_length,
        }
    })?;
    if bytecode_length % 2 == 0 {
        return Err(BytecodeHashError::NumberOfWordsMustBeOdd);
    }

    let bytecode_hash: [u8; 32] = sha2::Sha256::digest(bytecode).into();

    let mut contract_hash: [u8; 32] = [0u8; 32];
    contract_hash[..2].copy_from_slice(&0x0100_u16.to_be_bytes());
    contract_hash[2..4].copy_from_slice(&bytecode_length.to_be_bytes());
    contract_hash[4..].copy_from_slice(&bytecode_hash[4..]);

    Ok(contract_hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;

    #[test]
    fn bytecode_hash() {
        // Reference values calculated using zksync codebase.
        #[rustfmt::skip]
        let test_vector = [
            (vec![10u8; 32], hex::decode("01000001e7718454476f04edeb935022ae4f4d90934ab7ce913ff20c8baeb399").unwrap()),
            (vec![20u8; 96], hex::decode("01000003c743f1d99f4d7dc11f5d9630e32ff5a212c5aaf64c7ac815193463d4").unwrap()),
        ];

        for (input, expected) in test_vector.iter() {
            let hash = hash_bytecode(input).unwrap();
            assert_eq!(&hash[..], &expected[..]);
        }

        assert_matches!(
            hash_bytecode(&[]),
            Err(BytecodeHashError::NumberOfWordsMustBeOdd)
        );
        assert_matches!(
            hash_bytecode(&[1]),
            Err(BytecodeHashError::BytecodeNotAligned)
        );
    }
}
