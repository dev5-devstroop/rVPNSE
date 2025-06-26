# RVPNSE C API Reference

## Overview

The RVPNSE library provides a comprehensive C Foreign Function Interface (FFI) for seamless integration with applications written in C, C++, and other languages that support C bindings.

## Header File

```c
#include "rvpnse.h"
```

## Core Functions

### Configuration Management

#### `rvpnse_config_new`
```c
struct VpnConfig* rvpnse_config_new(void);
```
**Description**: Creates a new VPN configuration structure.
**Returns**: Pointer to `VpnConfig` or `NULL` on failure.
**Thread Safety**: Safe

#### `rvpnse_config_free`
```c
void rvpnse_config_free(struct VpnConfig* config);
```
**Description**: Frees memory allocated for VPN configuration.
**Parameters**: 
- `config`: Pointer to configuration structure to free
**Thread Safety**: Safe

#### `rvpnse_config_load_from_file`
```c
int rvpnse_config_load_from_file(struct VpnConfig* config, const char* file_path);
```
**Description**: Loads configuration from TOML file.
**Parameters**:
- `config`: Pointer to configuration structure
- `file_path`: Path to TOML configuration file
**Returns**: 0 on success, error code on failure
**Thread Safety**: Safe

### Client Management

#### `rvpnse_client_new`
```c
struct VpnClient* rvpnse_client_new(struct VpnConfig* config);
```
**Description**: Creates a new VPN client instance.
**Parameters**:
- `config`: Pointer to initialized configuration
**Returns**: Pointer to `VpnClient` or `NULL` on failure
**Thread Safety**: Safe

#### `rvpnse_client_free`
```c
void rvpnse_client_free(struct VpnClient* client);
```
**Description**: Frees memory allocated for VPN client.
**Parameters**:
- `client`: Pointer to client structure to free
**Thread Safety**: Safe

#### `rvpnse_client_connect`
```c
int rvpnse_client_connect(struct VpnClient* client);
```
**Description**: Initiates VPN connection.
**Parameters**:
- `client`: Pointer to initialized client
**Returns**: 0 on success, error code on failure
**Thread Safety**: Not thread-safe (single client instance)

#### `rvpnse_client_disconnect`
```c
int rvpnse_client_disconnect(struct VpnClient* client);
```
**Description**: Disconnects active VPN connection.
**Parameters**:
- `client`: Pointer to connected client
**Returns**: 0 on success, error code on failure
**Thread Safety**: Not thread-safe (single client instance)

#### `rvpnse_client_status`
```c
enum ConnectionState rvpnse_client_status(struct VpnClient* client);
```
**Description**: Gets current connection status.
**Parameters**:
- `client`: Pointer to client
**Returns**: Current connection state
**Thread Safety**: Safe

## Data Structures

### VpnConfig
```c
struct VpnConfig {
    char* server_host;
    uint16_t server_port;
    char* username;
    char* password;
    char* hub_name;
    bool verify_certificate;
    uint32_t timeout_seconds;
    uint32_t keepalive_interval;
};
```

### VpnClient
```c
struct VpnClient {
    // Opaque structure - do not access members directly
    // Use provided API functions
};
```

### ConnectionState
```c
enum ConnectionState {
    CONNECTION_DISCONNECTED = 0,
    CONNECTION_CONNECTING = 1,
    CONNECTION_CONNECTED = 2,
    CONNECTION_DISCONNECTING = 3,
    CONNECTION_ERROR = 4
};
```

## Error Codes

| Code | Name | Description |
|------|------|-------------|
| `0` | `RVPNSE_SUCCESS` | Operation completed successfully |
| `1` | `RVPNSE_ERROR_CONFIG` | Configuration error |
| `2` | `RVPNSE_ERROR_NETWORK` | Network connectivity error |
| `3` | `RVPNSE_ERROR_CONNECTION` | Connection establishment error |
| `4` | `RVPNSE_ERROR_AUTHENTICATION` | Authentication failure |
| `5` | `RVPNSE_ERROR_PROTOCOL` | Protocol error |
| `6` | `RVPNSE_ERROR_CRYPTO` | Cryptographic error |
| `7` | `RVPNSE_ERROR_PLATFORM` | Platform-specific error |
| `8` | `RVPNSE_ERROR_TUNTAP` | TUN/TAP interface error |
| `9` | `RVPNSE_ERROR_ROUTING` | Routing configuration error |
| `10` | `RVPNSE_ERROR_DNS` | DNS configuration error |
| `11` | `RVPNSE_ERROR_PERMISSION` | Insufficient permissions |

## Usage Example

```c
#include "rvpnse.h"
#include <stdio.h>
#include <stdlib.h>

int main() {
    // Create configuration
    struct VpnConfig* config = rvpnse_config_new();
    if (!config) {
        fprintf(stderr, "Failed to create config\n");
        return 1;
    }

    // Load from file
    if (rvpnse_config_load_from_file(config, "config.toml") != 0) {
        fprintf(stderr, "Failed to load config\n");
        rvpnse_config_free(config);
        return 1;
    }

    // Create client
    struct VpnClient* client = rvpnse_client_new(config);
    if (!client) {
        fprintf(stderr, "Failed to create client\n");
        rvpnse_config_free(config);
        return 1;
    }

    // Connect
    printf("Connecting to VPN...\n");
    if (rvpnse_client_connect(client) != 0) {
        fprintf(stderr, "Connection failed\n");
        rvpnse_client_free(client);
        rvpnse_config_free(config);
        return 1;
    }

    printf("Connected successfully!\n");

    // Check status
    enum ConnectionState state = rvpnse_client_status(client);
    printf("Connection state: %d\n", state);

    // Disconnect and cleanup
    rvpnse_client_disconnect(client);
    rvpnse_client_free(client);
    rvpnse_config_free(config);

    return 0;
}
```

## Platform-Specific Notes

### Windows
- Requires Administrator privileges for TUN/TAP interface creation
- TAP-Windows driver must be installed
- Use Windows SDK for compilation

### macOS
- Requires root privileges or proper entitlements
- Uses `utun` interfaces
- Xcode command line tools required

### Linux
- Requires `CAP_NET_ADMIN` capability or root privileges
- TUN/TAP kernel modules must be loaded
- Use standard GCC toolchain

### Android
- Requires `VpnService` permission in manifest
- Use Android NDK for compilation
- Must target appropriate API levels

### iOS
- Requires NetworkExtension entitlements
- Must be code-signed for device deployment
- Use Xcode for compilation and signing

## Memory Management

- All structures returned by `*_new()` functions must be freed with corresponding `*_free()` functions
- String parameters are copied internally - caller retains ownership
- The library handles internal memory management automatically

## Thread Safety

- Configuration and status functions are thread-safe
- Connection operations (connect/disconnect) are not thread-safe
- Use external synchronization when accessing the same client from multiple threads
