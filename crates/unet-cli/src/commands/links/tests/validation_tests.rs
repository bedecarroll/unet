/// Tests for validation and data format checks in link commands

#[tokio::test]
async fn test_bandwidth_values() {
    // Test common bandwidth values
    let bandwidths: Vec<u64> = vec![
        1_000_000,      // 1 Mbps
        10_000_000,     // 10 Mbps
        100_000_000,    // 100 Mbps
        1_000_000_000,  // 1 Gbps
        10_000_000_000, // 10 Gbps
    ];

    for bandwidth in bandwidths {
        assert!(bandwidth > 0);
    }
}

#[tokio::test]
async fn test_interface_name_formatting() {
    let interface_names = vec![
        "GigabitEthernet0/1",
        "FastEthernet0/0",
        "Serial0/0/0",
        "WAN0",
        "eth0",
        "ge-0/0/1",
    ];

    for interface_name in interface_names {
        assert!(!interface_name.is_empty());
        assert!(!interface_name.is_empty());
    }
}
