# Rust VPNSE Overview

## 🎯 What is Rust VPNSE?

Rust VPNSE is a **static library framework** that provides the foundation for SoftEther SSL-VPN protocol implementation. It's designed to be integrated into any application that needs SoftEther VPN connectivity, providing configuration management, state tracking, and FFI interfaces while requiring protocol and platform-specific implementation from your application.

## 🏗️ Architecture

```
┌─────────────────────────────────────┐
│ Your Application                    │
│ ┌─────────────────────────────────┐ │
│ │ Platform-Specific Implementation│ │
│ │ ├── TLS Connection (required)   │ │
│ │ ├── SoftEther Protocol (req.)   │ │
│ │ ├── TUN/TAP Interface (req.)    │ │
│ │ ├── Routing Management (req.)   │ │
│ │ ├── DNS Configuration (req.)    │ │
│ │ └── Platform Permissions (req.) │ │
│ └─────────────────────────────────┘ │
│ ┌─────────────────────────────────┐ │
│ │ Rust VPNSE Framework (provided) │ │
│ │ ├── Configuration Management   │ │
│ │ ├── Connection State Tracking  │ │
│ │ ├── Session Management Frame   │ │
│ │ ├── Error Handling System      │ │
│ │ └── C FFI Interface           │ │
│ └─────────────────────────────────┘ │
└─────────────────────────────────────┘
```

## ✅ What Rust VPNSE Framework Provides

### **Framework Components**
- **Configuration Management** - TOML-based configuration parsing and validation
- **Connection State Management** - Connection lifecycle tracking and session state
- **Error Handling System** - Comprehensive error types with detailed error information
- **FFI Interface** - Complete C-compatible API for cross-platform integration
- **Platform Abstraction** - Modular structure for platform-specific implementations
- **Example Patterns** - Integration examples showing expected API usage

### **Language Integration Support**
- **C/C++** - Native C header with all functions
- **Swift** - iOS/macOS integration via C bridge
- **Kotlin/Java** - Android integration via JNI
- **Dart** - Flutter integration via FFI
- **C#** - .NET integration via P/Invoke
- **Python** - Python integration via ctypes

### **Platform Framework Support**
| Platform | Framework Builds | Integration Required |
|----------|----------------|---------------------|
| **iOS** | ✅ Static Library | TLS + Protocol + NetworkExtension |
| **Android** | ✅ Static Library | TLS + Protocol + VpnService |
| **Windows** | ✅ Static Library | TLS + Protocol + WinTUN |
| **macOS** | ✅ Static Library | TLS + Protocol + utun |
| **Linux** | ✅ Static Library | TLS + Protocol + TUN/TAP |

## 🏗️ Implementation Status

### **✅ Framework Components (Complete)**
- **Configuration Management** - TOML parsing, validation, defaults
- **Error Handling System** - Comprehensive error types, FFI-compatible codes
- **Connection State Management** - Session lifecycle, state tracking
- **FFI Interface** - Complete C API with safe pointer handling
- **Platform Abstraction** - Modular structure for platform implementations

### **🔧 Integration Required (Your Implementation)**
- **TLS Implementation** - Connect using your platform's TLS library
- **SoftEther Protocol** - HTTP/HTTPS communication with SoftEther servers
- **Platform Networking** - TUN/TAP interface creation and management
- **Packet Handling** - Read/write packets from VPN interface
- **Routing Management** - Configure routing tables and traffic forwarding
- **DNS Configuration** - Set DNS servers and prevent leaks

### **📋 Current Examples Status**
- **Framework Examples** ✅ - Show API usage patterns and integration structure
- **Protocol Implementation** 🔧 - Simplified mock for demonstration (you implement real protocol)
- **Tunnel Management** 🔧 - Interface definitions only (you implement platform TUN/TAP)

### **Why This Architecture?**
Each platform has different VPN APIs and security requirements:

- **iOS** requires `NetworkExtension` framework integration
- **Android** requires `VpnService` class implementation  
- **Windows** requires WinTUN or TAP-Windows driver integration
- **macOS** requires `utun` interface creation
- **Linux** requires TUN/TAP device management

The framework provides the foundation while you implement the platform-specific components using the appropriate APIs.

## 🔄 Data Flow

```
Your App ←→ Platform VPN API ←→ Rust VPNSE ←→ SoftEther Server
    ↑              ↑                ↑              ↑
Interface     TUN/TAP         Protocol      Network
Management    Packets         Handling    Communication
```

### **Typical Flow**
1. **Your App**: Request VPN connection
2. **Rust VPNSE**: Parse configuration, connect to server, authenticate
3. **Your App**: Create TUN/TAP interface based on server settings
4. **Your App**: Configure routing and DNS
5. **Runtime**: Forward packets between TUN/TAP and Rust VPNSE
6. **Rust VPNSE**: Handle protocol communication with server

## 🎯 Use Cases

### **Mobile VPN Apps**
- **iOS VPN apps** using NetworkExtension
- **Android VPN apps** using VpnService
- **Flutter cross-platform** VPN apps

### **Desktop VPN Clients**
- **Windows VPN clients** using WinTUN
- **macOS VPN clients** using utun
- **Linux VPN clients** using TUN/TAP

### **Enterprise Integration**
- **Corporate apps** with embedded VPN
- **IoT devices** with SoftEther connectivity
- **Network appliances** with VPN capabilities

## 🚀 Getting Started

1. **Read** the [Quick Start Guide](quick-start.md)
2. **Choose** your platform integration guide:
   - [iOS Integration](integration/ios.md)
   - [Android Integration](integration/android.md)
   - [Flutter Integration](integration/flutter.md)
   - [C/C++ Integration](integration/c-cpp.md)
3. **Build** the static library for your platform
4. **Implement** platform-specific networking
5. **Test** with a SoftEther server

## 📚 Next Steps

- [Quick Start Guide](quick-start.md) - Build and test in 5 minutes
- [Configuration Reference](configuration.md) - Understand TOML configuration
- [Integration Guides](integration/) - Platform-specific integration
- [Troubleshooting](advanced/troubleshooting.md) - Common issues and solutions

---

**Remember: Rust VPNSE provides the protocol, you provide the platform integration.**
