use std::{
    fmt::{self, Display},
    str::FromStr,
};

use thiserror::Error;

pub const PNG_CHUNK_TYPE_LENGTH: usize = 4;

pub type PngChunkTypeBinaryData = [u8; PNG_CHUNK_TYPE_LENGTH];
pub type PngChunkTypeData = [char; PNG_CHUNK_TYPE_LENGTH];

#[derive(Debug, Eq, PartialEq)]
pub struct PngChunkType(pub PngChunkTypeData);

#[derive(Error, Debug)]
pub enum PngChunkTypeParsingError {
    #[error("expected string of length {expected}, got {got}")]
    InvalidStringLength { expected: usize, got: usize },
    #[error("expected ascii character string")]
    InvalidAsciiCharacters,
}

impl TryFrom<PngChunkTypeBinaryData> for PngChunkType {
    type Error = PngChunkTypeParsingError;

    fn try_from(bytes: PngChunkTypeBinaryData) -> Result<Self, Self::Error> {
        let instance = PngChunkType([
            char::from(bytes[0]),
            char::from(bytes[1]),
            char::from(bytes[2]),
            char::from(bytes[3]),
        ]);

        if !instance.is_valid_chars() {
            return Err(PngChunkTypeParsingError::InvalidAsciiCharacters);
        }

        Ok(instance)
    }
}

impl FromStr for PngChunkType {
    type Err = PngChunkTypeParsingError;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        if str.len() != PNG_CHUNK_TYPE_LENGTH {
            return Err(PngChunkTypeParsingError::InvalidStringLength {
                expected: PNG_CHUNK_TYPE_LENGTH,
                got: str.len(),
            });
        }

        let chars = &mut str.chars();

        PngChunkType::try_from(
            [chars.next(), chars.next(), chars.next(), chars.next()]
                .map(|maybe_char| maybe_char.unwrap() as u8),
        )
    }
}

impl Display for PngChunkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", String::from_iter(self.0))
    }
}

fn is_ascii_letter_byte(byte: u8) -> bool {
    (byte >= 65 && byte <= 90) || (byte >= 97 && byte <= 122)
}

impl PngChunkType {
    pub fn bytes(&self) -> PngChunkTypeBinaryData {
        self.0.map(|char| char as u8)
    }

    fn is_valid_chars(&self) -> bool {
        self.bytes().iter().all(|&byte| is_ascii_letter_byte(byte))
    }

    #[allow(unused)]
    fn is_valid(&self) -> bool {
        self.0[2].is_uppercase() && self.is_valid_chars()
    }

    #[allow(unused)]
    fn is_critical(&self) -> bool {
        self.0[0].is_uppercase()
    }

    #[allow(unused)]
    fn is_public(&self) -> bool {
        self.0[1].is_uppercase()
    }

    #[allow(unused)]
    fn is_reserved_bit_valid(&self) -> bool {
        self.0[2].is_uppercase()
    }

    #[allow(unused)]
    fn is_safe_to_copy(&self) -> bool {
        self.0[3].is_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use std::str::FromStr;

    #[test]
    pub fn test_chunk_type_from_bytes() {
        let expected = [82, 117, 83, 116];
        let actual = PngChunkType::try_from([82, 117, 83, 116]).unwrap();

        assert_eq!(expected, actual.bytes());
    }

    #[test]
    pub fn test_chunk_type_from_str() {
        let expected = PngChunkType::try_from([82, 117, 83, 116]).unwrap();
        let actual = PngChunkType::from_str("RuSt").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    pub fn test_chunk_type_is_critical() {
        let chunk = PngChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_not_critical() {
        let chunk = PngChunkType::from_str("ruSt").unwrap();
        assert!(!chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_public() {
        let chunk = PngChunkType::from_str("RUSt").unwrap();
        assert!(chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_not_public() {
        let chunk = PngChunkType::from_str("RuSt").unwrap();
        assert!(!chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_valid() {
        let chunk = PngChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_invalid() {
        let chunk = PngChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_safe_to_copy() {
        let chunk = PngChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_chunk_type_is_unsafe_to_copy() {
        let chunk = PngChunkType::from_str("RuST").unwrap();
        assert!(!chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_valid_chunk_is_valid() {
        let chunk = PngChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_valid());
    }

    #[test]
    pub fn test_invalid_chunk_is_valid() {
        let chunk = PngChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_valid());

        let chunk = PngChunkType::from_str("Ru1t");
        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_type_string() {
        let chunk = PngChunkType::from_str("RuSt").unwrap();
        assert_eq!(&chunk.to_string(), "RuSt");
    }

    #[test]
    pub fn test_chunk_type_trait_impls() {
        let chunk_type_1: PngChunkType = TryFrom::try_from([82, 117, 83, 116]).unwrap();
        let chunk_type_2: PngChunkType = FromStr::from_str("RuSt").unwrap();
        let _chunk_string = format!("{}", chunk_type_1);
        let _are_chunks_equal = chunk_type_1 == chunk_type_2;
    }
}
