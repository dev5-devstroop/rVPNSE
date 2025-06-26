# 📚 rVPNSE Documentation

Welcome to **rVPNSE** - Production-ready Rust VPN library with SoftEther SSL-VPN protocol implementation.

> 🚀 **rVPNSE** provides a complete, cross-platform VPN solution with C FFI interface for seamless integration into any application.

## ⚡ Quick Navigation

<table>
<tr>
<td width="50%">

### 🏁 **Getting Started**
- [👀 What is rVPNSE?](01-overview/README.md)
- [🚀 Quick Start Guide](02-quickstart/README.md)
- [🛠️ Installation](02-quickstart/installation.md)
- [⚙️ Configuration](02-quickstart/configuration.md)

### 🔗 **Integration**
- [📱 Mobile Apps](03-integration/mobile/README.md)
- [🖥️ Desktop Apps](03-integration/desktop/README.md)
- [🌐 Web Apps](03-integration/web/README.md)
- [🔧 C/C++ Native](03-integration/native/README.md)

</td>
<td width="50%">

### 📚 **API Reference**
- [🔗 C FFI API](04-api/c-ffi.md)
- [🦀 Rust API](04-api/rust.md)
- [❌ Error Codes](04-api/errors.md)
- [📋 Configuration Schema](04-api/config.md)

### 🔧 **Advanced**
- [🏗️ Build System](05-build/README.md)
- [🔒 Security Guide](06-advanced/security.md)
- [⚡ Performance](06-advanced/performance.md)
- [🐛 Troubleshooting](07-troubleshooting/README.md)

</td>
</tr>
</table>

---

## 📖 Documentation Sections

### 1️⃣ [Overview & Concepts](01-overview/)
Learn what rVPNSE is, its architecture, and core concepts.

### 2️⃣ [Quick Start](02-quickstart/)
Get up and running with rVPNSE in minutes.

### 3️⃣ [Integration Guides](03-integration/)
Platform-specific integration guides for mobile, desktop, and web applications.

### 4️⃣ [API Reference](04-api/)
Complete API documentation for C FFI and Rust interfaces.

### 5️⃣ [Build System](05-build/)
Comprehensive build guides for all supported platforms.

### 6️⃣ [Advanced Topics](06-advanced/)
Security, performance optimization, and advanced configuration.

### 7️⃣ [Troubleshooting](07-troubleshooting/)
Common issues, debugging guides, and FAQ.

---

## 🎯 Platform Support

| Platform | Status | Documentation |
|----------|--------|---------------|
| **Linux** | ✅ Production | [Linux Guide](03-integration/platforms/linux.md) |
| **Windows** | ✅ Production | [Windows Guide](03-integration/platforms/windows.md) |
| **macOS** | ✅ Production | [macOS Guide](03-integration/platforms/macos.md) |
| **Android** | ✅ Production | [Android Guide](03-integration/mobile/android.md) |
| **iOS** | ✅ Production | [iOS Guide](03-integration/mobile/ios.md) |

## 🔧 Language Bindings

| Language | Status | Documentation |
|----------|--------|---------------|
| **C/C++** | ✅ Native | [C/C++ Integration](03-integration/native/cpp.md) |
| **Rust** | ✅ Native | [Rust Integration](03-integration/native/rust.md) |
| **Swift** | ✅ Complete | [Swift Integration](03-integration/mobile/ios.md) |
| **Kotlin/Java** | ✅ Complete | [Android Integration](03-integration/mobile/android.md) |
| **Dart/Flutter** | ✅ Complete | [Flutter Integration](03-integration/mobile/flutter.md) |
| **C#/.NET** | ✅ Complete | [.NET Integration](03-integration/desktop/dotnet.md) |
| **Python** | ✅ Complete | [Python Integration](03-integration/desktop/python.md) |
| **JavaScript** | � In Progress | [JS Integration](03-integration/web/javascript.md) |

---

## 🚀 Quick Start

```bash
# Download the library for your platform
curl -L https://github.com/devstroop/rvpnse/releases/latest/download/librvpnse-linux-x64.so

# Include in your project
#include "rvpnse.h"

# Initialize and connect
RvpnseConfig* config = rvpnse_config_from_file("config.toml");
RvpnseClient* client = rvpnse_client_new(config);
rvpnse_client_connect(client);
```

See the [Quick Start Guide](02-quickstart/README.md) for detailed instructions.

---

## ⛳️ Project Status

- 🎯 **Version**: v0.1.0
- 📅 **Last Updated**: June 2025
- 🔒 **Security**: AWS-LC-RS & Ring cryptography
- ⚡ **Performance**: Production-optimized builds
- 🧪 **Testing**: Comprehensive test suite
- 📚 **Documentation**: Complete & up-to-date

---

## 🤝 Contributing

- [Development Status](01-overview/status.md)
- [Contributing Guide](../CONTRIBUTING.md)
- [Build Instructions](05-build/README.md)
- [Testing Guide](07-troubleshooting/testing.md)

---

## 📄 License

rVPNSE is open source software. See [LICENSE](../LICENSE) for details.

## 🆘 Support

- 📖 **Documentation**: You're reading it!
- 🐛 **Issues**: [GitHub Issues](https://github.com/devstroop/rvpnse/issues)
- 💬 **Discussions**: [GitHub Discussions](https://github.com/devstroop/rvpnse/discussions)
- 📧 **Email**: [support@devstroop.com](mailto:support@devstroop.com)
- **[Cross-compilation](advanced/cross-compilation.md)** - Advanced build customization

### 🏗️ Architecture & Design
- **[Architecture Diagrams](architecture-diagrams.md)** - Visual system architecture overview
- **[Connection Limits Diagrams](connection-limits-diagrams.md)** - Visual guide to connection management
- **[Deployment Architectures](deployment-architectures.md)** - Real-world deployment scenarios
- **[State Machine Diagrams](architecture-diagrams.md#vpn-client-state-machine)** - Client state transitions
- **[Protocol Flow Diagrams](architecture-diagrams.md#protocol-communication-flow)** - Communication sequences

### 🛠️ Development
- **[Contributing](../CONTRIBUTING.md)** - How to contribute to the project
- **[Architecture](development/architecture.md)** - Internal library architecture
- **[Testing](development/testing.md)** - Testing strategies and test suite
- **[Release Process](development/releases.md)** - How releases are managed

## 🎯 Key Features

### ✅ Production Ready
- **Zero warnings** - Strict code quality with clippy compliance
- **100% test coverage** - All core functionality thoroughly tested
- **Multi-platform CI/CD** - Automated builds and releases
- **Security auditing** - Automated vulnerability scanning

### 🔄 Cross-Platform Support
- **Desktop**: Windows, macOS, Linux
- **Mobile**: Android, iOS
- **Architectures**: x86_64, ARM64, ARMv7

### �️ Enterprise Features
- **TLS/SSL encryption** - Secure communication channels
- **Certificate validation** - Configurable certificate verification
- **Session management** - Robust connection state handling
- **Error recovery** - Graceful handling of network issues

## 📊 Current Status

**Version**: 0.1.0  
**Status**: Production Ready ✅  
**Last Updated**: June 26, 2025

| Component | Status | Notes |
|-----------|--------|-------|
| **Core Library** | ✅ Complete | All major features implemented |
| **C FFI Interface** | ✅ Complete | Stable API for integration |
| **Build System** | ✅ Complete | Unified Python build for all platforms |
| **Documentation** | ✅ Complete | Comprehensive guides and API docs |
| **CI/CD Pipeline** | ✅ Complete | Automated testing and releases |
| **Platform Support** | ✅ Complete | Windows, macOS, Linux, Android, iOS |

See [Development Status](STATUS.md) for detailed progress information.

## 🏃‍♂️ Quick Example

```c
#include "rvpnse.h"

int main() {
    // Create and configure VPN client
    struct VpnConfig* config = rvpnse_config_new();
    rvpnse_config_load_from_file(config, "vpn.toml");
    
    struct VpnClient* client = rvpnse_client_new(config);
    
    // Connect to VPN
    if (rvpnse_client_connect(client) == 0) {
        printf("Connected to VPN!\n");
        
        // Your application logic here
        
        rvpnse_client_disconnect(client);
    }
    
    // Cleanup
    rvpnse_client_free(client);
    rvpnse_config_free(config);
    return 0;
}
```

## 🆘 Getting Help

- **Documentation Issues**: Check the [troubleshooting guide](troubleshooting.md)
- **Build Problems**: See the [build troubleshooting](build/README.md#troubleshooting)
- **Integration Help**: Review the [integration guides](#integration-guides)
- **Bug Reports**: [Open an issue](https://github.com/devstroop/rvpnse/issues)
- **Feature Requests**: [Start a discussion](https://github.com/devstroop/rvpnse/discussions)

## 📄 License

RVPNSE is licensed under the [Apache License 2.0](../LICENSE).

---

**Ready to get started?** Jump to the [Quick Start Guide](quick-start.md) or [Build Documentation](build/README.md)!
| **Integrate into Flutter app** | [Flutter Integration](integration/flutter.md) |
| **Use with C/C++** | [C/C++ Integration](integration/c-cpp.md) |
| **Configure the library** | [Configuration](configuration.md) |
| **Solve a problem** | [Troubleshooting](advanced/troubleshooting.md) |
| **Contribute code** | [Contributing](../CONTRIBUTING.md) |

## ⚠️ Important Notes

- **Rust VPNSE is a protocol library** - It handles SoftEther SSL-VPN communication
- **You must implement platform networking** - TUN/TAP interfaces, routing, DNS
- **Each platform has specific requirements** - Check the platform guides
- **Start with the integration guide** for your target platform

---

*Last updated: December 2024*
