use std::{
    fmt::{self, Display},
    string::FromUtf8Error,
};

use anyhow::Result;
use crc32fast::Hasher;
use thiserror::Error;

use super::chunk_type::{PNG_CHUNK_TYPE_LENGTH, PngChunkType, PngChunkTypeParsingError};

#[derive(Debug)]
pub struct PngChunk {
    pub length: u32,
    pub chunk_type: PngChunkType,
    pub data: Vec<u8>,
}

pub const PNG_CHUNK_DATA_LENGTH_LENGTH: usize = 4;
pub const CRC_LENGTH: usize = 4;
pub const PNG_CHUNK_MINIMUM_LENGTH: usize =
    PNG_CHUNK_DATA_LENGTH_LENGTH + PNG_CHUNK_TYPE_LENGTH + CRC_LENGTH;

#[derive(Error, Debug)]
pub enum PngChunkParsingError {
    #[error(
        "expected at least {PNG_CHUNK_MINIMUM_LENGTH} bytes (length, chunk type and crc), got {0}"
    )]
    InvalidMinimumLength(usize),
    #[error("expected data of length {expected}, got {got}")]
    InvalidDataLength { expected: usize, got: usize },
    #[error("expected crc {expected}, got {got}")]
    InvalidCRC { expected: u32, got: u32 },
    #[error(transparent)]
    InvalidChunkType(#[from] PngChunkTypeParsingError),
}

impl TryFrom<&[u8]> for PngChunk {
    type Error = PngChunkParsingError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < PNG_CHUNK_MINIMUM_LENGTH {
            return Err(PngChunkParsingError::InvalidMinimumLength(bytes.len()));
        }

        let data_length_bytes: [u8; 4] = bytes[0..PNG_CHUNK_DATA_LENGTH_LENGTH].try_into().unwrap();
        let data_length = u32::from_be_bytes(data_length_bytes);

        if bytes.len() < (data_length as usize) + PNG_CHUNK_TYPE_LENGTH + CRC_LENGTH {
            return Err(PngChunkParsingError::InvalidDataLength {
                expected: data_length as usize,
                got: bytes.len(),
            });
        }

        let chunk_type_bytes: [u8; 4] = bytes[4..8].try_into().unwrap();
        let chunk_type = PngChunkType::try_from(chunk_type_bytes)?;

        let data_last_index = 8 + data_length as usize;
        let data = bytes[8..data_last_index].to_vec();

        let crc_bytes: [u8; 4] = bytes[data_last_index..(data_last_index + 4)]
            .try_into()
            .unwrap();
        let crc = u32::from_be_bytes(crc_bytes);

        let chunk = PngChunk {
            length: data_length,
            chunk_type,
            data,
        };

        let calculated_crc = chunk.crc();

        if crc != calculated_crc {
            return Err(PngChunkParsingError::InvalidCRC {
                expected: calculated_crc,
                got: crc,
            });
        }

        Ok(chunk)
    }
}

impl Display for PngChunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({}){:?} (CRC: {})",
            self.chunk_type.to_string(),
            self.length,
            self.data,
            self.crc()
        )
    }
}

impl PngChunk {
    pub fn new(chunk_type: PngChunkType, data: Vec<u8>) -> PngChunk {
        PngChunk {
            length: data.len() as u32,
            chunk_type,
            data,
        }
    }

    fn crc(&self) -> u32 {
        let mut hasher = Hasher::new();

        hasher.update(&self.chunk_type.bytes());
        hasher.update(&self.data);

        hasher.finalize()
    }

    pub fn data_as_string(&self) -> Result<String, FromUtf8Error> {
        String::from_utf8(self.data.clone())
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        self.length
            .to_be_bytes()
            .iter()
            .chain(self.chunk_type.bytes().iter())
            .chain(self.data.iter())
            .chain(self.crc().to_be_bytes().iter())
            .copied()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::PngChunkType;
    use super::*;
    use std::str::FromStr;

    fn testing_chunk() -> PngChunk {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        PngChunk::try_from(chunk_data.as_ref()).unwrap()
    }

    #[test]
    fn test_new_chunk() {
        let chunk_type = PngChunkType::from_str("RuSt").unwrap();
        let data = "This is where your secret message will be!"
            .as_bytes()
            .to_vec();
        let chunk = PngChunk::new(chunk_type, data);
        assert_eq!(chunk.length, 42);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_chunk_length() {
        let chunk = testing_chunk();
        assert_eq!(chunk.length, 42);
    }

    #[test]
    fn test_chunk_type() {
        let chunk = testing_chunk();
        assert_eq!(chunk.chunk_type.to_string(), String::from("RuSt"));
    }

    #[test]
    fn test_chunk_string() {
        let chunk = testing_chunk();
        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");
        assert_eq!(chunk_string, expected_chunk_string);
    }

    #[test]
    fn test_chunk_crc() {
        let chunk = testing_chunk();
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_valid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = PngChunk::try_from(chunk_data.as_ref()).unwrap();

        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");

        assert_eq!(chunk.length, 42);
        assert_eq!(chunk.chunk_type.to_string(), String::from("RuSt"));
        assert_eq!(chunk_string, expected_chunk_string);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_invalid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656333;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = PngChunk::try_from(chunk_data.as_ref());

        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_trait_impls() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk: PngChunk = TryFrom::try_from(chunk_data.as_ref()).unwrap();

        let _chunk_string = format!("{}", chunk);
    }
}
