use crate::replay_parser::ParseError;

use nom::{bytes::complete::take, number::complete::le_u32};

#[derive(Debug)]
pub struct Header {
    pub crc: u32,
    pub chunks: u32,
    pub bytes: u32,
    pub replay_version: String,
    pub remaining_file_size: u32,
}

pub fn parse_header(input: &[u8]) -> Result<(&[u8], Header), ParseError> {
    let (input, crc) = le_u32(input)?;
    let (input, chunks) = le_u32(input)?;
    let (input, bytes) = le_u32(input)?;
    let (input, version_bytes) = take(4usize)(input)?;
    let (input, remaining_file_size) = le_u32(input)?;

    if chunks != 1 {
        return Err(ParseError::InvalidData(format!(
            "Expected chunks to be 1, got {}",
            chunks
        )));
    }
    if bytes != 4 {
        return Err(ParseError::InvalidData(format!(
            "Expected bytes to be 4, got {}",
            bytes
        )));
    }

    let replay_version = String::from_utf8_lossy(version_bytes).to_string();

    Ok((
        input,
        Header {
            crc,
            chunks,
            bytes,
            replay_version,
            remaining_file_size,
        },
    ))
}
