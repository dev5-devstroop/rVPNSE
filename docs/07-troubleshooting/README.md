# 🐛 Troubleshooting Guide

Comprehensive troubleshooting guide for rVPNSE. Find solutions to common issues and learn debugging techniques.

## 📑 Contents

- [🚨 Common Issues](common-issues.md) - Most frequent problems and solutions
- [🔍 Debugging Guide](debugging.md) - How to debug rVPNSE issues
- [📊 Performance Issues](performance.md) - Performance troubleshooting
- [🔒 Security Issues](security.md) - Security-related problems
- [📱 Platform-Specific](platform-issues.md) - Platform-specific troubleshooting
- [❓ FAQ](faq.md) - Frequently asked questions

## 🚨 Quick Solutions

### **Connection Issues**

| Problem | Quick Fix |
|---------|-----------|
| **Connection timeout** | Check server address and firewall settings |
| **Authentication failed** | Verify username/password and server configuration |
| **DNS resolution fails** | Try using IP address instead of hostname |
| **Certificate errors** | Check system time and certificate validity |

### **Installation Issues**

| Problem | Quick Fix |
|---------|-----------|
| **Library not found** | Add library path to LD_LIBRARY_PATH/PATH |
| **Permission denied** | Run with appropriate permissions or check file permissions |
| **Missing dependencies** | Install required system dependencies |
| **Version conflicts** | Use compatible library versions |

### **Performance Issues**

| Problem | Quick Fix |
|---------|-----------|
| **Slow connection** | Check MTU settings and network configuration |
| **High CPU usage** | Disable debug logging in production |
| **Memory leaks** | Ensure proper cleanup of rVPNSE objects |
| **Frequent disconnections** | Increase keepalive interval |

## 🔍 Diagnostic Tools

### **Built-in Diagnostics**
```c
// Check library status
const char* version = rvpnse_version();
printf("rVPNSE Version: %s\\n", version);

// Test basic functionality
RvpnseResult result = rvpnse_self_test();
if (result == rVPNSE_SUCCESS) {
    printf("Self-test passed\\n");
} else {
    printf("Self-test failed: %s\\n", rvpnse_error_string(result));
}

// Get detailed error information
RvpnseErrorInfo error_info;
rvpnse_get_last_error(&error_info);
printf("Last error: %s (code: %d)\\n", error_info.message, error_info.code);
```

### **Configuration Validation**
```c
// Validate configuration file
RvpnseConfig* config = rvpnse_config_from_file("config.toml");
if (!config) {
    RvpnseConfigError error;
    rvpnse_config_get_last_error(&error);
    printf("Config error at line %d: %s\\n", error.line, error.message);
}

// Validate specific settings
RvpnseResult result = rvpnse_config_validate(config);
if (result != rVPNSE_SUCCESS) {
    printf("Configuration validation failed\\n");
}
```

### **Network Diagnostics**
```c
// Test server connectivity
RvpnseResult result = rvpnse_test_connectivity("vpn.example.com", 443);
if (result == rVPNSE_SUCCESS) {
    printf("Server is reachable\\n");
} else {
    printf("Cannot reach server\\n");
}

// Check local network interface
RvpnseNetworkInfo net_info;
rvpnse_get_network_info(&net_info);
printf("Default gateway: %s\\n", net_info.default_gateway);
printf("DNS servers: %s\\n", net_info.dns_servers);
```

## 📊 Debug Logging

### **Enable Debug Logging**
```toml
[logging]
level = "debug"
output = "file"
format = "json"

[logging.file]
path = "/tmp/rvpnse-debug.log"
max_size = "50MB"
max_files = 3
```

### **C API Logging**
```c
// Set log level at runtime
rvpnse_set_log_level(rVPNSE_LOG_DEBUG);

// Set custom log handler
void my_log_handler(RvpnseLogLevel level, const char* message, void* userdata) {
    printf("[%s] %s\\n", rvpnse_log_level_string(level), message);
}

rvpnse_set_log_handler(my_log_handler, NULL);
```

### **Rust API Logging**
```rust
use rvpnse::logging::{Logger, LogLevel};

// Initialize logging
Logger::init()
    .level(LogLevel::Debug)
    .output_file("/tmp/rvpnse.log")
    .format_json()
    .build();

// Log custom messages
log::debug!("Custom debug message");
log::info!("Connection attempt starting");
```

## 🔧 Environment Variables

### **Debug Variables**
```bash
# Enable debug logging
export rVPNSE_LOG_LEVEL=debug

# Enable trace logging (very verbose)
export rVPNSE_LOG_LEVEL=trace

# Log to file
export rVPNSE_LOG_FILE=/tmp/rvpnse.log

# Enable performance profiling
export rVPNSE_PROFILE=1

# Disable hardware acceleration
export rVPNSE_NO_HARDWARE_ACCEL=1
```

### **Network Variables**
```bash
# Force IPv4 only
export rVPNSE_IPV4_ONLY=1

# Custom DNS servers
export rVPNSE_DNS_SERVERS="8.8.8.8,8.8.4.4"

# Connection timeout
export rVPNSE_CONNECT_TIMEOUT=60

# Keepalive interval
export rVPNSE_KEEPALIVE=30
```

## 📱 Platform-Specific Issues

### **Android**
```kotlin
// Check VPN permissions
fun checkVpnPermission(): Boolean {
    val intent = VpnService.prepare(this)
    return intent == null
}

// Request VPN permission
fun requestVpnPermission() {
    val intent = VpnService.prepare(this)
    if (intent != null) {
        startActivityForResult(intent, VPN_REQUEST_CODE)
    }
}

// Check network security config
// Ensure network_security_config.xml allows cleartext traffic if needed
```

### **iOS**
```swift
// Check NetworkExtension permissions
func checkVPNPermission() {
    NEVPNManager.shared().loadFromPreferences { error in
        if let error = error {
            print("Failed to load VPN preferences: \\(error)")
        }
    }
}

// Enable App Transport Security exceptions if needed
// Add to Info.plist:
/*
<key>NSAppTransportSecurity</key>
<dict>
    <key>NSAllowsArbitraryLoads</key>
    <true/>
</dict>
*/
```

### **Linux**
```bash
# Check required capabilities
getcap /path/to/your/app

# Add required capabilities
sudo setcap cap_net_admin+ep /path/to/your/app

# Check TUN/TAP module
lsmod | grep tun

# Load TUN/TAP module if needed
sudo modprobe tun
```

### **Windows**
```cmd
REM Check administrator privileges
net session >nul 2>&1
if %errorLevel% neq 0 (
    echo Administrator privileges required
)

REM Check WinTUN driver
sc query wintun

REM Install WinTUN driver if needed
wintun-installer.exe
```

## 🛠️ Advanced Debugging

### **Memory Debugging**
```bash
# Run with Valgrind (Linux)
valgrind --leak-check=full --track-origins=yes ./your_app

# Run with AddressSanitizer
export ASAN_OPTIONS=detect_leaks=1
gcc -fsanitize=address -g -o your_app main.c -lrvpnse
./your_app
```

### **Network Debugging**
```bash
# Capture network traffic
sudo tcpdump -i any -w rvpnse-traffic.pcap port 443

# Monitor system calls
strace -e trace=network ./your_app

# Monitor file descriptors
lsof -p $(pgrep your_app)
```

### **Performance Profiling**
```bash
# CPU profiling with perf
perf record ./your_app
perf report

# Memory profiling with heaptrack
heaptrack ./your_app
heaptrack_gui heaptrack.your_app.*.gz
```

## 📋 Diagnostic Checklist

### **Before Reporting Issues**

- [ ] Check rVPNSE version: `rvpnse_version()`
- [ ] Verify configuration: `rvpnse_config_validate()`
- [ ] Test connectivity: `rvpnse_test_connectivity()`
- [ ] Check logs: Enable debug logging
- [ ] Test with minimal config: Remove optional settings
- [ ] Verify permissions: Ensure proper VPN permissions
- [ ] Check dependencies: Ensure all required libraries are installed
- [ ] Test on different network: Rule out network-specific issues

### **Information to Include in Bug Reports**

1. **Environment**
   - Operating system and version
   - rVPNSE version
   - Compiler and version (if building from source)
   - Architecture (x86, x64, ARM)

2. **Configuration**
   - Anonymized configuration file
   - Environment variables used
   - Command line arguments

3. **Logs**
   - Full debug logs
   - Error messages
   - Stack traces (if available)

4. **Steps to Reproduce**
   - Minimal code example
   - Exact steps taken
   - Expected vs actual behavior

## 🆘 Getting Help

### **Self-Service Options**
1. Search [existing issues](https://github.com/devstroop/rvpnse/issues)
2. Check [FAQ](faq.md)
3. Review [documentation](../README.md)
4. Try [diagnostic tools](#diagnostic-tools)

### **Community Support**
1. [GitHub Discussions](https://github.com/devstroop/rvpnse/discussions)
2. [Stack Overflow](https://stackoverflow.com/questions/tagged/rvpnse)
3. [Reddit Community](https://reddit.com/r/rvpnse)

### **Professional Support**
1. [Email Support](mailto:support@devstroop.com)
2. [Commercial Support Plans](https://devstroop.com/support)
3. [Consulting Services](https://devstroop.com/consulting)

## 🎯 Next Steps

Based on your issue type:

- **Connection problems** → [Common Issues](common-issues.md)
- **Performance issues** → [Performance Guide](performance.md)
- **Security concerns** → [Security Issues](security.md)
- **Platform-specific** → [Platform Issues](platform-issues.md)
- **Still stuck?** → [Contact Support](#getting-help)

---

**💡 Tip**: Keep this troubleshooting guide handy during development and deployment for quick reference!
