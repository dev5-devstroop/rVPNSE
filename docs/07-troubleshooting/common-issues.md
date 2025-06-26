# rVPNSE Troubleshooting Guide

This guide covers common issues you might encounter when building, integrating, or using rVPNSE.

## 🚨 Quick Diagnostics

Run these commands to quickly identify common issues:

```bash
# Check build environment
python3 build.py --doctor

# Validate dependencies
python3 build.py --check-deps

# Test basic functionality
python3 build.py --test
```

## 🔄 CI/CD and Release Issues

### GitHub Actions Permission Errors

#### "Write access to repository not granted" (403 Error)
This occurs when GitHub Actions lacks permission to push commits or create releases.

**Solution:**
1. **Check Repository Settings:**
   - Go to Settings → Actions → General
   - Under "Workflow permissions", select "Read and write permissions"
   - Check "Allow GitHub Actions to create and approve pull requests"

2. **Verify Workflow Permissions:**
   ```yaml
   permissions:
     contents: write
     pull-requests: read
     actions: read
   ```

3. **Check Token Usage:**
   ```yaml
   - name: Checkout code
     uses: actions/checkout@v4
     with:
       token: ${{ secrets.GITHUB_TOKEN }}
   ```

#### "Resource not accessible by integration"
**Common Causes:**
- Workflow running on forked repository
- Insufficient token permissions
- Repository settings restricting Actions

**Solution:**
```yaml
# In workflow file, ensure proper permissions
permissions:
  contents: write
  issues: write
  pull-requests: write
```

#### Release Workflow Fails on Version Bump
**Error:** `fatal: unable to access 'https://github.com/...': The requested URL returned error: 403`

**Solution:**
```yaml
# Use proper git configuration
- name: Configure Git
  run: |
    git config --local user.email "41898282+github-actions[bot]@users.noreply.github.com"
    git config --local user.name "github-actions[bot]"
    
# Push with explicit branch reference
- name: Push Changes
  run: |
    git push origin HEAD:${{ github.ref_name }}
    git push origin "v$NEW_VERSION"
```

#### Deprecated Actions Warnings
Replace deprecated actions with modern alternatives:

```yaml
# OLD (deprecated)
- uses: actions/create-release@v1

# NEW (recommended)  
- name: Create Release
  env:
    GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
  run: |
    gh release create "$VERSION" \
      --title "Release $VERSION" \
      --notes "Release notes here"
```

### Build Matrix Failures

#### Platform-Specific Build Failures
**Check Matrix Configuration:**
```yaml
strategy:
  fail-fast: false  # Continue other builds if one fails
  matrix:
    include:
      - os: ubuntu-latest
        target: x86_64-unknown-linux-gnu
      - os: macos-latest
        target: x86_64-apple-darwin
```

#### Cross-Compilation Issues
```bash
# Add required targets
rustup target add aarch64-apple-darwin
rustup target add x86_64-pc-windows-gnu

# Install cross-compilation tools
cargo install cross
```

## 🏗️ Build Issues

### Rust Toolchain Problems

#### "cargo: command not found"
```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Or on Windows (PowerShell)
Invoke-WebRequest -Uri https://win.rustup.rs/ -OutFile rustup-init.exe
.\rustup-init.exe
```

#### "rustc version too old"
```bash
# Update Rust to latest stable
rustup update stable
rustup default stable

# Verify version (should be 1.70+)
rustc --version
```

#### "linker `cc` not found"
```bash
# Ubuntu/Debian
sudo apt update && sudo apt install build-essential

# CentOS/RHEL/Fedora
sudo dnf groupinstall "Development Tools"

# macOS
xcode-select --install

# Arch Linux
sudo pacman -S base-devel
```

### Platform-Specific Build Issues

#### Windows Build Failures

**"MSVC not found"**
```bash
# Install Visual Studio Build Tools
# Download from: https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022

# Or set VS environment (if installed)
call "C:\Program Files\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
```

**"Windows SDK not found"**
```bash
# Install latest Windows SDK
# Download from: https://developer.microsoft.com/en-us/windows/downloads/windows-sdk/

# Or set SDK path
set WINDOWS_SDK_DIR=C:\Program Files (x86)\Windows Kits\10
```

#### macOS Build Failures

**"xcrun: error: invalid active developer path"**
```bash
# Install/reinstall Xcode command line tools
sudo xcode-select --reset
xcode-select --install
```

**"No available toolchain for target"**
```bash
# Add iOS targets for cross-compilation
rustup target add aarch64-apple-ios
rustup target add x86_64-apple-ios
```

#### Linux Build Failures

**"error: Microsoft C++ Build Tools is required"**
```bash
# This error on Linux indicates cross-compilation to Windows
# Install mingw-w64 for Windows cross-compilation
sudo apt install mingw-w64  # Ubuntu/Debian
sudo dnf install mingw64-gcc  # Fedora
```

#### Android Build Failures

**"Android NDK not found"**
```bash
# Download and install Android NDK
# From: https://developer.android.com/ndk/downloads

# Set NDK path
export ANDROID_NDK_HOME=/path/to/android-ndk-r25c

# Add NDK to PATH
export PATH=$ANDROID_NDK_HOME:$PATH
```

**"Target not found: aarch64-linux-android"**
```bash
# Add Android targets
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add i686-linux-android
rustup target add x86_64-linux-android
```

## 🔗 Integration Issues

### C/C++ Integration

#### "Undefined reference to rvpnse_*"
```c
// Make sure to link the library
gcc -o myapp myapp.c -L./dist/linux-x64 -lrvpnse

// Or for static linking
gcc -o myapp myapp.c ./dist/linux-x64/librvpnse.a
```

#### "Header file not found"
```c
// Include path must point to the header
#include "dist/linux-x64/rvpnse.h"

// Or copy header to system include path
// sudo cp dist/linux-x64/rvpnse.h /usr/local/include/
```

### Flutter Integration

#### "Failed to load dynamic library"
```dart  
// Make sure library is in the correct location
// Android: android/app/src/main/jniLibs/[arch]/librvpnse.so
// iOS: ios/Frameworks/librvpnse.dylib

// Check library permissions
// chmod +x librvpnse.so  # Linux/Android
```

#### "Symbol not found in library"  
```dart
// Verify function names match exactly
final int Function() initVpn = nativeLib
    .lookup<NativeFunction<Int32 Function()>>("rvpnse_init")
    .asFunction();

// Check if library was built correctly
nm -D librvpnse.so | grep rvpnse  # Linux
objdump -t librvpnse.so | grep rvpnse  # Alternative
```

### iOS Integration

#### "Code signing failed"
```bash
# Ensure proper iOS development setup
# 1. Valid Apple Developer account
# 2. Provisioning profile includes device UDIDs
# 3. Code signing identity in keychain

# Check code signing
codesign -dv --verbose=4 YourApp.app
```

#### "NetworkExtension capability missing"
```xml
<!-- Add to iOS project entitlements -->
<key>com.apple.developer.networking.networkextension</key>
<array>
    <string>packet-tunnel-provider</string>
</array>
```

### Android Integration

#### "VpnService permission denied"
```xml
<!-- Add to AndroidManifest.xml -->
<uses-permission android:name="android.permission.BIND_VPN_SERVICE" />

<!-- In your service declaration -->
<service android:name=".VpnService"
         android:permission="android.permission.BIND_VPN_SERVICE">
    <intent-filter>
        <action android:name="android.net.VpnService" />
    </intent-filter>
</service>
```

#### "UnsatisfiedLinkError: dlopen failed"
```java
// Ensure library is packaged correctly
// Check build.gradle for proper ndk configuration
android {
    ndkVersion "25.2.9519653"
    
    defaultConfig {
        ndk {
            abiFilters 'arm64-v8a', 'armeabi-v7a', 'x86_64'
        }
    }
}
```

## ⚡ Runtime Issues

### Connection Problems

#### "Connection timeout"
```toml
# Increase timeout in configuration
[connection]
timeout_seconds = 30
keepalive_interval = 10

# Check network connectivity
ping your-vpn-server.com
telnet your-vpn-server.com 443
```

#### "Authentication failed"
```toml
# Verify credentials in config
[authentication]
username = "your_username"
password = "your_password"
hub_name = "VPN"

# Test credentials with SoftEther VPN Client
```

#### "Certificate verification failed"
```toml
# For testing, disable certificate verification
[tls]
verify_certificate = false

# For production, add custom CA or use proper certificates
ca_certificate_path = "/path/to/ca.pem"
```

### Platform-Specific Runtime Issues

#### Windows: "Access denied"
```bash
# Run as Administrator
# Right-click -> "Run as administrator"

# Or use elevated PowerShell
Start-Process PowerShell -Verb RunAs
```

#### macOS: "Operation not permitted"
```bash
# Grant necessary permissions
sudo chown root:wheel /path/to/your/app
sudo chmod +s /path/to/your/app

# Or use proper entitlements for sandboxed apps
```

#### Linux: "Permission denied" for TUN/TAP
```bash
# Add CAP_NET_ADMIN capability
sudo setcap cap_net_admin+ep /path/to/your/app

# Or run with sudo (not recommended for production)
sudo ./your_app

# Or add user to specific groups
sudo usermod -a -G netdev $USER
```

## 🐛 Debugging

### Enable Verbose Logging

```rust
// Set environment variable for detailed logs
export RUST_LOG=debug

// Or in your application
use log::LevelFilter;
env_logger::Builder::from_default_env()
    .filter_level(LevelFilter::Debug)
    .init();
```

### Debug Build Information

```bash
# Build with debug symbols
python3 build.py --mode debug

# Check library information
file dist/linux-x64/librvpnse.so
ldd dist/linux-x64/librvpnse.so  # Linux dependencies
otool -L dist/macos-arm64/librvpnse.dylib  # macOS dependencies
```

### Memory Debugging

```bash
# Use Valgrind on Linux
valgrind --leak-check=full ./your_app

# Use AddressSanitizer during build
export RUSTFLAGS="-Z sanitizer=address"
python3 build.py --mode debug
```

## 📊 Performance Issues

### Connection Slow

```toml
# Optimize connection settings
[connection]
keepalive_interval = 5  # Reduce for faster detection
timeout_seconds = 15    # Reduce for faster failures

# Use faster cipher suites
[tls]
cipher_suites = ["TLS_AES_128_GCM_SHA256"]
```

### High Memory Usage

```rust
// Monitor memory usage
let client = rvpnse_client_new(config);
// ... use client
rvpnse_client_free(client);  // Always free resources
```

### High CPU Usage

```bash
# Profile with perf (Linux)
perf record -g ./your_app
perf report

# Profile with Instruments (macOS)
# Use Xcode -> Open Developer Tool -> Instruments
```

## 🆘 Getting Further Help

### Information to Include in Bug Reports

1. **Environment Information**:
   ```bash
   # OS and version
   uname -a
   
   # Rust version
   rustc --version
   
   # Build command used
   python3 build.py --verbose --targets your-target
   ```

2. **Library Information**:
   ```bash
   # Library details
   file dist/your-target/librvpnse.*
   nm -D dist/your-target/librvpnse.so | head -20
   ```

3. **Configuration** (sanitize sensitive data):
   ```toml
   # Include relevant parts of your TOML config
   [server]
   host = "vpn.example.com"  # Replace with actual (if not sensitive)
   port = 443
   ```

4. **Error Messages**: Full error output with stack traces

5. **Minimal Reproduction**: Smallest possible code that demonstrates the issue

### Where to Get Help

- **Documentation**: Start with this troubleshooting guide
- **GitHub Issues**: [Report bugs and request features](https://github.com/devstroop/rvpnse/issues)
- **Discussions**: [Community Q&A](https://github.com/devstroop/rvpnse/discussions)
- **Security Issues**: Email security issues to security@devstroop.com

### Before Reporting Issues

1. ✅ Check this troubleshooting guide
2. ✅ Search existing GitHub issues
3. ✅ Try with the latest version
4. ✅ Test with minimal configuration
5. ✅ Include all requested information

---

**Still having issues?** [Open a GitHub issue](https://github.com/devstroop/rvpnse/issues/new) with detailed information.
