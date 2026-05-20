use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ParseError {
    #[error("invalid number of octets (expected 4)")]
    InvalidOctetCount,

    #[error("invalid octet {0}")]
    InvalidOctet(String),

    #[error("invalid prefix length: {0}")]
    InvalidPrefixLength(String),

    #[error("missing prefix length after '/'")]
    MissingPrefix,
}
