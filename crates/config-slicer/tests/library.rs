use config_slicer::{Vendor, diff_text, parse_match, slice_text};

#[test]
fn parse_match_rejects_empty_expression() {
    let error = parse_match("").unwrap_err();

    assert_eq!(
        error.to_string(),
        "invalid match expression: match expression cannot be empty"
    );
}

#[test]
fn parse_match_rejects_empty_levels() {
    let error = parse_match("system||||ntp").unwrap_err();

    assert_eq!(
        error.to_string(),
        "invalid match expression: match expression cannot contain empty levels"
    );
}

#[test]
fn slice_text_filters_flat_config_by_match_expression() {
    let spec = parse_match("system||ntp").unwrap();
    let config = concat!(
        "set system ntp server 192.0.2.1\n",
        "set system ntp source-address 192.0.2.10\n",
        "set interfaces ge-0/0/0 disable\n",
    );

    let lines = slice_text(config, &spec, Vendor::Flat);

    assert_eq!(
        lines,
        vec![
            "set system ntp server 192.0.2.1".to_string(),
            "set system ntp source-address 192.0.2.10".to_string(),
        ]
    );
}

#[test]
fn slice_text_autodetects_vendor_when_requested() {
    let spec = parse_match("system||services").unwrap();
    let config = concat!(
        "system {\n",
        "    services {\n",
        "        ssh;\n",
        "    }\n",
        "}\n",
    );

    let lines = slice_text(config, &spec, Vendor::Autodetect);

    assert_eq!(
        lines,
        vec![
            "    services {".to_string(),
            "        ssh;".to_string(),
            "    }".to_string(),
        ]
    );
}

#[test]
fn slice_text_keeps_matching_junos_block() {
    let spec = parse_match("system||services").unwrap();
    let config = concat!(
        "system {\n",
        "    services {\n",
        "        ssh;\n",
        "    }\n",
        "    host-name edge-1;\n",
        "}\n",
    );

    let lines = slice_text(config, &spec, Vendor::Junos);

    assert_eq!(
        lines,
        vec![
            "    services {".to_string(),
            "        ssh;".to_string(),
            "    }".to_string(),
        ]
    );
}

#[test]
fn diff_text_ignores_unmatched_lines() {
    let spec = parse_match("system||ntp").unwrap();
    let source = concat!(
        "set system ntp server 192.0.2.1\n",
        "set interfaces ge-0/0/0 description old\n",
    );
    let target = concat!(
        "set system ntp server 192.0.2.2\n",
        "set interfaces ge-0/0/0 description new\n",
    );

    let diff = diff_text(source, target, &spec, Vendor::Flat);

    assert!(diff.contains("-set system ntp server 192.0.2.1"));
    assert!(diff.contains("+set system ntp server 192.0.2.2"));
    assert!(!diff.contains("interfaces ge-0/0/0"));
}

#[test]
fn diff_text_returns_empty_string_when_slices_match() {
    let spec = parse_match("system||ntp").unwrap();
    let source = concat!(
        "set system ntp server 192.0.2.1\n",
        "set interfaces ge-0/0/0 description old\n",
    );
    let target = concat!(
        "set system ntp server 192.0.2.1\n",
        "set interfaces ge-0/0/0 description new\n",
    );

    let diff = diff_text(source, target, &spec, Vendor::Flat);

    assert!(diff.is_empty());
}
