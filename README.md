# rVPNSE - Rust VPN SoftEther Library

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/github/actions/workflow/status/rVPNSE/rVPNSE/release.yml?branch=main)](https://github.com/rVPNSE/rVPNSE/actions)
[![Documentation](https://img.shields.io/badge/docs-📖%20available-brightgreen.svg)](docs/README.md)

**Rust library for SoftEther VPN protocol implementation with C FFI interface.**

rVPNSE provides a robust, cross-platform foundation for building VPN applications with SoftEther protocol support. Perfect for integration into mobile apps, desktop applications, embedded devices and enterprise solutions.

## ✨ Key Features

- 🦀 **Production-ready Rust** - Zero warnings, comprehensive testing, strict quality standards
- 🌍 **Cross-platform** - Windows, macOS, Linux, Android, iOS support
- 🔒 **Secure by default** - TLS encryption, certificate validation, secure session management
- 🚀 **High performance** - Async/await, zero-copy operations, optimized networking
- 🔧 **Easy integration** - C FFI interface for seamless language interop
- 📱 **Mobile-optimized** - Battery-efficient, network-aware implementations
- ⚡ **Advanced networking** - Direct IP connections, clustering support, connection pooling
- 🎯 **SoftEther clustering** - Full support for clustered SoftEther VPN servers
- 🛡️ **Robust authentication** - Password, certificate, and anonymous authentication methods

## 🚀 Quick Start

### 1. Build the Library

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/rVPNSE/rVPNSE.git
cd rVPNSE
cargo build --release

# Run the client
cargo run --bin rvpnse-client
```

### 2. Basic Usage

```c
#include "rvpnse.h"

int main() {
    // Create configuration
    struct VpnConfig* config = rvpnse_config_new();
    rvpnse_config_load_from_file(config, "config.toml");
    
    // Create and connect client
    struct VpnClient* client = rvpnse_client_new(config);
    if (rvpnse_client_connect(client) == 0) {
        printf("✅ Connected to VPN!\n");
        
        // Your application logic here
        
        rvpnse_client_disconnect(client);
    }
    
    // Cleanup
    rvpnse_client_free(client);
    rvpnse_config_free(config);
    return 0;
}
```

### 3. Configuration

Create a `config.toml` configuration file:

```toml
[server]
# Server IP address (mandatory)
address = "62.24.65.211"
# Server hostname for Host header (optional) 
hostname = "worxvpn.662.cloud"
port = 992
hub = "VPN"
use_ssl = true
verify_certificate = false
timeout = 30
keepalive_interval = 60

[auth]
method = "password"
username = "your_username"
password = "your_password"
# Optional certificate-based authentication
# client_cert = "path/to/client.crt"
# client_key = "path/to/client.key"
# ca_cert = "path/to/ca.crt"

[connection_limits]
max_connections = 10
enable_pooling = true
pool_size = 5
idle_timeout = 300
max_lifetime = 3600
retry_attempts = 3
retry_delay = 1000
backoff_factor = 2.0
max_retry_delay = 30

[network]
enable_ipv6 = false
user_agent = "rVPNSE/0.1.0"
enable_http2 = true
tcp_keepalive = true
tcp_nodelay = true
# bind_address = "192.168.1.100"
# proxy_url = "http://proxy.example.com:8080"

[logging]
level = "info"
colored = true
json_format = false
# file = "rvpnse.log"
```

## 📦 Integration

rVPNSE supports integration with multiple platforms and languages:

| Platform | Language | Guide |
|----------|----------|-------|
| **iOS** | Swift | [iOS Integration](docs/integration/ios.md) |
| **Android** | Kotlin/Java | [Android Integration](docs/integration/android.md) |
| **Flutter** | Dart | [Flutter Integration](docs/integration/flutter.md) |
| **Desktop** | C/C++ | [C/C++ Integration](docs/integration/c-cpp.md) |
| **.NET** | C# | [.NET Integration](docs/integration/dotnet.md) |
| **Python** | Python | [Python Integration](docs/integration/python.md) |

## 🏗️ Architecture

```mermaid
graph TB
    subgraph "Application Layer"
        A[Mobile Apps]
        B[Desktop Apps] 
        C[Web Services]
        D[Embedded Devices]
    end
    
    subgraph "Language Bindings"
        E[Swift/Kotlin FFI]
        F[C/C++ FFI]
        G[Python/JS FFI]
    end
    
    subgraph "rVPNSE Core"
        H[C FFI Interface]
        I[VPN Client]
        J[Authentication Manager]
        K[SoftEther Protocol]
        L[Connection Pooling]
        M[PACK Protocol Parser]
        N[Clustering Support]
    end
    
    subgraph "Platform Integration"
        O[TUN/TAP Interface]
        P[Network Stack]
        Q[Certificate Store]
    end
    
    subgraph "External"
        R[SoftEther VPN Server]
        S[Clustering RPC]
        T[DNS Servers]
    end
    
    A --> E
    B --> F
    C --> G
    D --> F
    
    E --> H
    F --> H
    G --> H
    
    H --> I
    I --> J
    I --> K
    I --> L
    I --> M
    I --> N
    
    J --> O
    J --> P
    J --> Q
    
    K --> R
    N --> S
    P --> T
    
    style I fill:#99ccff
    style N fill:#ffcc99
    style H fill:#ff9999
```

### 🔐 Connection Management Flow

```mermaid
sequenceDiagram
    participant App as Application
    participant Client as VpnClient
    participant Limits as ConnectionTracker
    participant Server as SoftEther Server
    
    App->>Client: connect(server, port)
    Client->>Limits: can_connect()
    
    alt Connection limits OK
        Limits-->>Client: ✅ Allowed
        Client->>Server: Establish connection
        Server-->>Client: Connected
        Client->>Limits: record_connection()
        Client-->>App: ✅ Connected
    else Limits exceeded
        Limits-->>Client: ❌ Limit exceeded
        Client-->>App: ❌ ConnectionLimitReached
    end
    
    App->>Client: disconnect()
    Client->>Server: Close connection
    Client->>Limits: record_disconnection()
```

## 💠 Build Status

| Platform | Architecture | Status |
|----------|-------------|--------|
| **Windows** | x86_64 | ✅ Passing |
| **macOS** | ARM64, x86_64 | ✅ Passing |
| **Linux** | x86_64 | ✅ Passing |
| **Android** | ARM64, ARMv7, x86_64 | ✅ Passing |
| **iOS** | ARM64, Simulator | ✅ Passing |

## ⚡ Performance Benchmarks

<!-- BENCHMARK_RESULTS_START -->
Performance metrics are automatically updated by our CI/CD pipeline. Run `cargo bench` to generate local benchmarks.

| Benchmark Category | Average Time | Throughput | Status |
|--------------------|--------------|------------|--------|
| Configuration Parsing | - | - | Pending |
| Client Operations | - | - | Pending |
| FFI Interface | - | - | Pending |
| Connection Limits | - | - | Pending |
| Crypto Operations | - | - | Pending |
| Network Throughput | - | - | Pending |
| Memory Management | - | - | Pending |

*Last updated: Pending first benchmark run*
<!-- BENCHMARK_RESULTS_END -->

## 📖 Documentation

- **[📚 Complete Documentation](docs/README.md)** - Comprehensive guides and API reference
- **[🚀 Quick Start Guide](docs/quick-start.md)** - Get up and running in 5 minutes  
- **[🏗️ Build Instructions](docs/build/README.md)** - Detailed build guide for all platforms
- **[🔧 Troubleshooting](docs/troubleshooting.md)** - Common issues and solutions
- **[📋 API Reference](docs/api/c-api.md)** - Complete C API documentation

## 🤝 Contributing

We welcome contributions! See our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/rVPNSE/rVPNSE.git
cd rVPNSE

# Install dependencies
rustup component add clippy rustfmt

# Build the project
cargo build

# Run tests
cargo test

# Run the client with debug logging
RUST_LOG=debug cargo run --bin rvpnse-client

# Check code quality
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check

# Run benchmarks
cargo bench
```

## 📄 License

rVPNSE is licensed under the [Apache License 2.0](LICENSE).

## 🆘 Support

- **📖 Documentation**: [docs/README.md](docs/README.md)
- **🐛 Bug Reports**: [GitHub Issues](https://github.com/rVPNSE/rVPNSE/issues)
- **💬 Discussions**: [GitHub Discussions](https://github.com/rVPNSE/rVPNSE/discussions)
- **ℹ️ General Contact**: hi@devstroop.com
- **🔒 Security**: Email security@devstroop.com for security issues
