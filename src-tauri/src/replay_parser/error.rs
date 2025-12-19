use std::fmt;

#[derive(Debug)]
pub enum ParseError {
    NomError(nom::Err<nom::error::Error<Vec<u8>>>),
    UnsupportedVersion(String),
    InvalidData(String),
    DecompressionError(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::NomError(e) => write!(f, "Parse error: {:?}", e),
            ParseError::UnsupportedVersion(v) => write!(f, "Unsupported replay version: {}", v),
            ParseError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ParseError::DecompressionError(msg) => write!(f, "Decompression error: {}", msg),
        }
    }
}

impl std::error::Error for ParseError {}

impl<T> From<nom::Err<nom::error::Error<T>>> for ParseError
where
    T: Into<Vec<u8>>,
{
    fn from(err: nom::Err<nom::error::Error<T>>) -> Self {
        match err {
            nom::Err::Error(e) => ParseError::NomError(nom::Err::Error(nom::error::Error {
                input: e.input.into(),
                code: e.code,
            })),
            nom::Err::Failure(e) => ParseError::NomError(nom::Err::Failure(nom::error::Error {
                input: e.input.into(),
                code: e.code,
            })),
            nom::Err::Incomplete(n) => ParseError::NomError(nom::Err::Incomplete(n)),
        }
    }
}
