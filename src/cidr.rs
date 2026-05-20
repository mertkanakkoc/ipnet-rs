use crate::{Ipv4Addr, ParseError};
use std::fmt;
use std::str::FromStr;

/// An iterator over the addresses in a network
pub struct Ipv4CidrIter {
    current: u32,
    end: u32,
    done: bool,
}

impl Iterator for Ipv4CidrIter {
    type Item = Ipv4Addr;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let addr = Ipv4Addr::from_bits(self.current);
        if self.current == self.end {
            self.done = true;
        } else {
            self.current += 1;
        }

        Some(addr)
    }
}

impl IntoIterator for &Ipv4Cidr {
    type Item = Ipv4Addr;
    type IntoIter = Ipv4CidrIter;

    fn into_iter(self) -> Self::IntoIter {
        self.addresses()
    }
}

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

    /// Returns an iterator over all addresses in the network,
    /// including the network and broadcast addresses.
    pub fn addresses(&self) -> Ipv4CidrIter {
        Ipv4CidrIter {
            current: self.network().to_bits(),
            end: self.broadcast().to_bits(),
            done: false,
        }
    }

    /// Returns an iterator over the usable host addresses,
    /// excluding the network and broadcast addresses.
    /// For /31 and /32 networks, returns all addresses.
    pub fn hosts(&self) -> Ipv4CidrIter {
        match self.prefix_len {
            31 | 32 => self.addresses(),
            _ => Ipv4CidrIter {
                current: self.network().to_bits() + 1,
                end: self.broadcast().to_bits() - 1,
                done: false,
            },
        }
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

    #[test]
    fn iterates_addresses() {
        let cidr: Ipv4Cidr = "192.168.1.0/30".parse().unwrap();
        let addrs: Vec<Ipv4Addr> = cidr.addresses().collect();
        assert_eq!(
            addrs,
            vec![
                Ipv4Addr::new(192, 168, 1, 0),
                Ipv4Addr::new(192, 168, 1, 1),
                Ipv4Addr::new(192, 168, 1, 2),
                Ipv4Addr::new(192, 168, 1, 3),
            ]
        );
    }

    #[test]
    fn iterates_hosts() {
        let cidr: Ipv4Cidr = "192.168.1.0/30".parse().unwrap();
        let hosts: Vec<Ipv4Addr> = cidr.hosts().collect();
        assert_eq!(
            hosts,
            vec![Ipv4Addr::new(192, 168, 1, 1), Ipv4Addr::new(192, 168, 1, 2),]
        );
    }

    #[test]
    fn iterates_single_host_for_32() {
        let cidr: Ipv4Cidr = "10.0.0.1/32".parse().unwrap();
        let hosts: Vec<Ipv4Addr> = cidr.hosts().collect();
        assert_eq!(hosts, vec![Ipv4Addr::new(10, 0, 0, 1)]);
    }

    #[test]
    fn iterates_two_hosts_for_31() {
        let cidr: Ipv4Cidr = "10.0.0.0/31".parse().unwrap();
        let hosts: Vec<Ipv4Addr> = cidr.hosts().collect();
        assert_eq!(
            hosts,
            vec![Ipv4Addr::new(10, 0, 0, 0), Ipv4Addr::new(10, 0, 0, 1),]
        );
    }

    #[test]
    fn for_loop_works() {
        let cidr: Ipv4Cidr = "192.168.1.0/30".parse().unwrap();
        let mut count = 0;
        for _addr in &cidr {
            count += 1;
        }
        assert_eq!(count, 4);
    }

    #[test]
    fn iterator_combinators_work() {
        let cidr: Ipv4Cidr = "10.0.0.0/24".parse().unwrap();

        // Filter to addresses ending in even number, take 5.
        let evens: Vec<Ipv4Addr> = cidr
            .hosts()
            .filter(|a| a.octets()[3] % 2 == 0)
            .take(5)
            .collect();
        assert_eq!(evens.len(), 5);
        assert_eq!(evens[0], Ipv4Addr::new(10, 0, 0, 2));
        assert_eq!(evens[4], Ipv4Addr::new(10, 0, 0, 10));
    }
}
