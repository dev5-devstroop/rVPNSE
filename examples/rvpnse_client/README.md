# rVPNSE Client Examples

This directory contains example applications that demonstrate how to use the `rvpnse` static library for SoftEther SSL-VPN connections.

## Examples Included

### 1. rVPNSE Client (`src/main.rs`)
Comprehensive Rust example demonstrating:
- Configuration loading and validation
- Protocol-level connection to SoftEther VPN servers
- Authentication and session management with keepalives
- Tunnel interface creation (demonstration mode)
- Public IP detection and routing verification
- Complete workflow from connection to cleanup

**Usage:**
```bash
cargo run
```

### 2. C FFI Client (`client_connection.c`)
C language example showing:
- Using the library from C applications
- Configuration parsing via FFI
- Connection and authentication
- Error handling
- Status checking

**Compile and run:**
```bash
# First build the Rust library
cd ../..
cargo build --release

# Then compile the C client
cd examples/rvpnse_client
gcc -o test_vpngate_c client_connection.c -L../../target/release -lrvpnse -lpthread -ldl -lm
./test_vpngate_c
```

## Usage

### Basic Client

```bash
# Basic usage with default config
cargo run

# Specify custom config file
cargo run -- --config custom_config.toml

# Override server hostname
cargo run -- --server vpn123456789.opengw.net

# Set custom timeout and enable verbose logging
cargo run -- --config config.toml --timeout 45 --verbose

# Show help
cargo run -- --help
```

**Command Line Options:**
- `--config <PATH>`: Path to configuration file (default: `config.toml`)
- `--server <HOSTNAME>`: Override server hostname from config
- `--timeout <SECONDS>`: Connection timeout in seconds (default: 30)
- `--verbose`: Enable verbose debug logging

### Quick Start

```bash
# Run with default configuration
cargo run

# Custom configuration and settings
cargo run -- --config vpn_gate.toml --timeout 60

# Skip tunnel demonstration (faster testing)
cargo run -- --skip-tunnel

# Custom tunnel duration
cargo run -- --tunnel-duration 30

# Verbose output for debugging
cargo run -- --verbose

# Override server for testing
cargo run -- --server vpn123456789.opengw.net --timeout 45
```

**Command Line Options:**
- `--config <PATH>`: Path to configuration file (default: `config.toml`)
- `--server <HOSTNAME>`: Override server hostname from config
- `--timeout <SECONDS>`: Connection timeout in seconds (default: 30)
- `--tunnel-duration <SECONDS>`: How long to maintain tunnel demo (default: 10)
- `--skip-tunnel`: Skip tunnel interface demonstration
- `--verbose`: Enable verbose debug logging

### C FFI Client

```bash
# Build the Rust library first
cd ../..
cargo build --release

# Compile and run the C client
cd examples/rvpnse_client
gcc -o test_vpngate_c client_connection.c -L../../target/release -lrvpnse -lpthread -ldl -lm
./test_vpngate_c
```

## Configuration

The examples use `config.toml` for VPN settings:

```toml
[server]
hostname = "public-vpn-247.opengw.net"
port = 443
hub = "VPNGATE"
use_ssl = true
verify_certificate = false
timeout = 30
keepalive_interval = 50

[auth]
method = "password"
username = "vpn"
password = "vpn"

[network]
auto_route = false
dns_override = false
dns_servers = []
mtu = 1500
custom_routes = []
exclude_routes = []

[logging]
level = "info"
file_logging = false
```

## What These Examples Demonstrate

### ✅ Library Capabilities
- **SoftEther SSL-VPN Protocol**: Complete implementation of the SoftEther protocol
- **Authentication**: Username/password and certificate-based authentication
- **Session Management**: Keepalive packets and session state management
- **Configuration**: TOML-based configuration with validation
- **Error Handling**: Comprehensive error reporting and recovery
- **Cross-Platform**: Works on macOS, Linux, and Windows
- **FFI Interface**: C-compatible API for integration with other languages

### ⚠️ Important Notes
These examples provide **protocol-level connectivity only**. The library handles:
- SoftEther SSL-VPN protocol communication
- TLS/SSL handshake and encryption
- Authentication and session management
- Configuration parsing and validation

### 🔧 What Your Application Must Implement
For full VPN functionality, you need to implement:
- **TUN/TAP Interface Creation**: Platform-specific virtual network interfaces
- **Routing Management**: Configure system routing tables to route traffic through VPN
- **DNS Configuration**: Override system DNS settings to use VPN DNS servers
- **Platform Permissions**: Handle VPN privileges and entitlements (requires admin/root)
- **Packet Forwarding**: Forward packets between VPN interface and the protocol layer

## Platform Integration Examples

See the main project documentation for platform-specific integration guides:
- iOS with NetworkExtension
- Android with VpnService
- Flutter with Dart FFI
- Windows with WinTUN
- macOS with utun interfaces
- Linux with TUN/TAP

## Testing Different Servers

To test with your own SoftEther server, modify `config.toml`:

```toml
[server]
hostname = "your-vpn-server.com"
port = 443
hub = "YOUR_HUB"
# ... rest of configuration
```

## Troubleshooting

**Connection failures are expected** in these examples because:
1. VPN Gate servers rotate frequently
2. The library provides protocol implementation only
3. No actual traffic routing is performed

**If you see "Protocol connection successful"**, the library is working correctly!

## Building

```bash
# Build the Rust library
cargo build --release

# Build and run Rust example
cargo run

# Build C example
gcc -o test_vpngate_c client_connection.c -L../../target/release -lrvpnse -lpthread -ldl -lm
```

## Next Steps

1. **Choose your target platform** (iOS, Android, Flutter, etc.)
2. **Follow the integration guide** in the main project documentation
3. **Implement platform-specific networking** (TUN/TAP, routing, DNS)
4. **Handle platform permissions** (VPN entitlements, admin privileges)
5. **Integrate the library** using the patterns shown in these examples
