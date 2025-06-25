//! Simplified performance benchmarks for config-slicer

use config_slicer::{api::ConfigSlicerApi, parser::Vendor, streaming::StreamingProcessor};
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use std::fs;

/// Load test fixture
fn load_fixture(name: &str) -> String {
    let path = format!("tests/fixtures/{}", name);
    fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to load fixture: {}", path))
}

/// Generate synthetic configuration for scaling tests
fn generate_synthetic_config(interface_count: usize) -> String {
    let mut config = String::new();
    config.push_str("!\n! Generated synthetic configuration\n!\n");

    for i in 1..=interface_count {
        config.push_str(&format!("interface GigabitEthernet0/{}\n", i));
        config.push_str(&format!(" description Interface {}\n", i));
        config.push_str(&format!(
            " ip address 192.168.{}.1 255.255.255.0\n",
            i % 255
        ));
        config.push_str(" duplex full\n");
        config.push_str(" speed 1000\n");
        config.push_str("!\n");
    }

    config
}

/// Benchmark configuration parsing performance
fn bench_parsing_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing_performance");

    // Test with real configuration files
    let real_configs = vec![
        ("cisco_ios_large", "cisco_ios_large.cfg", Vendor::Cisco),
        (
            "juniper_junos_large",
            "juniper_junos_large.cfg",
            Vendor::Juniper,
        ),
        ("arista_eos_large", "arista_eos_large.cfg", Vendor::Arista),
    ];

    let api = ConfigSlicerApi::new();

    for (name, fixture, vendor) in real_configs {
        let config_content = load_fixture(fixture);

        group.bench_with_input(
            BenchmarkId::new("real_config", name),
            &config_content,
            |b, config| {
                b.iter(|| {
                    let result = api.parse_config(black_box(config), black_box(Some(vendor)));
                    black_box(result)
                })
            },
        );
    }

    // Test with synthetic configurations of varying sizes
    let synthetic_sizes = vec![10, 50, 100, 200];

    for interface_count in synthetic_sizes {
        let config_content = generate_synthetic_config(interface_count);

        group.bench_with_input(
            BenchmarkId::new("synthetic_config", interface_count),
            &config_content,
            |b, config| {
                b.iter(|| {
                    let result =
                        api.parse_config(black_box(config), black_box(Some(Vendor::Cisco)));
                    black_box(result)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark slicing performance
fn bench_slicing_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("slicing_performance");

    let config_content = load_fixture("cisco_ios_large.cfg");
    let api = ConfigSlicerApi::new();

    // Parse once for reuse
    let config_tree = api
        .parse_config(&config_content, Some(Vendor::Cisco))
        .unwrap();

    // Test glob patterns
    let glob_patterns = vec!["interface*", "vlan*", "router*"];

    for pattern in glob_patterns {
        group.bench_with_input(
            BenchmarkId::new("glob_pattern", pattern),
            &pattern,
            |b, p| {
                b.iter(|| {
                    let result = api.slice_by_glob(black_box(&config_tree), black_box(p));
                    black_box(result)
                })
            },
        );
    }

    // Test regex patterns
    let regex_patterns = vec![r"interface.*", r"router.*", r"vlan.*"];

    for pattern in regex_patterns {
        group.bench_with_input(
            BenchmarkId::new("regex_pattern", pattern),
            &pattern,
            |b, p| {
                b.iter(|| {
                    let result = api.slice_by_regex(black_box(&config_tree), black_box(p));
                    black_box(result)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark streaming processor performance
fn bench_streaming_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("streaming_performance");

    let large_config = generate_synthetic_config(500); // Large config
    let processor = StreamingProcessor::new();

    group.bench_function("streaming_processing", |b| {
        b.iter(|| {
            let cursor = std::io::Cursor::new(large_config.as_bytes());
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
    bench_streaming_performance
);
criterion_main!(benches);
