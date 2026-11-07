//! Lowercase hexadecimal encoding and decoding.
//!
//! Uppercase input is rejected rather than normalized so that a canonical
//! value has exactly one accepted spelling.

use crate::error::SecurityError;

/// Decodes a lowercase hexadecimal string into exactly `expected_bytes` bytes.
///
/// # Errors
///
/// Returns [`SecurityError::InvalidHexLength`] when the input length is wrong
/// and [`SecurityError::InvalidHexCharacter`] when a character is not a
/// lowercase hexadecimal digit.
pub fn decode_lower_hex(value: &str, expected_bytes: usize) -> Result<Vec<u8>, SecurityError> {
    if value.len() != expected_bytes * 2 {
        return Err(SecurityError::InvalidHexLength {
            expected_bytes,
            actual_chars: value.len(),
        });
    }

    let mut output = Vec::with_capacity(expected_bytes);
    for (index, pair) in value.as_bytes().as_chunks::<2>().0.iter().enumerate() {
        let high = decode_nibble(pair[0], index * 2)?;
        let low = decode_nibble(pair[1], index * 2 + 1)?;
        output.push((high << 4) | low);
    }
    Ok(output)
}

/// Encodes bytes as lowercase hexadecimal.
#[must_use]
pub fn encode_lower_hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}

fn decode_nibble(byte: u8, index: usize) -> Result<u8, SecurityError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err(SecurityError::InvalidHexCharacter { index }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_lowercase_hex() {
        let bytes = [0x00, 0x01, 0x0f, 0x10, 0xab, 0xff];
        let encoded = encode_lower_hex(&bytes);
        assert_eq!(encoded, "00010f10abff");
        assert_eq!(decode_lower_hex(&encoded, bytes.len()).unwrap(), bytes);
    }

    #[test]
    fn rejects_uppercase() {
        let error = decode_lower_hex("AB", 1).unwrap_err();
        assert_eq!(error, SecurityError::InvalidHexCharacter { index: 0 });
    }

    #[test]
    fn rejects_odd_length() {
        let error = decode_lower_hex("abc", 2).unwrap_err();
        assert_eq!(
            error,
            SecurityError::InvalidHexLength {
                expected_bytes: 2,
                actual_chars: 3,
            }
        );
    }
}
