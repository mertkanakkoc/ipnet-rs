use ipnet_rs::{Ipv4Addr, Ipv4Cidr};

#[test]
fn typical_workflow() {
    // Parse a network the user provided.
    let net: Ipv4Cidr = "10.0.0.0/24".parse().unwrap();

    // Confirm key properties.
    assert_eq!(net.network(), Ipv4Addr::new(10, 0, 0, 0));
    assert_eq!(net.broadcast(), Ipv4Addr::new(10, 0, 0, 255));
    assert_eq!(net.usable_hosts(), 254);

    // Iterate hosts and check the first few.
    let first_three: Vec<Ipv4Addr> = net.hosts().take(3).collect();
    assert_eq!(
        first_three,
        vec![
            Ipv4Addr::new(10, 0, 0, 1),
            Ipv4Addr::new(10, 0, 0, 2),
            Ipv4Addr::new(10, 0, 0, 3),
        ]
    );

    // Split into smaller pieces for VLAN assignment.
    let vlans = net.split(26);
    assert_eq!(vlans.len(), 4);

    // Each VLAN has 62 usable hosts (64 - 2 for network/broadcast).
    for vlan in &vlans {
        assert_eq!(vlan.usable_hosts(), 62);
        assert!(net.contains(vlan.network()));
        assert!(net.contains(vlan.broadcast()));
    }
}

#[test]
fn equality_and_hashing() {
    use std::collections::HashSet;

    let a: Ipv4Cidr = "192.168.1.0/24".parse().unwrap();
    let b: Ipv4Cidr = "192.168.1.0/24".parse().unwrap();
    assert_eq!(a, b);

    let mut set = HashSet::new();
    set.insert(a);
    assert!(set.contains(&b));
}
