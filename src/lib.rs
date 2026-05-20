//! ipnet-rs: a small IPv4 networking library
//!
//! This crate provides types for working with IPv4 addresses and CIDR networks.
//! It supports parsing, formatting, iteration, subnet arithmetic, and splitting.
//!
//! # Quick start
//!
//! ```
//! use ipnet_rs::{Ipv4Addr, Ipv4Cidr};
//!
//! let net: Ipv4Cidr = "192.168.1.0/24".parse().unwrap();
//! assert_eq!(net.network(), Ipv4Addr::new(192, 168, 1, 0));
//! assert_eq!(net.usable_hosts(), 254);
//! ```
mod cidr;
mod error;
use std::fmt;
use std::str::FromStr;

pub use cidr::Ipv4Cidr;
pub use error::ParseError;

/// An IPv4 address.
///
/// Internally stored as four octets. The type is `Copy` since it only
/// holds 4 bytes of stack data.
///
/// # Examples
///
/// ```
/// use ipnet_rs::Ipv4Addr;
///
/// let addr = Ipv4Addr::new(192, 168, 1, 1);
/// assert_eq!(addr.to_string(), "192.168.1.1");
///
/// let parsed: Ipv4Addr = "10.0.0.1".parse().unwrap();
/// assert_eq!(parsed.octets(), [10, 0, 0, 1]);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ipv4Addr {
    octets: [u8; 4],
}

impl Ipv4Addr {
    /// Creates an IPv4 address from four octets.
    ///
    /// # Examples
    ///
    /// ```
    /// use ipnet_rs::Ipv4Addr;
    ///
    /// let addr = Ipv4Addr::new(192, 168, 1, 1);
    /// assert_eq!(addr.to_string(), "192.168.1.1");
    /// ```
    pub fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Self {
            octets: [a, b, c, d],
        }
    }

    /// Returns the four octets of the address.
    ///
    /// # Examples
    ///
    /// ```
    /// use ipnet_rs::Ipv4Addr;
    ///
    /// let addr = Ipv4Addr::new(10, 20, 30, 40);
    /// assert_eq!(addr.octets(), [10, 20, 30, 40]);
    /// ```
    pub fn octets(&self) -> [u8; 4] {
        self.octets
    }

    /// Returns the address as a 32-bit integer in host byte order.
    ///
    /// The first octet becomes the most-significant byte of the result.
    ///
    /// # Examples
    ///
    /// ```
    /// use ipnet_rs::Ipv4Addr;
    ///
    /// let addr = Ipv4Addr::new(192, 168, 1, 1);
    /// assert_eq!(addr.to_bits(), 0xC0A80101);
    /// ```
    pub fn to_bits(&self) -> u32 {
        u32::from_be_bytes(self.octets)
    }

    /// Creates an address from a 32-bit integer in host byte order.
    ///
    /// The most-significant byte becomes the first octet.
    ///
    /// # Examples
    ///
    /// ```
    /// use ipnet_rs::Ipv4Addr;
    ///
    /// let addr = Ipv4Addr::from_bits(0xC0A80101);
    /// assert_eq!(addr, Ipv4Addr::new(192, 168, 1, 1));
    /// ```
    pub fn from_bits(bits: u32) -> Self {
        Self {
            octets: bits.to_be_bytes(),
        }
    }
}

impl fmt::Display for Ipv4Addr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}.{}.{}.{}",
            self.octets[0], self.octets[1], self.octets[2], self.octets[3]
        )
    }
}

impl FromStr for Ipv4Addr {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 4 {
            return Err(ParseError::InvalidOctetCount);
        }

        let mut octets = [0u8; 4];
        for (i, part) in parts.iter().enumerate() {
            octets[i] = part
                .parse::<u8>()
                .map_err(|_| ParseError::InvalidOctet(part.to_string()))?;
        }

        Ok(Self { octets })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_from_octets() {
        let addr = Ipv4Addr::new(192, 168, 1, 1);
        assert_eq!(addr.octets(), [192, 168, 1, 1]);
    }

    #[test]
    fn displays_dotted_quad() {
        let addr = Ipv4Addr::new(192, 168, 1, 1);
        assert_eq!(addr.to_string(), "192.168.1.1");
    }

    #[test]
    fn parses_dotted_quad() {
        let addr: Ipv4Addr = "192.168.1.1".parse().unwrap();
        assert_eq!(addr, Ipv4Addr::new(192, 168, 1, 1));
    }

    #[test]
    fn rejects_wrong_octet_count() {
        let result: Result<Ipv4Addr, _> = "192.168.1".parse();
        assert_eq!(result, Err(ParseError::InvalidOctetCount));
    }

    #[test]
    fn rejects_out_of_range_octet() {
        let result: Result<Ipv4Addr, _> = "192.168.1.256".parse();
        assert!(matches!(result, Err(ParseError::InvalidOctet(_))));
    }

    #[test]
    fn rejects_non_numeric_octet() {
        let result: Result<Ipv4Addr, _> = "192.168.1.abc".parse();
        assert!(matches!(result, Err(ParseError::InvalidOctet(_))));
    }

    #[test]
    fn converts_to_bits() {
        let addr = Ipv4Addr::new(192, 168, 1, 1);
        assert_eq!(addr.to_bits(), 0xC0A80101);
    }

    #[test]
    fn converts_from_bits() {
        let addr = Ipv4Addr::from_bits(0xC0A80101);
        assert_eq!(addr, Ipv4Addr::new(192, 168, 1, 1));
    }

    #[test]
    fn bits_roundtrip() {
        let addr = Ipv4Addr::new(10, 20, 30, 40);
        assert_eq!(Ipv4Addr::from_bits(addr.to_bits()), addr);
    }
}
