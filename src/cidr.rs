use crate::{Ipv4Addr, ParseError};
use std::fmt;
use std::str::FromStr;

/// An iterator over the IPv4 addresses in an [`Ipv4Cidr`].
///
/// Returned by [`Ipv4Cidr::addresses`], [`Ipv4Cidr::hosts`],
/// [`Ipv4Cidr::iter`], and `IntoIterator for &Ipv4Cidr`.
///
/// # Examples
///
/// ```
/// use ipnet_rs::Ipv4Cidr;
///
/// let cidr: Ipv4Cidr = "10.0.0.0/30".parse().unwrap();
/// for addr in &cidr {
///     println!("{addr}");
/// }
/// ```
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

/// An IPv4 CIDR network: an address and a prefix length.
///
/// The prefix length is the number of leading bits that identify the network.
/// `192.168.1.0/24` means the top 24 bits identify the network and the
/// bottom 8 bits identify hosts within it.
///
/// The stored address need not be the network address; methods like
/// [`network`](Self::network) and [`broadcast`](Self::broadcast) compute
/// the canonical addresses from whatever address you passed in.
///
/// # Examples
///
/// ```
/// use ipnet_rs::{Ipv4Addr, Ipv4Cidr};
///
/// let net: Ipv4Cidr = "192.168.1.42/24".parse().unwrap();
/// assert_eq!(net.network(), Ipv4Addr::new(192, 168, 1, 0));
/// assert_eq!(net.broadcast(), Ipv4Addr::new(192, 168, 1, 255));
/// assert_eq!(net.usable_hosts(), 254);
/// ```
impl Ipv4Cidr {
    /// Creates a new CIDR network from an address and prefix length.
    ///
    /// The address does not need to be the network address. To get the
    /// canonical network address, call [`network`](Self::network).
    ///
    /// # Errors
    ///
    /// Returns [`ParseError::InvalidPrefixLength`] if `prefix_len > 32`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ipnet_rs::{Ipv4Addr, Ipv4Cidr};
    ///
    /// let cidr = Ipv4Cidr::new(Ipv4Addr::new(10, 0, 0, 0), 8).unwrap();
    /// assert_eq!(cidr.to_string(), "10.0.0.0/8");
    ///
    /// assert!(Ipv4Cidr::new(Ipv4Addr::new(0, 0, 0, 0), 33).is_err());
    /// ```
    pub fn new(address: Ipv4Addr, prefix_len: u8) -> Result<Self, ParseError> {
        if prefix_len > 32 {
            return Err(ParseError::InvalidPrefixLength(prefix_len.to_string()));
        }
        Ok(Self {
            address,
            prefix_len,
        })
    }

    /// Returns the address this network was constructed with.
    ///
    /// This is *not* necessarily the network address; for that, use
    /// [`network`](Self::network).
    ///
    /// # Examples
    ///
    /// ```
    /// use ipnet_rs::{Ipv4Addr, Ipv4Cidr};
    ///
    /// let cidr: Ipv4Cidr = "192.168.1.42/24".parse().unwrap();
    /// assert_eq!(cidr.address(), Ipv4Addr::new(192, 168, 1, 42));
    /// assert_eq!(cidr.network(), Ipv4Addr::new(192, 168, 1, 0));
    /// ```
    pub fn address(&self) -> Ipv4Addr {
        self.address
    }

    /// Returns the prefix length (the number after the `/`).
    ///
    /// # Examples
    ///
    /// ```
    /// use ipnet_rs::Ipv4Cidr;
    ///
    /// let cidr: Ipv4Cidr = "10.0.0.0/8".parse().unwrap();
    /// assert_eq!(cidr.prefix_len(), 8);
    /// ```
    pub fn prefix_len(&self) -> u8 {
        self.prefix_len
    }

    /// Returns the netmask as a 32-bit integer. Used internally for
    /// bit arithmetic.
    fn netmask_bits(&self) -> u32 {
        if self.prefix_len == 0 {
            0
        } else {
            !0u32 << (32 - self.prefix_len)
        }
    }

    /// Returns the netmask as an [`Ipv4Addr`].
    ///
    /// For a `/24` this is `255.255.255.0`. For a `/0` it is `0.0.0.0`.
    /// For a `/32` it is `255.255.255.255`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ipnet_rs::{Ipv4Addr, Ipv4Cidr};
    ///
    /// let cidr: Ipv4Cidr = "192.168.1.0/24".parse().unwrap();
    /// assert_eq!(cidr.netmask(), Ipv4Addr::new(255, 255, 255, 0));
    /// ```
    pub fn netmask(&self) -> Ipv4Addr {
        Ipv4Addr::from_bits(self.netmask_bits())
    }

    /// Returns the network address (the address with all host bits cleared).
    ///
    /// For `192.168.1.42/24`, the network address is `192.168.1.0`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ipnet_rs::{Ipv4Addr, Ipv4Cidr};
    ///
    /// let cidr: Ipv4Cidr = "192.168.1.42/24".parse().unwrap();
    /// assert_eq!(cidr.network(), Ipv4Addr::new(192, 168, 1, 0));
    /// ```
    pub fn network(&self) -> Ipv4Addr {
        Ipv4Addr::from_bits(self.address.to_bits() & self.netmask_bits())
    }

    /// Returns the broadcast address (the address with all host bits set).
    ///
    /// For `192.168.1.0/24`, the broadcast address is `192.168.1.255`.
    /// For a `/32`, the broadcast equals the network address.
    ///
    /// # Examples
    ///
    /// ```
    /// use ipnet_rs::{Ipv4Addr, Ipv4Cidr};
    ///
    /// let cidr: Ipv4Cidr = "192.168.1.0/24".parse().unwrap();
    /// assert_eq!(cidr.broadcast(), Ipv4Addr::new(192, 168, 1, 255));
    /// ```
    pub fn broadcast(&self) -> Ipv4Addr {
        Ipv4Addr::from_bits(self.address.to_bits() | !self.netmask_bits())
    }

    /// Returns the total number of addresses in the network, including
    /// the network and broadcast addresses.
    ///
    /// For a `/24` this is 256. For a `/0` it is 2³² (4,294,967,296),
    /// which is why the return type is `u64`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ipnet_rs::Ipv4Cidr;
    ///
    /// let cidr: Ipv4Cidr = "192.168.1.0/24".parse().unwrap();
    /// assert_eq!(cidr.total_addresses(), 256);
    ///
    /// let cidr: Ipv4Cidr = "10.0.0.0/8".parse().unwrap();
    /// assert_eq!(cidr.total_addresses(), 16_777_216);
    /// ```
    pub fn total_addresses(&self) -> u64 {
        1u64 << (32 - self.prefix_len)
    }

    /// Returns the number of usable host addresses, excluding the network
    /// and broadcast addresses.
    ///
    /// Special cases per RFC 3021 and RFC 3627:
    /// - For `/31`, returns 2 (both addresses are usable on point-to-point links).
    /// - For `/32`, returns 1 (a host route to a single address).
    ///
    /// # Examples
    ///
    /// ```
    /// use ipnet_rs::Ipv4Cidr;
    ///
    /// let cidr: Ipv4Cidr = "192.168.1.0/24".parse().unwrap();
    /// assert_eq!(cidr.usable_hosts(), 254);
    ///
    /// let cidr: Ipv4Cidr = "10.0.0.0/31".parse().unwrap();
    /// assert_eq!(cidr.usable_hosts(), 2);
    ///
    /// let cidr: Ipv4Cidr = "10.0.0.1/32".parse().unwrap();
    /// assert_eq!(cidr.usable_hosts(), 1);
    /// ```
    pub fn usable_hosts(&self) -> u64 {
        match self.prefix_len {
            32 => 1,
            31 => 2,
            _ => self.total_addresses() - 2,
        }
    }

    /// Returns `true` if the given address belongs to this network.
    ///
    /// # Examples
    ///
    /// ```
    /// use ipnet_rs::{Ipv4Addr, Ipv4Cidr};
    ///
    /// let cidr: Ipv4Cidr = "192.168.1.0/24".parse().unwrap();
    /// assert!(cidr.contains(Ipv4Addr::new(192, 168, 1, 42)));
    /// assert!(!cidr.contains(Ipv4Addr::new(192, 168, 2, 0)));
    /// ```
    pub fn contains(&self, addr: Ipv4Addr) -> bool {
        addr.to_bits() & self.netmask_bits() == self.network().to_bits()
    }

    /// Returns an iterator over every address in the network, including
    /// the network and broadcast addresses.
    ///
    /// For large networks this iterator can yield billions of addresses;
    /// combine it with [`Iterator::take`] or [`Iterator::filter`] as needed.
    ///
    /// # Examples
    ///
    /// ```
    /// use ipnet_rs::{Ipv4Addr, Ipv4Cidr};
    ///
    /// let cidr: Ipv4Cidr = "192.168.1.0/30".parse().unwrap();
    /// let all: Vec<Ipv4Addr> = cidr.addresses().collect();
    /// assert_eq!(all.len(), 4);
    /// assert_eq!(all[0], Ipv4Addr::new(192, 168, 1, 0));
    /// assert_eq!(all[3], Ipv4Addr::new(192, 168, 1, 3));
    /// ```
    pub fn addresses(&self) -> Ipv4CidrIter {
        Ipv4CidrIter {
            current: self.network().to_bits(),
            end: self.broadcast().to_bits(),
            done: false,
        }
    }

    /// Returns an iterator over the usable host addresses, excluding
    /// the network and broadcast addresses.
    ///
    /// For `/31` and `/32` networks, returns all addresses (matching
    /// [`usable_hosts`](Self::usable_hosts) semantics).
    ///
    /// # Examples
    ///
    /// ```
    /// use ipnet_rs::{Ipv4Addr, Ipv4Cidr};
    ///
    /// let cidr: Ipv4Cidr = "192.168.1.0/30".parse().unwrap();
    /// let hosts: Vec<Ipv4Addr> = cidr.hosts().collect();
    /// assert_eq!(hosts, vec![
    ///     Ipv4Addr::new(192, 168, 1, 1),
    ///     Ipv4Addr::new(192, 168, 1, 2),
    /// ]);
    /// ```
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

    /// Same as [`addresses`](Self::addresses). Provided for consistency
    /// with stdlib iteration conventions (`.iter()` on a collection).
    ///
    /// # Examples
    ///
    /// ```
    /// use ipnet_rs::Ipv4Cidr;
    ///
    /// let cidr: Ipv4Cidr = "10.0.0.0/30".parse().unwrap();
    /// assert_eq!(cidr.iter().count(), 4);
    /// ```
    pub fn iter(&self) -> Ipv4CidrIter {
        self.addresses()
    }

    /// Splits this network into smaller subnets at the given prefix length.
    ///
    /// Returns an empty `Vec` if the new prefix length is not strictly
    /// larger than the current one, or if it is greater than 32. (You
    /// cannot split a network into bigger networks.)
    ///
    /// # Examples
    ///
    /// ```
    /// use ipnet_rs::Ipv4Cidr;
    ///
    /// let cidr: Ipv4Cidr = "192.168.1.0/24".parse().unwrap();
    /// let subnets = cidr.split(26);
    /// assert_eq!(subnets.len(), 4);
    /// assert_eq!(subnets[0].to_string(), "192.168.1.0/26");
    /// assert_eq!(subnets[3].to_string(), "192.168.1.192/26");
    /// ```
    pub fn split(&self, new_prefix_len: u8) -> Vec<Ipv4Cidr> {
        if new_prefix_len <= self.prefix_len || new_prefix_len > 32 {
            return Vec::new();
        }

        let count = 1u64 << (new_prefix_len - self.prefix_len);
        let step = 1u64 << (32 - new_prefix_len);
        let start = self.network().to_bits() as u64;

        (0..count)
            .map(|i| {
                let addr = Ipv4Addr::from_bits((start + i * step) as u32);
                Ipv4Cidr::new(addr, new_prefix_len).unwrap()
            })
            .collect()
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

    #[test]
    fn splits_into_smaller_subnets() {
        let cidr: Ipv4Cidr = "192.168.1.0/24".parse().unwrap();
        let subnets = cidr.split(26);
        assert_eq!(subnets.len(), 4);
        assert_eq!(subnets[0].to_string(), "192.168.1.0/26");
        assert_eq!(subnets[1].to_string(), "192.168.1.64/26");
        assert_eq!(subnets[2].to_string(), "192.168.1.128/26");
        assert_eq!(subnets[3].to_string(), "192.168.1.192/26");
    }

    #[test]
    fn splits_into_halves() {
        let cidr: Ipv4Cidr = "10.0.0.0/8".parse().unwrap();
        let subnets = cidr.split(9);
        assert_eq!(subnets.len(), 2);
        assert_eq!(subnets[0].to_string(), "10.0.0.0/9");
        assert_eq!(subnets[1].to_string(), "10.128.0.0/9");
    }

    #[test]
    fn split_rejects_invalid_prefix() {
        let cidr: Ipv4Cidr = "192.168.1.0/24".parse().unwrap();
        assert!(cidr.split(24).is_empty()); // same prefix
        assert!(cidr.split(20).is_empty()); // larger network
        assert!(cidr.split(33).is_empty()); // out of range
    }

    #[test]
    fn split_preserves_address_space() {
        let cidr: Ipv4Cidr = "10.0.0.0/16".parse().unwrap();
        let subnets = cidr.split(18);
        let total: u64 = subnets.iter().map(|s| s.total_addresses()).sum();
        assert_eq!(total, cidr.total_addresses());
    }
}
