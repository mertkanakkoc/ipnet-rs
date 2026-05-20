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

    // Returns the netmask as a 32-bit integer.
    fn netmask_bits(&self) -> u32 {
        if self.prefix_len == 0 {
            0
        } else {
            !0u32 << (32 - self.prefix_len)
        }
    }

    // Returns the network as an Ipv4 address
    pub fn netmask(&self) -> Ipv4Addr {
        Ipv4Addr::from_bits(self.netmask_bits())
    }

    /// Returns the network address (the address with all host bits zero).
    pub fn network(&self) -> Ipv4Addr {
        Ipv4Addr::from_bits(self.address.to_bits() & self.netmask_bits())
    }

    /// Returns the broadcast address (the address with all host bits one).
    pub fn broadcast(&self) -> Ipv4Addr {
        Ipv4Addr::from_bits(self.address.to_bits() | !self.netmask_bits())
    }

    /// Returns the total number of addresses in the network, including
    /// network and broadcast addresses.
    pub fn total_addresses(&self) -> u64 {
        1u64 << (32 - self.prefix_len)
    }

    /// Returns the number of host addressses (excluding network and broadcast).
    /// for /31 and /32 networks, returns the total instead of subtracting,
    /// since those have special semantics (point-point and host route).
    pub fn usable_hosts(&self) -> u64 {
        match self.prefix_len {
            32 => 1,
            31 => 2,
            _ => self.total_addresses() - 2,
        }
    }

    /// Returns true if the given address is in this network.
    pub fn contains(&self, addr: Ipv4Addr) -> bool {
        addr.to_bits() & self.netmask_bits() == self.network().to_bits()
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

    #[test]
    fn computes_netmask() {
        let cidr: Ipv4Cidr = "192.168.1.0/24".parse().unwrap();
        assert_eq!(cidr.netmask(), Ipv4Addr::new(255, 255, 255, 0));

        let cidr: Ipv4Cidr = "10.0.0.0/8".parse().unwrap();
        assert_eq!(cidr.netmask(), Ipv4Addr::new(255, 0, 0, 0));

        let cidr: Ipv4Cidr = "0.0.0.0/0".parse().unwrap();
        assert_eq!(cidr.netmask(), Ipv4Addr::new(0, 0, 0, 0));

        let cidr: Ipv4Cidr = "1.2.3.4/32".parse().unwrap();
        assert_eq!(cidr.netmask(), Ipv4Addr::new(255, 255, 255, 255));
    }

    #[test]
    fn computes_network_and_broadcast() {
        let cidr: Ipv4Cidr = "192.168.1.42/24".parse().unwrap();
        assert_eq!(cidr.network(), Ipv4Addr::new(192, 168, 1, 0));
        assert_eq!(cidr.broadcast(), Ipv4Addr::new(192, 168, 1, 255));
    }

    #[test]
    fn computes_host_counts() {
        let cidr: Ipv4Cidr = "192.168.1.0/24".parse().unwrap();
        assert_eq!(cidr.total_addresses(), 256);
        assert_eq!(cidr.usable_hosts(), 254);

        let cidr: Ipv4Cidr = "10.0.0.0/8".parse().unwrap();
        assert_eq!(cidr.total_addresses(), 16_777_216);

        let cidr: Ipv4Cidr = "192.168.1.0/31".parse().unwrap();
        assert_eq!(cidr.usable_hosts(), 2);
    }

    #[test]
    fn contains_address() {
        let cidr: Ipv4Cidr = "192.168.1.0/24".parse().unwrap();
        assert!(cidr.contains(Ipv4Addr::new(192, 168, 1, 0)));
        assert!(cidr.contains(Ipv4Addr::new(192, 168, 1, 42)));
        assert!(cidr.contains(Ipv4Addr::new(192, 168, 1, 255)));
        assert!(!cidr.contains(Ipv4Addr::new(192, 168, 2, 0)));
        assert!(!cidr.contains(Ipv4Addr::new(10, 0, 0, 1)));
    }
}
