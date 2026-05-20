use crate::{Ipv4Addr, ParseError};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ipv4Cidr {
    address: Ipv4Addr,
    prefix_len: u8,
}

impl Ipv4Cidr {
    /// Creates a new CIDR network.
    ///
    /// Returns an error if the prefix length is greater than 32.
    pub fn new(address: Ipv4Addr, prefix_len: u8) -> Result<Self, ParseError> {
        if prefix_len > 32 {
            return Err(ParseError::InvalidPrefixLength(prefix_len.to_string()));
        }
        Ok(Self {
            address,
            prefix_len,
        })
    }

    pub fn address(&self) -> Ipv4Addr {
        self.address
    }

    pub fn prefix_len(&self) -> u8 {
        self.prefix_len
    }
}

impl fmt::Display for Ipv4Cidr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.address, self.prefix_len)
    }
}

impl FromStr for Ipv4Cidr {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (addr_part, prefix_part) = s.split_once('/').ok_or(ParseError::MissingPrefix)?;
        let address: Ipv4Addr = addr_part.parse()?;
        let prefix_len: u8 = prefix_part
            .parse()
            .map_err(|_| ParseError::InvalidPrefixLength(prefix_part.to_string()))?;
        Self::new(address, prefix_len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_cidr() {
        let cidr: Ipv4Cidr = "192.168.1.0/24".parse().unwrap();
        assert_eq!(cidr.address(), Ipv4Addr::new(192, 168, 1, 0));
        assert_eq!(cidr.prefix_len(), 24);
    }

    #[test]
    fn displays_cidr() {
        let cidr = Ipv4Cidr::new(Ipv4Addr::new(10, 0, 0, 0), 8).unwrap();
        assert_eq!(cidr.to_string(), "10.0.0.0/8");
    }

    #[test]
    fn rejects_missing_prefix() {
        let result: Result<Ipv4Cidr, _> = "192.168.1.0".parse();
        assert_eq!(result, Err(ParseError::MissingPrefix));
    }

    #[test]
    fn rejects_too_large_prefix() {
        let result: Result<Ipv4Cidr, _> = "192.168.1.0/33".parse();
        assert!(matches!(result, Err(ParseError::InvalidPrefixLength(_))));
    }

    #[test]
    fn rejects_non_numeric_prefix() {
        let result: Result<Ipv4Cidr, _> = "192.168.1.0/abc".parse();
        assert!(matches!(result, Err(ParseError::InvalidPrefixLength(_))));
    }
}
