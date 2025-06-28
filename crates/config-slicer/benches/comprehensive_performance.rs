//! Comprehensive performance benchmarks for config-slicer
//!
//! This benchmark suite tests all major performance-critical operations
//! with various data sizes and configurations to identify bottlenecks.

use config_slicer::{
    api::ConfigSlicerApi,
    parser::{Vendor, core::HierarchicalParser},
    streaming::{StreamingConfig, StreamingProcessor},
};
use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use std::io::Cursor;

/// Generate synthetic Cisco IOS configuration of specified size
fn generate_cisco_config(interface_count: usize, acl_entries: usize) -> String {
    let mut config = String::new();
    config.push_str("!\n! Synthetic Cisco IOS Configuration\n!\n");
    config.push_str("version 15.4\n");
    config.push_str("hostname BENCH-ROUTER\n");
    config.push_str("!\n");

    // Generate interfaces
    for i in 1..=interface_count {
        config.push_str(&format!("interface GigabitEthernet0/{}\n", i));
        config.push_str(&format!(" description Interface_{}_description\n", i));
        config.push_str(&format!(
            " ip address 192.168.{}.1 255.255.255.0\n",
            (i % 254) + 1
        ));
        config.push_str(" duplex full\n");
        config.push_str(" speed 1000\n");
        config.push_str(" no shutdown\n");
        config.push_str("!\n");
    }

    // Generate ACLs
    config.push_str("access-list 100 remark === Benchmark ACL ===\n");
    for i in 1..=acl_entries {
        config.push_str(&format!(
            "access-list 100 permit tcp 192.168.{}.0 0.0.0.255 any eq 80\n",
            (i % 254) + 1
        ));
    }
    config.push_str("!\n");

    // Add routing configuration
    config.push_str("router ospf 1\n");
    config.push_str(" router-id 192.168.1.1\n");
    for i in 1..=10 {
        config.push_str(&format!(" network 192.168.{}.0 0.0.0.255 area 0\n", i));
    }
    config.push_str("!\n");

    config.push_str("end\n");
    config
}

/// Generate synthetic Juniper configuration
fn generate_juniper_config(interface_count: usize) -> String {
    let mut config = String::new();
    config.push_str("system {\n");
    config.push_str("    host-name BENCH-ROUTER;\n");
    config.push_str("    domain-name example.com;\n");
    config.push_str("}\n");

    config.push_str("interfaces {\n");
    for i in 1..=interface_count {
        config.push_str(&format!("    ge-0/0/{} {{\n", i));
        config.push_str(&format!("        description \"Interface {}\";\n", i));
        config.push_str(&format!(
            "        unit 0 {{\n            family inet {{\n                address 192.168.{}.1/24;\n            }}\n        }}\n",
            (i % 254) + 1
        ));
        config.push_str("    }\n");
    }
    config.push_str("}\n");

    config.push_str("protocols {\n");
    config.push_str("    ospf {\n");
    config.push_str("        area 0.0.0.0 {\n");
    for i in 1..=5 {
        config.push_str(&format!("            interface ge-0/0/{}.0;\n", i));
    }
    config.push_str("        }\n");
    config.push_str("    }\n");
    config.push_str("}\n");

    config
}

/// Benchmark parsing performance across different vendors and sizes
fn bench_parsing_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing_performance");

    // Test different configuration sizes
    let sizes = vec![10, 50, 100, 500, 1000];

    for size in sizes {
        let cisco_config = generate_cisco_config(size, size / 10);
        let juniper_config = generate_juniper_config(size);

        // Set throughput for size comparison
        group.throughput(Throughput::Elements(size as u64));

        // Benchmark Cisco parsing
        group.bench_with_input(
            BenchmarkId::new("cisco_ios", size),
            &cisco_config,
            |b, config| {
                let parser = HierarchicalParser::new().unwrap();
                b.iter(|| {
                    let result = parser.parse(black_box(config));
                    black_box(result)
                });
            },
        );

        // Benchmark Juniper parsing
        group.bench_with_input(
            BenchmarkId::new("juniper_junos", size),
            &juniper_config,
            |b, config| {
                let parser = HierarchicalParser::new().unwrap();
                b.iter(|| {
                    let result = parser.parse(black_box(config));
                    black_box(result)
                });
            },
        );

        // Benchmark API parsing (with vendor detection)
        group.bench_with_input(
            BenchmarkId::new("api_with_detection", size),
            &cisco_config,
            |b, config| {
                let api = ConfigSlicerApi::new();
                b.iter(|| {
                    let result = api.parse_config(black_box(config), None);
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark slice extraction performance
fn bench_slicing_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("slicing_performance");

    let config = generate_cisco_config(200, 50);
    let api = ConfigSlicerApi::new();

    // Test basic slice extraction via API
    group.bench_function("api_slice_by_glob", |b| {
        b.iter(|| {
            if let Ok(config_tree) = api.parse_config(&config, Some(Vendor::Cisco)) {
                let result = api.slice_by_glob(black_box(&config_tree), "interface*");
                black_box(result)
            } else {
                Err(config_slicer::Error::MalformedConfig(
                    "Parse failed".to_string(),
                ))
            }
        });
    });

    group.bench_function("api_slice_by_regex", |b| {
        b.iter(|| {
            if let Ok(config_tree) = api.parse_config(&config, Some(Vendor::Cisco)) {
                let result = api.slice_by_regex(black_box(&config_tree), r"interface\s+\S+");
                black_box(result)
            } else {
                Err(config_slicer::Error::MalformedConfig(
                    "Parse failed".to_string(),
                ))
            }
        });
    });

    group.finish();
}

/// Benchmark diff algorithms performance
fn bench_diff_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("diff_performance");

    // Generate base and modified configurations
    let base_config = generate_cisco_config(100, 20);
    let mut modified_config = base_config.clone();
    modified_config.push_str(
        "\ninterface GigabitEthernet0/101\n description New Interface\n no shutdown\n!\n",
    );

    // Simple text comparison benchmark
    group.bench_function("text_comparison", |b| {
        b.iter(|| {
            let base_lines: Vec<&str> = base_config.lines().collect();
            let modified_lines: Vec<&str> = modified_config.lines().collect();
            let differences = base_lines.len().abs_diff(modified_lines.len());
            black_box(differences)
        });
    });

    group.finish();
}

/// Benchmark streaming performance with different configurations
fn bench_streaming_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("streaming_performance");

    // Test basic streaming processing
    group.bench_function("basic_streaming", |b| {
        let large_config = generate_cisco_config(1000, 50);
        b.iter(|| {
            let processor = StreamingProcessor::with_config(StreamingConfig::new());
            let cursor = Cursor::new(large_config.as_bytes());
            let result = processor.process_large_config(black_box(cursor), Some(Vendor::Cisco));
            black_box(result)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_parsing_performance,
    bench_slicing_performance,
    bench_diff_performance,
    bench_streaming_performance
);
criterion_main!(benches);
