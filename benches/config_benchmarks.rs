//! Configuration parsing and validation benchmarks

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rvpnse::Config;
use std::hint::black_box;
use std::str::FromStr;

const CONFIG_TOML: &str = r#"
[server]
hostname = "vpn.example.com"
port = 443
hub = "DEFAULT"
use_ssl = true
verify_certificate = false
timeout = 30
keepalive_interval = 50

[auth]
method = "password"
username = "testuser"
password = "testpass"

[network]
auto_route = false
dns_override = false
dns_servers = ["8.8.8.8", "8.8.4.4"]
mtu = 1500
custom_routes = []
exclude_routes = []

[crypto]
cipher = "aes-256-cbc"
compression = true

[tunnel]
compression = true
bridge_mode = false
"#;

const LARGE_CONFIG_TOML: &str = r#"
[server]
hostname = "vpn.example.com"
port = 443
hub = "DEFAULT"
use_ssl = true
verify_certificate = false
timeout = 30
keepalive_interval = 50

[auth]
method = "password"
username = "testuser"
password = "testpass"

[network]
auto_route = false
dns_override = false
dns_servers = ["8.8.8.8", "8.8.4.4", "1.1.1.1", "1.0.0.1", "208.67.222.222", "208.67.220.220"]
mtu = 1500
custom_routes = [
    "192.168.1.0/24",
    "192.168.2.0/24", 
    "10.0.0.0/8",
    "172.16.0.0/12",
    "192.168.0.0/16"
]
exclude_routes = [
    "0.0.0.0/0",
    "169.254.0.0/16"
]

[crypto]
cipher = "aes-256-cbc"
compression = true

[tunnel]
compression = true
bridge_mode = false
"#;

fn config_parsing_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_parsing");

    // Basic config parsing
    group.bench_function("parse_basic_config", |b| {
        b.iter(|| {
            let config = Config::from_str(black_box(CONFIG_TOML)).unwrap();
            black_box(config);
        });
    });

    // Large config parsing
    group.bench_function("parse_large_config", |b| {
        b.iter(|| {
            let config = Config::from_str(black_box(LARGE_CONFIG_TOML)).unwrap();
            black_box(config);
        });
    });

    // Config validation
    group.bench_function("validate_config", |b| {
        let config = Config::from_str(CONFIG_TOML).unwrap();
        b.iter(|| {
            let result = black_box(&config).validate();
            let _ = black_box(result);
        });
    });

    // Default config generation
    group.bench_function("default_vpn_gate_config", |b| {
        b.iter(|| {
            let config = Config::default_vpn_gate();
            black_box(config);
        });
    });

    group.finish();
}

fn config_serialization_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_serialization");

    let config = Config::from_str(CONFIG_TOML).unwrap();

    group.bench_function("serialize_config", |b| {
        b.iter(|| {
            let serialized = toml::to_string(black_box(&config)).unwrap();
            black_box(serialized);
        });
    });

    group.finish();
}

fn config_throughput_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("config_throughput");

    // Test different config sizes
    let configs = vec![("small", CONFIG_TOML), ("large", LARGE_CONFIG_TOML)];

    for (size, config_str) in configs {
        group.throughput(Throughput::Bytes(config_str.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("parse_throughput", size),
            config_str,
            |b, config_str| {
                b.iter(|| {
                    let config = Config::from_str(black_box(config_str)).unwrap();
                    black_box(config);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    config_parsing_benchmark,
    config_serialization_benchmark,
    config_throughput_benchmark
);
criterion_main!(benches);
