//! Memory profiling benchmarks for config-slicer
//!
//! This benchmark focuses specifically on memory usage patterns and optimization
//! to identify memory-intensive operations and potential optimizations.

use config_slicer::{
    api::ConfigSlicerApi,
    parser::{Vendor, core::HierarchicalParser},
    streaming::{MemoryMonitor, StreamingConfig, StreamingProcessor},
};
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use std::io::Cursor;

/// Generate large configuration for memory testing
fn generate_large_config(size_multiplier: usize) -> String {
    let mut config = String::with_capacity(size_multiplier * 1024); // Pre-allocate

    config.push_str("!\n! Large configuration for memory testing\n!\n");
    config.push_str("version 15.4\n");
    config.push_str("hostname MEMORY-TEST-ROUTER\n");
    config.push_str("!\n");

    // Generate many interfaces with complex configuration
    for i in 1..=(size_multiplier * 10) {
        config.push_str(&format!("interface GigabitEthernet0/{i}\n"));
        config.push_str(&format!(
            " description Interface_{i}_with_long_description_for_memory_testing\n"
        ));
        config.push_str(&format!(
            " ip address 192.168.{}.1 255.255.255.0\n",
            (i % 254) + 1
        ));
        config.push_str(" duplex full\n");
        config.push_str(" speed 1000\n");
        config.push_str(" no shutdown\n");
        config.push_str(" storm-control broadcast level 10.00\n");
        config.push_str(" storm-control multicast level 10.00\n");
        config.push_str(" spanning-tree portfast\n");
        config.push_str(" spanning-tree bpduguard enable\n");
        config.push_str("!\n");
    }

    // Generate large ACL
    config.push_str("ip access-list extended MEMORY_TEST_ACL\n");
    for i in 1..=(size_multiplier * 50) {
        config.push_str(&format!(
            " permit tcp 192.168.{}.0 0.0.0.255 any eq 80\n",
            (i % 254) + 1
        ));
        config.push_str(&format!(
            " permit tcp 192.168.{}.0 0.0.0.255 any eq 443\n",
            (i % 254) + 1
        ));
    }
    config.push_str("!\n");

    // Generate routing configuration
    config.push_str("router bgp 65001\n");
    config.push_str(" bgp router-id 192.168.1.1\n");
    for i in 1..=(size_multiplier * 5) {
        config.push_str(&format!(" network 192.168.{i}.0 mask 255.255.255.0\n"));
    }
    config.push_str("!\n");

    config.push_str("end\n");
    config
}

/// Memory usage tracking utility
struct MemoryTracker {
    start_usage: usize,
    peak_usage: usize,
}

impl MemoryTracker {
    fn new() -> Self {
        Self {
            start_usage: get_memory_usage(),
            peak_usage: 0,
        }
    }

    fn update_peak(&mut self) {
        let current = get_memory_usage();
        self.peak_usage = self.peak_usage.max(current);
    }

    fn get_delta(&self) -> usize {
        self.peak_usage.saturating_sub(self.start_usage)
    }
}

/// Get current memory usage (simplified - in real implementation would use more sophisticated method)
fn get_memory_usage() -> usize {
    // This is a simplified placeholder - real implementation would query system memory
    std::thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(1)
        * 1024
        * 1024
}

/// Benchmark memory usage during parsing
fn bench_parsing_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing_memory_usage");

    let sizes = vec![1, 5, 10, 20];

    for size in sizes {
        let config = generate_large_config(size);

        group.bench_with_input(
            BenchmarkId::new("hierarchical_parser", size),
            &config,
            |b, config| {
                b.iter_custom(|iters| {
                    let mut total_duration = std::time::Duration::new(0, 0);
                    let mut tracker = MemoryTracker::new();

                    for _i in 0..iters {
                        let start = std::time::Instant::now();

                        let parser = HierarchicalParser::new().unwrap();
                        let _ = parser.parse(black_box(config));

                        total_duration += start.elapsed();
                        tracker.update_peak();
                    }

                    // Log memory usage
                    let memory_delta = tracker.get_delta();
                    println!("Memory delta for size {size}: {memory_delta} bytes");

                    total_duration
                });
            },
        );

        // Compare with API parsing
        group.bench_with_input(
            BenchmarkId::new("api_parser", size),
            &config,
            |b, config| {
                b.iter_custom(|iters| {
                    let mut total_duration = std::time::Duration::new(0, 0);
                    let mut tracker = MemoryTracker::new();

                    for _i in 0..iters {
                        let start = std::time::Instant::now();

                        let api = ConfigSlicerApi::new();
                        let _ = api.parse_config(black_box(config), Some(Vendor::Cisco));

                        total_duration += start.elapsed();
                        tracker.update_peak();
                    }

                    let memory_delta = tracker.get_delta();
                    println!("API Memory delta for size {size}: {memory_delta} bytes");

                    total_duration
                });
            },
        );
    }

    group.finish();
}

/// Benchmark streaming memory efficiency
fn bench_streaming_memory_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("streaming_memory_efficiency");

    let large_config = generate_large_config(50); // Very large config

    // Test different streaming configurations
    let streaming_configs = vec![
        (
            "small_memory_limit",
            StreamingConfig::new()
                .with_memory_limit(1024 * 1024) // 1MB limit
                .with_chunk_size(100)
                .with_aggressive_cleanup(true),
        ),
        (
            "medium_memory_limit",
            StreamingConfig::new()
                .with_memory_limit(5 * 1024 * 1024) // 5MB limit
                .with_chunk_size(500)
                .with_aggressive_cleanup(false),
        ),
        (
            "large_memory_limit",
            StreamingConfig::new()
                .with_memory_limit(20 * 1024 * 1024) // 20MB limit
                .with_chunk_size(2000)
                .with_aggressive_cleanup(false),
        ),
    ];

    for (name, config) in streaming_configs {
        group.bench_with_input(
            BenchmarkId::new("streaming_processor", name),
            &large_config,
            |b, config_text| {
                b.iter_custom(|iters| {
                    let mut total_duration = std::time::Duration::new(0, 0);
                    let mut tracker = MemoryTracker::new();

                    for _i in 0..iters {
                        let start = std::time::Instant::now();

                        let processor = StreamingProcessor::with_config(config.clone());
                        let cursor = Cursor::new(config_text.as_bytes());
                        let _ =
                            processor.process_large_config(black_box(cursor), Some(Vendor::Cisco));

                        total_duration += start.elapsed();
                        tracker.update_peak();
                    }

                    let memory_delta = tracker.get_delta();
                    println!("Streaming memory delta for {name}: {memory_delta} bytes");

                    total_duration
                });
            },
        );
    }

    group.finish();
}

/// Benchmark memory monitor effectiveness
fn bench_memory_monitor(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_monitor");

    let sizes = vec![1024, 4096, 16384, 65536]; // Different allocation sizes

    for size in sizes {
        group.bench_with_input(
            BenchmarkId::new("memory_monitor_tracking", size),
            &size,
            |b, &allocation_size| {
                b.iter(|| {
                    let mut monitor = MemoryMonitor::new(100 * 1024 * 1024); // 100MB limit

                    // Simulate memory allocations
                    for i in 0..100 {
                        if monitor.allocate(allocation_size).is_err() {
                            // Handle memory limit reached
                            monitor.deallocate(allocation_size * (i / 2)); // Deallocate some memory
                        }
                    }

                    let peak = monitor.peak_usage();
                    let current = monitor.current_usage();
                    black_box((peak, current));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark memory optimization techniques
fn bench_memory_optimization_techniques(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_optimization");

    let config = generate_large_config(10);

    // Benchmark with different memory management strategies
    group.bench_function("standard_parsing", |b| {
        b.iter(|| {
            let parser = HierarchicalParser::new().unwrap();
            let result = parser.parse(black_box(&config));
            black_box(result)
        });
    });

    group.bench_function("streaming_with_cleanup", |b| {
        b.iter(|| {
            let streaming_config = StreamingConfig::new()
                .with_aggressive_cleanup(true)
                .with_memory_limit(10 * 1024 * 1024);

            let processor = StreamingProcessor::with_config(streaming_config);
            let cursor = Cursor::new(config.as_bytes());
            let result = processor.process_large_config(black_box(cursor), Some(Vendor::Cisco));
            black_box(result)
        });
    });

    group.bench_function("chunked_processing", |b| {
        b.iter(|| {
            let streaming_config = StreamingConfig::new()
                .with_chunk_size(200)
                .with_buffer_size(4096);

            let processor = StreamingProcessor::with_config(streaming_config);
            let cursor = Cursor::new(config.as_bytes());
            let result = processor.process_chunks(
                std::io::BufReader::new(cursor),
                |_chunk| Ok(()),
                Some(Vendor::Cisco),
            );
            black_box(result)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_parsing_memory_usage,
    bench_streaming_memory_efficiency,
    bench_memory_monitor,
    bench_memory_optimization_techniques
);
criterion_main!(benches);
