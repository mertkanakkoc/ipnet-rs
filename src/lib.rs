//! ipnet-rs: a small IPv4 networking library
//!
//! This crate provides types for working with IPv4 addresses and CIDR networks.

mod cidr;
mod error;
use std::fmt;
use std::str::FromStr;

pub use cidr::Ipv4Cidr;
pub use error::ParseError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ipv4Addr {
    octets: [u8; 4],
}

impl Ipv4Addr {
    /// Creates a new IPv4 address from four octets.
    ///
    /// # Example
    /// ```
    /// use ipnet_rs::Ipv4Addr;
    ///
    /// let addr = Ipv4Addr::new(192, 168, 1, 1);
    /// assert_eq!(addr.to_string(), "192.168.1.1");
    ///
    /// let parsed: Ipv4Addr = "192.168.1.1".parse().unwrap();
    /// assert_eq!(addr, parsed);
    /// ```
    pub fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Self {
            octets: [a, b, c, d],
        }
    }

    /// Returns the four octests of the address.
    pub fn octets(&self) -> [u8; 4] {
        self.octets
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
}
