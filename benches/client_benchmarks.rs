//! VPN client performance benchmarks

use criterion::{criterion_group, criterion_main, Criterion};
use rvpnse::{Config, VpnClient};
use std::hint::black_box;
use std::time::Duration;

fn client_creation_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("client_creation");

    let config = Config::default_vpn_gate();

    group.bench_function("create_client", |b| {
        b.iter(|| {
            let client = VpnClient::new(black_box(config.clone())).unwrap();
            black_box(client);
        });
    });

    group.finish();
}

fn client_connection_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("client_connection");
    group.sample_size(10); // Fewer samples for network operations
    group.measurement_time(Duration::from_secs(30));

    let config = Config::default_vpn_gate();

    group.bench_function("resolve_address", |b| {
        let mut client = VpnClient::new(config.clone()).unwrap();
        b.iter(|| {
            // Test address resolution performance
            let result = client.connect(black_box("8.8.8.8"), black_box(53));
            let _ = black_box(result);
            let _ = client.disconnect(); // Clean up
        });
    });

    group.finish();
}

fn client_state_management_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("client_state");

    let config = Config::default_vpn_gate();
    let client = VpnClient::new(config).unwrap();

    group.bench_function("status_check", |b| {
        b.iter(|| {
            let status = black_box(&client).status();
            black_box(status);
        });
    });

    group.bench_function("server_endpoint", |b| {
        b.iter(|| {
            let endpoint = black_box(&client).server_endpoint();
            black_box(endpoint);
        });
    });

    group.finish();
}

fn authentication_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("authentication");

    let config = Config::default_vpn_gate();

    group.bench_function("auth_parameter_validation", |b| {
        let mut client = VpnClient::new(config.clone()).unwrap();
        let _ = client.connect("127.0.0.1", 443); // Mock connection

        b.iter(|| {
            let result = client.authenticate(black_box("testuser"), black_box("testpass"));
            let _ = black_box(result);
        });
    });

    group.finish();
}

fn session_management_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("session_management");

    let config = Config::default_vpn_gate();

    group.bench_function("keepalive_simulation", |b| {
        let mut client = VpnClient::new(config.clone()).unwrap();
        let _ = client.connect("127.0.0.1", 443);
        let _ = client.authenticate("testuser", "testpass");

        b.iter(|| {
            let result = client.send_keepalive();
            let _ = black_box(result);
        });
    });

    group.finish();
}

fn memory_usage_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");

    group.bench_function("client_lifecycle", |b| {
        b.iter(|| {
            let config = Config::default_vpn_gate();
            let mut client = VpnClient::new(black_box(config)).unwrap();
            let _ = client.connect("127.0.0.1", 443);
            let _ = client.authenticate("user", "pass");
            let _ = client.disconnect();
            black_box(client);
        });
    });

    // Test with multiple clients
    group.bench_function("multiple_clients", |b| {
        b.iter(|| {
            let config = Config::default_vpn_gate();
            let mut clients = Vec::with_capacity(10);

            for _ in 0..10 {
                let client = VpnClient::new(config.clone()).unwrap();
                clients.push(client);
            }

            black_box(clients);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    client_creation_benchmark,
    client_connection_benchmark,
    client_state_management_benchmark,
    authentication_benchmark,
    session_management_benchmark,
    memory_usage_benchmark
);
criterion_main!(benches);
