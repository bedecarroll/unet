use config_slicer::{parse_match, slice_config};
use std::fs;

#[test]
fn slice_interface_block() {
    let config =
        fs::read_to_string("tests/fixtures/basic_cisco_router_config.txt").expect("fixture");
    let spec = parse_match("interface GigabitEthernet0/0.101").expect("parse");
    let lines = slice_config(&config, &spec).expect("slice");
    let expected = [
        "interface GigabitEthernet0/0.101",
        " description LAN",
        " encapsulation dot1Q 101",
        " ip address 192.168.1.1 255.255.255.0",
        " ip nat inside",
        " ip virtual-reassembly in",
        " ip virtual-reassembly out",
        " ipv6 address prefix-from-COMCAST ::1/64",
        " ipv6 enable",
        " ipv6 nd other-config-flag",
        " ipv6 dhcp server DHCPv6",
    ];
    assert_eq!(lines, expected);
}
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn cli_slice_interface_block() {
    let mut cmd = Command::cargo_bin("config-slicer").expect("binary");
    cmd.args([
        "--match",
        "interface GigabitEthernet0/0.101",
        "--file",
        "tests/fixtures/basic_cisco_router_config.txt",
    ]);
    let expected = [
        "interface GigabitEthernet0/0.101",
        " description LAN",
        " encapsulation dot1Q 101",
        " ip address 192.168.1.1 255.255.255.0",
        " ip nat inside",
        " ip virtual-reassembly in",
        " ip virtual-reassembly out",
        " ipv6 address prefix-from-COMCAST ::1/64",
        " ipv6 enable",
        " ipv6 nd other-config-flag",
        " ipv6 dhcp server DHCPv6",
    ]
    .join("\n")
        + "\n";
    cmd.assert().success().stdout(predicate::eq(expected));
}
