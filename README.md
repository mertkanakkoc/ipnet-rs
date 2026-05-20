# ipnet-rs

A small IPv4 networking library in Rust. Provides types for parsing,
formatting, iterating, and manipulating IPv4 addresses and CIDR networks.

## Quick start

```rust
use ipnet_rs::{Ipv4Addr, Ipv4Cidr};

let net: Ipv4Cidr = "192.168.1.0/24".parse()?;

assert_eq!(net.network(), Ipv4Addr::new(192, 168, 1, 0));
assert_eq!(net.broadcast(), Ipv4Addr::new(192, 168, 1, 255));
assert_eq!(net.usable_hosts(), 254);

for host in net.hosts().take(3) {
    println!("{host}");
}

let subnets = net.split(26);
assert_eq!(subnets.len(), 4);
```

## Features

- Parse and format IPv4 addresses (`192.168.1.1`) and CIDR networks (`192.168.1.0/24`)
- Compute network address, broadcast address, netmask
- Iterate hosts in a subnet
- Check subnet membership
- Split subnets into smaller subnets
- Proper error types via `thiserror`

## Status

Educational project built while learning Rust. For production code,
consider using the [`ipnet`](https://crates.io/crates/ipnet) crate
or stdlib's `std::net::Ipv4Addr` + manual subnet math.
