# Quick Start Guide

Get Rust VPNSE up and running in 5 minutes! This guide will help you build the library and test basic functionality.

## 📋 Prerequisites

- **Rust 1.70+** with `cargo`
- **C compiler** (gcc, clang, or MSVC)
- **Git** for cloning the repository

### **Install Rust**
```bash
# Install Rust if you haven't already
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

## 🚀 Step 1: Clone and Build

```bash
# Clone the repository
git clone https://github.com/your-org/rvpnse.git
cd rvpnse

# Build the static library
cargo build --release

# Verify the build
ls -la target/release/
```

### **Expected Output**
```
target/release/
├── librvpnse.a         # Static library (Unix)
├── librvpnse.so        # Shared library (Unix)
├── rvpnse.lib          # Static library (Windows)
├── rvpnse.dll          # Shared library (Windows)
└── deps/                   # Dependencies
```

## 🧪 Step 2: Test Configuration Parsing

Create a test configuration file:

```bash
cat > test_config.toml << EOF
[server]
hostname = "vpn.example.com"
port = 443
hub = "VPN"

[auth]
method = "password"
username = "testuser"
password = "testpass"

[vpn]
auto_reconnect = true
keepalive_interval = 60
EOF
```

Test the configuration:

```bash
# Run the configuration test
cargo run --example config_test test_config.toml
```

### **Expected Output**
```
✅ Configuration loaded successfully
Server: vpn.example.com:443
Hub: VPN
Auth: Password (testuser)
Auto-reconnect: true
```

## 🔧 Step 3: Test C FFI Interface

Generate the C header:

```bash
# Install cbindgen if needed
cargo install cbindgen

# Generate C header
cbindgen --config cbindgen.toml --crate rvpnse-rust --output include/rvpnse.h

# Verify header was created
ls -la include/rvpnse.h
```

Create a simple C test:

```bash
cat > test_ffi.c << 'EOF'
#include "include/rvpnse.h"
#include <stdio.h>
#include <stdlib.h>

int main() {
    printf("Testing Rust VPNSE C FFI...\n");
    
    // Test configuration parsing
    const char* config = "[server]\nhostname = \"test.example.com\"\nport = 443\nhub = \"VPN\"\n";
    char error_msg[256];
    
    int result = vpnse_parse_config(config, error_msg, sizeof(error_msg));
    
    if (result == VPNSE_SUCCESS) {
        printf("✅ Configuration parsing: SUCCESS\n");
    } else {
        printf("❌ Configuration parsing failed: %s\n", error_msg);
        return 1;
    }
    
    // Test client creation
    vpnse_client_t* client = vpnse_client_new(config);
    if (client) {
        printf("✅ Client creation: SUCCESS\n");
        vpnse_client_free(client);
    } else {
        printf("❌ Client creation: FAILED\n");
        return 1;
    }
    
    printf("🎉 All tests passed!\n");
    return 0;
}
EOF
```

Compile and run the C test:

```bash
# Compile the test
gcc test_ffi.c -L./target/release -lrvpnse -o test_ffi

# Run the test
./test_ffi
```

### **Expected Output**
```
Testing Rust VPNSE C FFI...
✅ Configuration parsing: SUCCESS
✅ Client creation: SUCCESS
🎉 All tests passed!
```

## 🌐 Step 4: Test Real Connection (Optional)

If you have access to a SoftEther server, you can test a real connection:

```bash
# Update test_config.toml with real server details
cat > real_server.toml << EOF
[server]
hostname = "your-server.com"
port = 443
hub = "VPN"

[auth]
method = "password"
username = "your-username"
password = "your-password"

[vpn]
auto_reconnect = true
keepalive_interval = 60
EOF

# Test connection (this will connect but not create TUN/TAP)
cargo run --example connection_test real_server.toml
```

### **Expected Output**
```
🔗 Connecting to your-server.com:443...
✅ Connected successfully
🔐 Authenticating with username/password...
✅ Authentication successful
ℹ️  Connection established (no TUN/TAP interface created)
📊 Server info: SoftEther VPN 4.x
🔌 Disconnecting...
✅ Disconnected cleanly
```

## 📱 Step 5: Choose Your Integration Path

Now that Rust VPNSE is working, choose your platform integration:

### **Mobile Development**
- **[iOS Integration](integration/ios.md)** - Swift + NetworkExtension
- **[Android Integration](integration/android.md)** - Kotlin + VpnService
- **[Flutter Integration](integration/flutter.md)** - Dart FFI for cross-platform

### **Desktop Development**
- **[C/C++ Integration](integration/c-cpp.md)** - Native applications
- **[.NET Integration](integration/dotnet.md)** - C# applications
- **[Python Integration](integration/python.md)** - Python applications

### **Platform-Specific Guides**
- **[Windows Platform](platforms/windows.md)** - WinTUN integration
- **[macOS Platform](platforms/macos.md)** - utun integration
- **[Linux Platform](platforms/linux.md)** - TUN/TAP integration

## 🚨 Common Issues

### **Build Failures**
```bash
# Update Rust
rustup update

# Clean and rebuild
cargo clean
cargo build --release
```

### **C Compilation Issues**
```bash
# Make sure you have a C compiler
# Linux: sudo apt install build-essential
# macOS: xcode-select --install
# Windows: Install Visual Studio Build Tools
```

### **FFI Test Failures**
```bash
# Make sure library path is correct
export LD_LIBRARY_PATH=$PWD/target/release:$LD_LIBRARY_PATH

# On macOS, you might need:
export DYLD_LIBRARY_PATH=$PWD/target/release:$DYLD_LIBRARY_PATH
```

## ✅ Success Checklist

- [ ] Rust VPNSE builds successfully
- [ ] Configuration parsing works
- [ ] C FFI interface works
- [ ] Basic client creation works
- [ ] (Optional) Real server connection works

## 📚 Next Steps

1. **Read** the [Configuration Guide](configuration.md) to understand all options
2. **Choose** your platform integration guide
3. **Implement** platform-specific networking
4. **Test** with your target application

## 🆘 Need Help?

- **[Troubleshooting](advanced/troubleshooting.md)** - Common issues and solutions
- **[Contributing](../CONTRIBUTING.md)** - Report bugs or contribute
- **[API Reference](api/c-api.md)** - Complete API documentation

---

**🎉 Congratulations! Rust VPNSE is ready for integration.**
