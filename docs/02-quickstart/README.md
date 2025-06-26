# 🚀 Quick Start Guide

Get up and running with rVPNSE in under 10 minutes! This guide will walk you through downloading, installing, and making your first VPN connection.

## 📑 Contents

- [🛠️ Installation](installation.md) - Download and install RVPNSE
- [⚙️ Configuration](configuration.md) - Basic configuration setup
- [🔌 First Connection](first-connection.md) - Make your first VPN connection
- [📱 Platform Guides](platform-guides.md) - Platform-specific quickstarts
- [🧪 Examples](examples.md) - Working code examples

## ⚡ 5-Minute Quick Start

### Step 1: Download rVPNSE

Choose your platform and download the latest release:

```bash
# Linux x64
curl -L -o librvpnse.so \
  https://github.com/devstroop/rvpnse/releases/latest/download/librvpnse-linux-x64.so

# macOS Intel
curl -L -o librvpnse.dylib \
  https://github.com/devstroop/rvpnse/releases/latest/download/librvpnse-macos-x64.dylib

# macOS Apple Silicon
curl -L -o librvpnse.dylib \
  https://github.com/devstroop/rvpnse/releases/latest/download/librvpnse-macos-arm64.dylib

# Windows x64
curl -L -o librvpnse.dll \
  https://github.com/devstroop/rvpnse/releases/latest/download/librvpnse-windows-x64.dll
```

### Step 2: Get the Header File

```bash
curl -L -o rvpnse.h \
  https://github.com/devstroop/rvpnse/releases/latest/download/rvpnse.h
```

### Step 3: Create Configuration

Create a `config.toml` file:

```toml
[server]
host = "your-vpn-server.com"
port = 443
protocol = "ssl"

[client]
name = "MyClient"
auto_reconnect = true
connection_timeout = 30

[credentials]
username = "your-username"
password = "your-password"

[network]
use_system_dns = true
routes = ["0.0.0.0/0"]
```

### Step 4: Write Your First Program

Create `main.c`:

```c
#include "rvpnse.h"
#include <stdio.h>

int main() {
    // Initialize rVPNSE
    if (rvpnse_init() != RVPNSE_SUCCESS) {
        printf("Failed to initialize rVPNSE\\n");
        return 1;
    }
    
    // Load configuration
    RvpnseConfig* config = rvpnse_config_from_file("config.toml");
    if (!config) {
        printf("Failed to load configuration\\n");
        return 1;
    }
    
    // Create client
    RvpnseClient* client = rvpnse_client_new(config);
    if (!client) {
        printf("Failed to create client\\n");
        return 1;
    }
    
    // Connect to VPN
    printf("Connecting to VPN...\\n");
    RvpnseResult result = rvpnse_client_connect(client);
    if (result == RVPNSE_SUCCESS) {
        printf("Connected successfully!\\n");
        
        // Keep connection alive for 60 seconds
        rvpnse_sleep(60000);
        
        // Disconnect
        rvpnse_client_disconnect(client);
        printf("Disconnected\\n");
    } else {
        printf("Connection failed: %s\\n", rvpnse_error_string(result));
    }
    
    // Cleanup
    rvpnse_client_free(client);
    rvpnse_config_free(config);
    rvpnse_cleanup();
    
    return 0;
}
```

### Step 5: Compile and Run

```bash
# Linux/macOS
gcc -o vpn_client main.c -L. -lrvpnse

# Windows (with MinGW)
gcc -o vpn_client.exe main.c -L. -lrvpnse

# Run
./vpn_client
```

## 🎯 Platform-Specific Quick Starts

### 📱 **Mobile Apps**

| Platform | Guide | Download |
|----------|-------|----------|
| **Android** | [Android Quick Start](../03-integration/mobile/android.md) | [Android Bundle](https://github.com/devstroop/rvpnse/releases/latest/download/rvpnse-android.tar.gz) |
| **iOS** | [iOS Quick Start](../03-integration/mobile/ios.md) | [iOS Framework](https://github.com/devstroop/rvpnse/releases/latest/download/rvpnse-ios.tar.gz) |
| **Flutter** | [Flutter Quick Start](../03-integration/mobile/flutter.md) | [Flutter Plugin](https://pub.dev/packages/rvpnse) |

### 🖥️ **Desktop Apps**

| Platform | Guide | Example |
|----------|-------|---------|
| **C/C++** | [Native Integration](../03-integration/native/cpp.md) | [C++ Example](examples/cpp-example.md) |
| **Python** | [Python Integration](../03-integration/desktop/python.md) | [Python Example](examples/python-example.md) |
| **C#/.NET** | [.NET Integration](../03-integration/desktop/dotnet.md) | [C# Example](examples/csharp-example.md) |
| **Rust** | [Rust Integration](../03-integration/native/rust.md) | [Rust Example](examples/rust-example.md) |

### 🌐 **Web Apps**

| Platform | Status | Guide |
|----------|--------|-------|
| **WebAssembly** | 🚧 Beta | [WASM Integration](../03-integration/web/wasm.md) |
| **Node.js** | 🚧 Beta | [Node.js Integration](../03-integration/web/nodejs.md) |
| **Electron** | ✅ Ready | [Electron Integration](../03-integration/web/electron.md) |

## 🔧 Development Tools

### **Build from Source**
```bash
git clone https://github.com/devstroop/rvpnse.git
cd rvpnse
cargo build --release
```

### **Testing**
```bash
# Run tests
cargo test

# Run benchmarks
cargo bench

# Check formatting
cargo fmt --check
```

### **Documentation**
```bash
# Generate Rust docs
cargo doc --open

# Serve docs locally
python3 -m http.server 8000 -d docs/
```

## 🆘 Getting Help

### **Common Issues**

| Issue | Solution |
|-------|----------|
| **Library not found** | Check library path and LD_LIBRARY_PATH |
| **Permission denied** | Run with appropriate permissions for VPN |
| **Connection timeout** | Check server configuration and firewall |
| **DNS resolution** | Verify DNS settings in configuration |

### **Support Resources**

- 📖 [Full Documentation](../README.md)
- 🐛 [Report Issues](https://github.com/devstroop/rvpnse/issues)
- 💬 [Ask Questions](https://github.com/devstroop/rvpnse/discussions)
- 📧 [Email Support](mailto:support@devstroop.com)

## 🎯 Next Steps

### **For Beginners**
1. Complete this quick start guide
2. Read [Configuration Guide](configuration.md)
3. Try [Platform Examples](examples.md)
4. Explore [API Reference](../04-api/README.md)

### **For Advanced Users**
1. Check [Advanced Topics](../06-advanced/README.md)
2. Review [Security Guide](../06-advanced/security.md)
3. Optimize [Performance](../06-advanced/performance.md)
4. Contribute to [Development](../05-build/README.md)

---

**🎉 Congratulations!** You've successfully set up rVPNSE. Now you can integrate VPN functionality into your applications with confidence.
