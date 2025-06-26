# 🏗️ rVPNSE Architecture

## 🎯 System Overview

rVPNSE follows a layered architecture that separates concerns and provides clear interfaces between components.

```
┌─────────────────────────────────────────────────────────────┐
│                    Your Application                         │
├─────────────────────────────────────────────────────────────┤
│                  Platform Integration                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │ Network Ext. │  │ VPN Service  │  │ System APIs  │     │
│  │   (iOS)      │  │  (Android)   │  │ (Win/Linux)  │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
├─────────────────────────────────────────────────────────────┤
│                     rVPNSE C FFI                           │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ C API Functions (rvpnse.h)                         │   │
│  │ • rvpnse_client_new() • rvpnse_config_load()       │   │
│  │ • rvpnse_connect()    • rvpnse_disconnect()        │   │
│  └─────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                    rVPNSE Core (Rust)                      │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐   │
│  │   Client    │ │   Config    │ │    Protocol         │   │
│  │  Manager    │ │  Manager    │ │   Handlers          │   │
│  └─────────────┘ └─────────────┘ └─────────────────────┘   │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐   │
│  │    TLS      │ │    Crypto   │ │      Error          │   │
│  │  Handler    │ │   Provider  │ │    Handling         │   │
│  └─────────────┘ └─────────────┘ └─────────────────────┘   │
├─────────────────────────────────────────────────────────────┤
│                  System Dependencies                       │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────┐   │
│  │  AWS-LC-RS  │ │    Ring     │ │      OpenSSL        │   │
│  │  (Android)  │ │  (Desktop)  │ │     (Fallback)      │   │
│  └─────────────┘ └─────────────┘ └─────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

## 🧩 Core Components

### 1. **Client Manager**
- Connection lifecycle management
- State tracking and synchronization
- Event handling and callbacks
- Session management

### 2. **Configuration Manager**
- TOML configuration parsing
- Environment variable support
- Runtime configuration updates
- Validation and defaults

### 3. **Protocol Handlers**
- SoftEther SSL-VPN protocol implementation
- Packet processing and routing
- Connection establishment and maintenance
- Authentication and authorization

### 4. **TLS Handler**
- Secure TLS connections
- Certificate validation
- Cipher suite negotiation
- Connection security management

### 5. **Crypto Provider**
- Platform-optimized cryptography
- AWS-LC-RS for Android (FIPS compliance)
- Ring for desktop platforms
- Hardware acceleration support

### 6. **Error Handling**
- Comprehensive error types
- Error propagation and logging
- Debugging and diagnostics
- Recovery mechanisms

## 🔄 Data Flow

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│    Your     │───▶│   rVPNSE    │───▶│  VPN        │
│    App      │    │   C FFI     │    │  Server     │
└─────────────┘    └─────────────┘    └─────────────┘
       ▲                   │                  │
       │                   ▼                  │
       │            ┌─────────────┐           │
       │            │   Config    │           │
       │            │  Manager    │           │
       │            └─────────────┘           │
       │                   │                  │
       │                   ▼                  │
       │            ┌─────────────┐           │
       │            │  Protocol   │           │
       │            │   Handler   │           │
       │            └─────────────┘           │
       │                   │                  │
       │                   ▼                  │
       │            ┌─────────────┐           │
       └────────────│    TLS      │───────────┘
                    │   Handler   │
                    └─────────────┘
```

## 🎯 Design Principles

### **1. Safety First**
- Memory-safe Rust implementation
- Comprehensive error handling
- Input validation and sanitization
- Secure defaults

### **2. Cross-Platform**
- Single codebase for all platforms
- Platform-specific optimizations
- Consistent API across platforms
- Native performance

### **3. Production Ready**
- Extensive testing and validation
- Performance optimization
- Resource management
- Monitoring and diagnostics

### **4. Developer Friendly**
- Simple C FFI interface
- Comprehensive documentation
- Clear error messages
- Rich debugging support

## 🔧 Platform-Specific Adaptations

### **Android**
- Uses AWS-LC-RS for FIPS compliance
- VpnService integration
- Android permissions handling
- ARM optimization

### **iOS**
- NetworkExtension framework support
- App Store compatibility
- iOS security requirements
- Metal performance shaders

### **Desktop (Linux/Windows/macOS)**
- Ring cryptography for performance
- Native TUN/TAP interfaces
- System service integration
- Platform-specific networking

## 📊 Performance Characteristics

| Component | Memory Usage | CPU Usage | Notes |
|-----------|--------------|-----------|-------|
| **Core Library** | ~2-5 MB | Low | Rust efficiency |
| **TLS Connections** | ~100KB/conn | Medium | Per connection |
| **Crypto Operations** | Minimal | Low-High | Hardware accelerated |
| **Configuration** | <1 MB | Minimal | TOML parsing |

## 🔍 Integration Patterns

### **Pattern 1: Direct Integration**
```c
// Simple direct API usage
RvpnseClient* client = rvpnse_client_new(config);
rvpnse_client_connect(client);
```

### **Pattern 2: Event-Driven**
```c
// Callback-based integration
rvpnse_client_set_callbacks(client, &callbacks);
rvpnse_client_connect_async(client);
```

### **Pattern 3: Service Wrapper**
```c
// Platform service integration
RvpnseService* service = rvpnse_service_new();
rvpnse_service_start(service, config);
```

## 🎯 Next Steps

- Learn about [Features](features.md)
- Explore [Use Cases](use-cases.md)
- Check [Development Status](status.md)
- Start with [Quick Start Guide](../02-quickstart/README.md)
