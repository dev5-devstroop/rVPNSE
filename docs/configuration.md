# Configuration Guide

This document describes the configuration structure for rVPNSE, including all available options and their defaults.

## Configuration Structure

rVPNSE uses TOML format for configuration. The configuration is divided into several sections:

## [server] - Server Connection Settings

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `address` | String | ✅ Yes | - | Server IP address (mandatory, used for direct connection) |
| `hostname` | String | ❌ No | `None` | Server hostname for Host header (optional, for clustering) |
| `port` | u16 | ✅ Yes | - | Server port (usually 443 or 992) |
| `hub` | String | ✅ Yes | - | Hub name to connect to |
| `use_ssl` | Bool | ❌ No | `true` | Use SSL/TLS connection |
| `verify_certificate` | Bool | ❌ No | `true` | Verify server certificate |
| `timeout` | u32 | ❌ No | `30` | Connection timeout in seconds |
| `keepalive_interval` | u32 | ❌ No | `60` | Keepalive interval in seconds |

### Example:
```toml
[server]
address = "62.24.65.211"        # Direct IP connection
hostname = "worxvpn.662.cloud"  # Host header for clustering
port = 992
hub = "VPN"
use_ssl = true
verify_certificate = false
timeout = 30
keepalive_interval = 60
```

## [auth] - Authentication Settings

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `method` | String | ❌ No | `"password"` | Authentication method: "password", "certificate", "anonymous" |
| `username` | String | ✅* | `None` | Username for password authentication |
| `password` | String | ✅* | `None` | Password for password authentication |
| `client_cert` | String | ✅** | `None` | Client certificate file path |
| `client_key` | String | ✅** | `None` | Client private key file path |
| `ca_cert` | String | ❌ No | `None` | CA certificate file path |

*Required for password authentication
**Required for certificate authentication

### Example:
```toml
[auth]
method = "password"
username = "myuser"
password = "mypassword"

# For certificate authentication:
# method = "certificate"
# client_cert = "/path/to/client.crt"
# client_key = "/path/to/client.key"
# ca_cert = "/path/to/ca.crt"
```

## [connection_limits] - Connection Management

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `max_connections` | u32 | ❌ No | `10` | Maximum number of concurrent connections |
| `enable_pooling` | Bool | ❌ No | `true` | Enable connection pooling |
| `pool_size` | u32 | ❌ No | `5` | Pool size for persistent connections |
| `idle_timeout` | u32 | ❌ No | `300` | Connection idle timeout in seconds |
| `max_lifetime` | u32 | ❌ No | `3600` | Maximum connection lifetime in seconds |
| `enable_multiplexing` | Bool | ❌ No | `false` | Enable connection multiplexing |
| `max_streams_per_connection` | u32 | ❌ No | `100` | Maximum multiplexed streams per connection |
| `retry_attempts` | u32 | ❌ No | `3` | Connection retry attempts |
| `retry_delay` | u32 | ❌ No | `1000` | Retry delay in milliseconds |
| `backoff_factor` | f64 | ❌ No | `2.0` | Exponential backoff factor |
| `max_retry_delay` | u32 | ❌ No | `30` | Maximum retry delay in seconds |
| `health_check_interval` | u32 | ❌ No | `30` | Connection health check interval in seconds |
| `rate_limit_rps` | u32 | ❌ No | `100` | Rate limiting: requests per second |
| `rate_limit_burst` | u32 | ❌ No | `200` | Rate limiting: burst size |

### Example:
```toml
[connection_limits]
max_connections = 10
enable_pooling = true
pool_size = 5
retry_attempts = 3
retry_delay = 1000
backoff_factor = 2.0
```

## [network] - Network Configuration

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `enable_ipv6` | Bool | ❌ No | `false` | Enable IPv6 support |
| `bind_address` | String | ❌ No | `None` | Bind to specific local address |
| `proxy_url` | String | ❌ No | `None` | Use proxy for connections |
| `user_agent` | String | ❌ No | `"rVPNSE/0.1.0"` | User agent string |
| `enable_http2` | Bool | ❌ No | `true` | Enable HTTP/2 support |
| `tcp_keepalive` | Bool | ❌ No | `true` | TCP keep-alive enabled |
| `tcp_nodelay` | Bool | ❌ No | `true` | TCP no-delay enabled |
| `socket_buffer_size` | u32 | ❌ No | `None` | Socket buffer sizes |

### Example:
```toml
[network]
enable_ipv6 = false
user_agent = "MyApp/1.0"
enable_http2 = true
tcp_keepalive = true
tcp_nodelay = true
# bind_address = "192.168.1.100"
# proxy_url = "http://proxy.example.com:8080"
```

## [logging] - Logging Configuration

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `level` | String | ❌ No | `"info"` | Log level: "error", "warn", "info", "debug", "trace" |
| `file` | String | ❌ No | `None` | Log file path (logs to console if not specified) |
| `json_format` | Bool | ❌ No | `false` | Enable JSON logging format |
| `colored` | Bool | ❌ No | `true` | Enable colored output |

### Example:
```toml
[logging]
level = "debug"
colored = true
json_format = false
# file = "rvpnse.log"
```

## Complete Example Configuration

```toml
[server]
address = "62.24.65.211"
hostname = "worxvpn.662.cloud"
port = 992
hub = "VPN"
use_ssl = true
verify_certificate = false
timeout = 30
keepalive_interval = 60

[auth]
method = "password"
username = "myuser"
password = "mypassword"

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
health_check_interval = 30
rate_limit_rps = 100
rate_limit_burst = 200

[network]
enable_ipv6 = false
user_agent = "rVPNSE/0.1.0"
enable_http2 = true
tcp_keepalive = true
tcp_nodelay = true

[logging]
level = "info"
colored = true
json_format = false
```

## Validation Rules

The configuration is automatically validated when loaded:

1. **Server validation**:
   - `address` cannot be empty
   - `port` must be non-zero
   - `hub` cannot be empty

2. **Authentication validation**:
   - For password method: `username` and `password` are required
   - For certificate method: `client_cert` and `client_key` are required
   - For anonymous method: no additional validation

3. **Network validation**:
   - `bind_address` must be a valid IP address if specified

4. **Connection limits validation**:
   - `max_connections` cannot exceed 1000
   - `pool_size` cannot exceed `max_connections`

## Environment Variables

You can override configuration values using environment variables:

```bash
RVPNSE_SERVER_ADDRESS=192.168.1.1
RVPNSE_SERVER_PORT=443
RVPNSE_AUTH_USERNAME=myuser
RVPNSE_AUTH_PASSWORD=mypass
RVPNSE_LOGGING_LEVEL=debug
```

## Loading Configuration

### From File
```rust
use rvpnse::config::Config;

let config = Config::from_file("config.toml")?;
```

### From String
```rust
let config_str = r#"
[server]
address = "192.168.1.1"
port = 443
hub = "VPN"

[auth]
username = "user"
password = "pass"
"#;

let config: Config = config_str.parse()?;
```

### Programmatically
```rust
use rvpnse::config::{Config, ServerConfig, AuthConfig, AuthMethod};

let config = Config {
    server: ServerConfig {
        address: "192.168.1.1".to_string(),
        hostname: Some("vpn.example.com".to_string()),
        port: 443,
        hub: "VPN".to_string(),
        use_ssl: true,
        verify_certificate: true,
        timeout: 30,
        keepalive_interval: 60,
    },
    auth: AuthConfig {
        method: AuthMethod::Password,
        username: Some("user".to_string()),
        password: Some("pass".to_string()),
        client_cert: None,
        client_key: None,
        ca_cert: None,
    },
    // ... other fields with defaults
    ..Default::default()
};
```
