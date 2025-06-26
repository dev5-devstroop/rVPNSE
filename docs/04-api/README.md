# 📚 API Reference

Complete API documentation for rVPNSE. Choose your interface below.

## 📑 Contents

- [🔗 C FFI API](c-ffi.md) - C Foreign Function Interface (primary API)
- [🦀 Rust API](rust.md) - Native Rust API (for Rust projects)
- [❌ Error Codes](errors.md) - Complete error code reference
- [📋 Configuration](config.md) - Configuration schema and options

## 🎯 API Overview

rVPNSE provides multiple API interfaces to suit different programming languages and use cases:

### **Primary APIs**

| API | Language | Use Case | Documentation |
|-----|----------|----------|---------------|
| **C FFI** | C/C++ | Native applications, language bindings | [C FFI API](c-ffi.md) |
| **Rust** | Rust | Rust applications, direct integration | [Rust API](rust.md) |

### **Language Bindings** (via C FFI)

| Language | Status | Documentation | Example |
|----------|--------|---------------|---------|
| **C/C++** | ✅ Native | [C/C++ Guide](../03-integration/native/cpp.md) | [Example](../02-quickstart/examples.md#c-example) |
| **Python** | ✅ Complete | [Python Guide](../03-integration/desktop/python.md) | [Example](../02-quickstart/examples.md#python-example) |
| **C#/.NET** | ✅ Complete | [.NET Guide](../03-integration/desktop/dotnet.md) | [Example](../02-quickstart/examples.md#csharp-example) |
| **Swift** | ✅ Complete | [iOS Guide](../03-integration/mobile/ios.md) | [Example](../02-quickstart/examples.md#swift-example) |
| **Kotlin/Java** | ✅ Complete | [Android Guide](../03-integration/mobile/android.md) | [Example](../02-quickstart/examples.md#kotlin-example) |
| **Dart** | ✅ Complete | [Flutter Guide](../03-integration/mobile/flutter.md) | [Example](../02-quickstart/examples.md#dart-example) |

## 🔗 C FFI API Quick Reference

### **Core Functions**
```c
// Library initialization
RvpnseResult rvpnse_init(void);
void rvpnse_cleanup(void);
const char* rvpnse_version(void);

// Configuration management
RvpnseConfig* rvpnse_config_new(void);
RvpnseConfig* rvpnse_config_from_file(const char* path);
RvpnseConfig* rvpnse_config_from_string(const char* toml);
void rvpnse_config_free(RvpnseConfig* config);

// Client management
RvpnseClient* rvpnse_client_new(RvpnseConfig* config);
void rvpnse_client_free(RvpnseClient* client);

// Connection control
RvpnseResult rvpnse_client_connect(RvpnseClient* client);
RvpnseResult rvpnse_client_disconnect(RvpnseClient* client);
RvpnseConnectionState rvpnse_client_state(RvpnseClient* client);
```

### **Result Codes**
```c
typedef enum {
    rVPNSE_SUCCESS = 0,
    rVPNSE_ERROR_INVALID_CONFIG = 1,
    rVPNSE_ERROR_CONNECTION_FAILED = 2,
    rVPNSE_ERROR_AUTHENTICATION_FAILED = 3,
    rVPNSE_ERROR_NETWORK_ERROR = 4,
    rVPNSE_ERROR_TIMEOUT = 5,
    // ... more error codes
} RvpnseResult;
```

## 🦀 Rust API Quick Reference

### **Core Types**
```rust
use rvpnse::{Client, Config, Error, Result};

// Configuration
let config = Config::from_file("config.toml")?;
let config = Config::from_str(toml_string)?;

// Client creation and management
let client = Client::new(config)?;
client.connect().await?;
client.disconnect().await?;

// State monitoring
let state = client.state();
let stats = client.statistics();
```

### **Async API**
```rust
use rvpnse::AsyncClient;

let client = AsyncClient::new(config)?;

// Non-blocking operations
let handle = client.connect_async()?;
handle.await?;

// Event-driven
client.on_state_changed(|state| {
    println!("Connection state: {:?}", state);
});
```

## 📋 Configuration Schema

### **Basic Configuration**
```toml
[server]
host = "vpn.example.com"
port = 443
protocol = "ssl"

[client]
name = "MyClient"
auto_reconnect = true

[credentials]
username = "user"
password = "pass"
```

### **Advanced Configuration**
```toml
[network]
use_system_dns = true
dns_servers = ["8.8.8.8", "8.8.4.4"]
routes = ["0.0.0.0/0"]
mtu = 1500

[security]
verify_server_cert = true
allowed_ciphers = ["ECDHE-RSA-AES256-GCM-SHA384"]

[performance]
connection_timeout = 30
keepalive_interval = 60
max_retries = 3
```

## ❌ Error Handling

### **C FFI Error Handling**
```c
RvpnseResult result = rvpnse_client_connect(client);
if (result != rVPNSE_SUCCESS) {
    const char* error_msg = rvpnse_error_string(result);
    printf("Connection failed: %s\\n", error_msg);
    
    // Get detailed error info
    RvpnseErrorInfo info;
    rvpnse_get_last_error(&info);
    printf("Error code: %d, Details: %s\\n", info.code, info.message);
}
```

### **Rust Error Handling**
```rust
match client.connect().await {
    Ok(()) => println!("Connected successfully"),
    Err(Error::ConnectionFailed { reason }) => {
        eprintln!("Connection failed: {}", reason);
    }
    Err(Error::AuthenticationFailed) => {
        eprintln!("Invalid credentials");
    }
    Err(e) => eprintln!("Unexpected error: {}", e),
}
```

## 🔄 Callback System

### **C FFI Callbacks**
```c
typedef struct {
    void (*on_state_changed)(RvpnseConnectionState state, void* userdata);
    void (*on_error)(RvpnseResult error, const char* message, void* userdata);
    void (*on_stats_updated)(RvpnseStats* stats, void* userdata);
} RvpnseCallbacks;

// Set callbacks
RvpnseCallbacks callbacks = {
    .on_state_changed = handle_state_change,
    .on_error = handle_error,
    .on_stats_updated = handle_stats
};
rvpnse_client_set_callbacks(client, &callbacks, userdata);
```

### **Rust Callbacks**
```rust
client.on_state_changed(|state| {
    match state {
        ConnectionState::Connected => println!("VPN connected"),
        ConnectionState::Disconnected => println!("VPN disconnected"),
        ConnectionState::Connecting => println!("Connecting..."),
        _ => {}
    }
});

client.on_error(|error| {
    eprintln!("VPN error: {}", error);
});
```

## 📊 Statistics and Monitoring

### **Connection Statistics**
```c
RvpnseStats stats;
rvpnse_client_get_stats(client, &stats);

printf("Bytes sent: %llu\\n", stats.bytes_sent);
printf("Bytes received: %llu\\n", stats.bytes_received);
printf("Connection time: %u seconds\\n", stats.connection_duration);
printf("Latency: %u ms\\n", stats.latency_ms);
```

```rust
let stats = client.statistics();
println!("Throughput: {:.2} MB/s", stats.throughput_mbps());
println!("Packet loss: {:.1}%", stats.packet_loss_percent());
```

## 🔧 Advanced Features

### **Custom Network Interfaces**
```c
// Custom TUN/TAP handling
RvpnseNetworkCallbacks net_callbacks = {
    .create_interface = my_create_tun,
    .configure_interface = my_configure_tun,
    .read_packet = my_read_packet,
    .write_packet = my_write_packet
};
rvpnse_client_set_network_callbacks(client, &net_callbacks);
```

### **Certificate Management**
```c
// Custom certificate validation
RvpnseResult validate_cert(const char* cert_pem, void* userdata) {
    // Custom validation logic
    return rVPNSE_SUCCESS;
}

rvpnse_client_set_cert_validator(client, validate_cert, userdata);
```

## 🎯 Next Steps

- **New to the API?** → Start with [C FFI API](c-ffi.md)
- **Using Rust?** → Check [Rust API](rust.md)
- **Need examples?** → Browse [Integration Guides](../03-integration/README.md)
- **Having issues?** → Check [Error Codes](errors.md) and [Troubleshooting](../07-troubleshooting/README.md)
