//! FFI interface performance benchmarks

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::ffi::{CStr, CString};
use std::hint::black_box;
use std::os::raw::{c_char, c_int};

// Mock FFI functions for benchmarking purposes
// In a real implementation, these would be provided by the FFI layer
unsafe fn vpnse_parse_config(
    _config_str: *const c_char,
    _error_msg: *mut c_char,
    _error_msg_len: usize,
) -> c_int {
    // Mock success
    0
}

unsafe fn vpnse_client_new(_config_str: *const c_char) -> *mut std::ffi::c_void {
    // Return a dummy pointer for benchmarking
    std::ptr::dangling_mut::<std::ffi::c_void>()
}

unsafe fn vpnse_client_free(_client: *mut std::ffi::c_void) {
    // Mock cleanup - nothing to do
}

unsafe fn vpnse_client_status(_client: *const std::ffi::c_void) -> c_int {
    // Mock status: connected
    1
}

unsafe fn vpnse_version() -> *const c_char {
    static VERSION: &[u8] = b"1.0.0\0";
    VERSION.as_ptr() as *const c_char
}

const CONFIG_STR: &str = r#"
[server]
hostname = "vpn.example.com"
port = 443
hub = "DEFAULT"
use_ssl = true
verify_certificate = false

[auth]
method = "password"
username = "testuser"
password = "testpass"

[network]
auto_route = false
dns_override = false
mtu = 1500

[crypto]
cipher = "aes-256-cbc"
compression = true

[tunnel]
compression = true
bridge_mode = false
"#;

fn ffi_config_parsing_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("ffi_config_parsing");

    let config_cstr = CString::new(CONFIG_STR).unwrap();
    let mut error_buffer = vec![0u8; 512];

    group.bench_function("parse_config_via_ffi", |b| {
        b.iter(|| {
            let result = unsafe {
                vpnse_parse_config(
                    black_box(config_cstr.as_ptr()),
                    black_box(error_buffer.as_mut_ptr() as *mut c_char),
                    black_box(error_buffer.len()),
                )
            };
            let _ = black_box(result);
        });
    });

    group.finish();
}

fn ffi_client_lifecycle_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("ffi_client_lifecycle");

    let config_cstr = CString::new(CONFIG_STR).unwrap();

    group.bench_function("create_and_free_client", |b| {
        b.iter(|| {
            let client = unsafe { vpnse_client_new(black_box(config_cstr.as_ptr())) };
            black_box(client);

            if !client.is_null() {
                unsafe {
                    vpnse_client_free(client);
                }
            }
        });
    });

    group.bench_function("client_status_check", |b| {
        let client = unsafe { vpnse_client_new(config_cstr.as_ptr()) };

        if !client.is_null() {
            b.iter(|| {
                let status = unsafe { vpnse_client_status(black_box(client)) };
                black_box(status);
            });

            unsafe {
                vpnse_client_free(client);
            }
        }
    });

    group.finish();
}

fn ffi_string_operations_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("ffi_string_operations");

    group.bench_function("version_string_access", |b| {
        b.iter(|| {
            let version_ptr = unsafe { vpnse_version() };
            if !version_ptr.is_null() {
                let version_cstr = unsafe { CStr::from_ptr(version_ptr) };
                let version_str = version_cstr.to_str().unwrap_or("unknown");
                black_box(version_str);
            }
        });
    });

    // Test string conversion overhead
    group.bench_function("cstring_conversion", |b| {
        b.iter(|| {
            let cstring = CString::new(black_box(CONFIG_STR)).unwrap();
            black_box(cstring);
        });
    });

    group.finish();
}

fn ffi_throughput_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("ffi_throughput");

    // Test config parsing throughput
    let medium_config = CONFIG_STR.repeat(5);
    let large_config = CONFIG_STR.repeat(10);

    let config_sizes = vec![
        ("small", CONFIG_STR),
        ("medium", &medium_config),
        ("large", &large_config),
    ];

    for (size_name, config_content) in config_sizes {
        let config_cstr = CString::new(config_content).unwrap();
        let mut error_buffer = vec![0u8; 1024];

        group.throughput(Throughput::Bytes(config_content.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("config_parse_throughput", size_name),
            &config_cstr,
            |b, config_cstr| {
                b.iter(|| {
                    let result = unsafe {
                        vpnse_parse_config(
                            black_box(config_cstr.as_ptr()),
                            black_box(error_buffer.as_mut_ptr() as *mut c_char),
                            black_box(error_buffer.len()),
                        )
                    };
                    let _ = black_box(result);
                });
            },
        );
    }

    group.finish();
}

fn ffi_memory_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("ffi_memory");

    let config_cstr = CString::new(CONFIG_STR).unwrap();

    // Test memory allocation patterns
    group.bench_function("multiple_client_creation", |b| {
        b.iter(|| {
            let mut clients = Vec::with_capacity(10);

            for _ in 0..10 {
                let client = unsafe { vpnse_client_new(config_cstr.as_ptr()) };
                if !client.is_null() {
                    clients.push(client);
                }
            }

            // Clean up
            for client in clients {
                unsafe {
                    vpnse_client_free(client);
                }
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    ffi_config_parsing_benchmark,
    ffi_client_lifecycle_benchmark,
    ffi_string_operations_benchmark,
    ffi_throughput_benchmark,
    ffi_memory_benchmark
);
criterion_main!(benches);
