use thiserror::Error;

/// Errors that can occur while parsing IPv4 addresses or CIDR networks.
///
/// # Examples
///
/// ```
/// use ipnet_rs::{Ipv4Addr, ParseError};
///
/// let err = "192.168.1".parse::<Ipv4Addr>().unwrap_err();
/// assert_eq!(err, ParseError::InvalidOctetCount);
/// ```
#[derive(Error, Debug, PartialEq, Eq)]
pub enum ParseError {
    /// An IPv4 address did not contain exactly four dot-separated octets.
    #[error("invalid number of octets (expected 4)")]
    InvalidOctetCount,

    /// An octet was not a valid number in the range 0–255.
    ///
    /// The wrapped string is the offending octet text.
    #[error("invalid octet {0}")]
    InvalidOctet(String),

    /// A CIDR prefix length was missing, non-numeric, or out of range (0–32).
    ///
    /// The wrapped string is the offending prefix text.
    #[error("invalid prefix length: {0}")]
    InvalidPrefixLength(String),

    /// A CIDR string was missing the `/prefix` part.
    #[error("missing prefix length after '/'")]
    MissingPrefix,
}
