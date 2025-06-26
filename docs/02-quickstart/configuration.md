# Configuration Reference

Rust VPNSE uses TOML configuration format for all VPN settings. This guide covers all available configuration options.

## 📄 Configuration File Structure

```toml
[server]
# Server connection settings

[connection_limits]
# Connection limits and rate limiting

[auth]
# Authentication settings

[vpn]
# VPN session settings

[advanced]
# Advanced protocol settings
```

## 🔧 Server Configuration

### **Required Settings**

```toml
[server]
# Server hostname or IP address (required)
hostname = "vpn.example.com"

# Server port (required, usually 443 for HTTPS)
port = 443

# Hub name to connect to (required)
hub = "VPN"
```

### **Optional Settings**

```toml
[server]
# Use SSL/TLS connection (default: true)
use_ssl = true

# Verify server certificate (default: true)
verify_certificate = true

# Server certificate fingerprint (optional, for pinning)
cert_fingerprint = "sha256:1234567890abcdef..."

# Connection timeout in seconds (default: 30)
timeout = 30

# DNS resolution timeout in seconds (default: 10)
dns_timeout = 10

# Proxy settings (optional)
proxy_type = "http"           # "http", "socks4", or "socks5"
proxy_host = "proxy.example.com"
proxy_port = 8080
proxy_username = "proxyuser"  # For authenticated proxies
proxy_password = "proxypass"
```

## 🔒 Connection Limits Configuration

Manage connection limits, rate limiting, and retry behavior:

```toml
[connection_limits]
# Maximum concurrent connections (0 = unlimited, default: 0)
max_concurrent_connections = 5

# Maximum connection attempts per minute (0 = unlimited, default: 0)
max_connections_per_minute = 10

# Maximum retry attempts for failed connections (default: 3)
max_retry_attempts = 3

# Delay between retry attempts in seconds (default: 5)
retry_delay = 5

# Connection queue size for pooling (default: 10)
connection_queue_size = 10

# Enable connection pooling - future feature (default: false)
enable_connection_pooling = false

# Pool idle timeout in seconds (default: 300)
pool_idle_timeout = 300

# Pool maximum lifetime in seconds (default: 3600)
pool_max_lifetime = 3600
```

**Use Cases:**
- **Production**: Prevent DoS attacks and resource exhaustion
- **Mobile**: Optimize battery and data usage
- **Enterprise**: Meet compliance and security requirements

See [Connection Limits Documentation](connection-limits.md) for detailed information.

## 🔐 Authentication Configuration

### **Password Authentication**

```toml
[auth]
method = "password"
username = "your-username"
password = "your-password"

# Optional: Save username for reconnection
save_username = true
```

### **Certificate Authentication**

```toml
[auth]
method = "certificate"

# Client certificate file path
cert_file = "/path/to/client.crt"

# Private key file path
key_file = "/path/to/client.key"

# Private key password (if encrypted)
key_password = "keypass"

# Certificate format (default: "pem")
cert_format = "pem"          # "pem" or "der"
```

### **Anonymous Authentication**

```toml
[auth]
method = "anonymous"
# No additional settings required
```

## 🌐 VPN Session Configuration

### **Basic Settings**

```toml
[vpn]
# Virtual adapter name (optional, platform-specific)
adapter_name = "VPNSE"

# MTU size (default: 1500)
mtu = 1500

# Enable compression (default: true)
use_compression = true

# Auto-reconnect on disconnect (default: true)
auto_reconnect = true

# Maximum reconnection attempts (default: unlimited)
max_reconnect_attempts = 10

# Reconnection delay in seconds (default: 5)
reconnect_delay = 5
```

### **Keepalive Settings**

```toml
[vpn]
# Keepalive interval in seconds (default: 60)
keepalive_interval = 60

# Keepalive timeout in seconds (default: 120)
keepalive_timeout = 120

# Send keepalive on idle (default: true)
keepalive_on_idle = true
```

### **Traffic Settings**

```toml
[vpn]
# Upload bandwidth limit in bytes/sec (0 = unlimited)
upload_limit = 0

# Download bandwidth limit in bytes/sec (0 = unlimited)
download_limit = 0

# Enable UDP acceleration (default: true)
use_udp_acceleration = true

# UDP acceleration port range
udp_port_range = "40000-44999"
```

## ⚙️ Advanced Configuration

### **Protocol Settings**

```toml
[advanced]
# Protocol version (default: "4.0")
protocol_version = "4.0"

# SSL/TLS version (default: "auto")
tls_version = "auto"         # "auto", "1.2", "1.3"

# Cipher suites (optional, comma-separated)
cipher_suites = "ECDHE-RSA-AES256-GCM-SHA384,ECDHE-RSA-AES128-GCM-SHA256"

# Connection mode (default: "tcp")
connection_mode = "tcp"      # "tcp" or "udp"

# Half connection mode (default: false)
half_connection = false

# Bridge mode (default: false)
bridge_mode = false
```

### **Logging Settings**

```toml
[advanced]
# Log level (default: "info")
log_level = "info"           # "trace", "debug", "info", "warn", "error"

# Log file path (optional)
log_file = "/tmp/vpnse.log"

# Log to console (default: true)
log_console = true

# Log packet details (default: false, very verbose)
log_packets = false
```

### **Buffer Settings**

```toml
[advanced]
# Send buffer size in bytes (default: 65536)
send_buffer_size = 65536

# Receive buffer size in bytes (default: 65536)
recv_buffer_size = 65536

# Socket send buffer size (default: system default)
socket_send_buffer = 262144

# Socket receive buffer size (default: system default)
socket_recv_buffer = 262144
```

## 📝 Complete Example Configuration

```toml
# Complete Rust VPNSE configuration example
[server]
hostname = "vpn.company.com"
port = 443
hub = "OFFICE_VPN"
use_ssl = true
verify_certificate = true
timeout = 30
dns_timeout = 10

[auth]
method = "password"
username = "john.doe"
password = "secure_password_123"
save_username = true

[vpn]
adapter_name = "CompanyVPN"
mtu = 1500
use_compression = true
auto_reconnect = true
max_reconnect_attempts = 5
reconnect_delay = 10
keepalive_interval = 60
keepalive_timeout = 120
upload_limit = 0
download_limit = 0
use_udp_acceleration = true

[advanced]
protocol_version = "4.0"
tls_version = "1.2"
connection_mode = "tcp"
log_level = "info"
log_console = true
send_buffer_size = 65536
recv_buffer_size = 65536
```

## 🔍 Configuration Validation

### **Using Rust API**

```rust
use rvpnse::config::Config;

let config_str = std::fs::read_to_string("config.toml")?;
let config = Config::from_str(&config_str)?;
println!("Configuration valid: {:?}", config);
```

### **Using C API**

```c
#include "rvpnse.h"

const char* config = "..."; // Your TOML config
char error_msg[256];

int result = vpnse_parse_config(config, error_msg, sizeof(error_msg));
if (result == VPNSE_SUCCESS) {
    printf("Configuration valid\n");
} else {
    printf("Configuration error: %s\n", error_msg);
}
```

### **Using CLI Tool**

```bash
# Validate configuration file
cargo run --example validate_config config.toml

# Test connection with configuration
cargo run --example test_connection config.toml
```

## 🚨 Common Configuration Errors

### **Invalid TOML Syntax**
```
Error: Expected '=' at line 5, column 10
```
**Solution**: Check TOML syntax, ensure proper quotes and structure.

### **Missing Required Fields**
```
Error: Missing required field 'hostname' in [server] section
```
**Solution**: Add all required fields: `hostname`, `port`, `hub`.

### **Invalid Values**
```
Error: Invalid port number: 70000 (must be 1-65535)
```
**Solution**: Check value ranges and data types.

### **Authentication Errors**
```
Error: Certificate file not found: /path/to/cert.pem
```
**Solution**: Verify file paths and permissions for certificate files.

## 🛠️ Configuration Templates

### **Basic Home VPN**
```toml
[server]
hostname = "home.ddns.net"
port = 443
hub = "VPN"

[auth]
method = "password"
username = "family"
password = "homepass123"

[vpn]
auto_reconnect = true
keepalive_interval = 60
```

### **Corporate VPN with Certificate**
```toml
[server]
hostname = "vpn.corporation.com"
port = 443
hub = "CORPORATE"
verify_certificate = true

[auth]
method = "certificate"
cert_file = "/etc/ssl/certs/client.crt"
key_file = "/etc/ssl/private/client.key"

[vpn]
mtu = 1400
use_compression = false
keepalive_interval = 30

[advanced]
log_level = "warn"
tls_version = "1.3"
```

### **Mobile/Roaming Configuration**
```toml
[server]
hostname = "mobile.vpn.com"
port = 443
hub = "MOBILE"
timeout = 15
dns_timeout = 5

[auth]
method = "password"
username = "mobile_user"
password = "mobile_pass"

[vpn]
auto_reconnect = true
max_reconnect_attempts = 10
reconnect_delay = 3
use_udp_acceleration = true
mtu = 1280

[advanced]
connection_mode = "tcp"
log_level = "error"
```

## 📚 Related Documentation

- [Quick Start Guide](quick-start.md) - Test configuration parsing
- [C API Reference](api/c-api.md) - Configuration parsing functions
- [Troubleshooting](advanced/troubleshooting.md) - Configuration issues
- [Security](advanced/security.md) - Secure configuration practices

---

**💡 Tip: Start with a minimal configuration and add options as needed.**
