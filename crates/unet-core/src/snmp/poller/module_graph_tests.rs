use std::path::Path;

#[test]
fn test_poller_module_has_no_orphaned_duplicate_core_file() {
    let poller_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/snmp/poller");

    assert!(
        !poller_dir.join("core_main.rs").exists(),
        "found orphaned duplicate poller core file at {}",
        poller_dir.join("core_main.rs").display()
    );
}
