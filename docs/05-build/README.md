# Building rVPNSE

## 🚀 Quick Start

rVPNSE uses a unified Python build system for all platforms. You can build for your current platform in seconds:

```bash
# Build for current platform (debug mode)
python3 tools/build.py

# Build for current platform (release mode)
python3 tools/build.py --mode release

# List all available targets
python3 tools/build.py --list
```

## 📋 Prerequisites

### Required Tools
- **Python 3.7+** - Build system
- **Rust 1.70+** - Core library compilation
- **Git** - Source code management

### Platform-Specific Requirements

#### Windows
- **Visual Studio Build Tools** or **Visual Studio 2019/2022**
- **Windows SDK** (latest version)
- **TAP-Windows driver** (for TUN/TAP functionality)

#### macOS
- **Xcode Command Line Tools**: `xcode-select --install`
- **Homebrew** (recommended): `/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"`

#### Linux
- **GCC/Clang** build toolchain
- **pkg-config** and **build-essential**
```bash
# Ubuntu/Debian
sudo apt update && sudo apt install build-essential pkg-config

# RHEL/CentOS/Fedora
sudo dnf groupinstall "Development Tools"
sudo dnf install pkg-config
```

#### Android (Cross-compilation)
- **Android NDK r25+**
- **Android SDK** (if building sample apps)
- **Java 11+** for Gradle builds

#### iOS (Cross-compilation, macOS only)
- **Xcode 14+** with iOS SDK
- **iOS deployment targets**: iOS 12.0+
- **Valid Apple Developer account** (for device deployment)

## 🏗️ Build System Architecture

The `tools/build.py` script provides a unified interface for all compilation tasks:

```
tools/build.py
├── Target Detection    - Automatic platform detection
├── Dependency Check    - Validates toolchain availability
├── Cross-compilation   - Android, iOS, Windows, Linux
├── Artifact Packaging - Static/dynamic libraries
└── Integration Tests   - Validates build outputs
```

## 🎯 Platform Targets

### Desktop Platforms

```bash
# Current platform (auto-detected)
python3 tools/build.py

# Specific desktop targets
python3 tools/build.py --targets macos-arm64 macos-x64 linux-x64 windows-x64

# All desktop platforms
python3 tools/build.py --all-desktop
```

### Mobile Platforms

```bash
# All Android architectures
python3 tools/build.py --all-android

# Specific Android targets
python3 tools/build.py --targets android-arm64 android-armv7 android-x86_64

# iOS (macOS only)
python3 tools/build.py --targets ios-arm64 ios-simulator-x64
```

### Build Modes

```bash
# Debug build (default) - includes debug symbols, no optimization
python3 tools/build.py --mode debug

# Release build - optimized, stripped symbols
python3 tools/build.py --mode release

# Development build - optimized but with debug info
python3 tools/build.py --mode dev
```

## 📦 Output Artifacts

Build outputs are organized in the `dist/` directory:

```
dist/
├── macos-arm64/
│   ├── librvpnse.dylib      # Dynamic library
│   ├── librvpnse.a          # Static library
│   └── rvpnse.h             # C header
├── linux-x64/
│   ├── librvpnse.so         # Dynamic library
│   ├── librvpnse.a          # Static library
│   └── rvpnse.h             # C header
├── windows-x64/
│   ├── rvpnse.dll           # Dynamic library
│   ├── rvpnse.lib           # Import library
│   ├── librvpnse.a          # Static library
│   └── rvpnse.h             # C header
└── android-arm64/
    ├── librvpnse.so         # Android shared library
    └── rvpnse.h             # C header
```

## 🔧 Advanced Build Options

### Custom Features

```bash
# Build with specific Cargo features
python3 tools/build.py --features "async,logging"

# Build without default features
python3 tools/build.py --no-default-features

# Combine custom features
python3 tools/build.py --no-default-features --features "core,tls"
```

### Environment Configuration

```bash
# Custom NDK path for Android
export ANDROID_NDK_HOME=/path/to/ndk
python3 tools/build.py --targets android-arm64

# Custom Rust toolchain
export RUSTUP_TOOLCHAIN=nightly
python3 tools/build.py --mode release

# Verbose output
python3 tools/build.py --verbose
```

### Clean Builds

```bash
# Clean all build artifacts
python3 tools/build.py --clean

# Clean and rebuild
python3 tools/build.py --clean --mode release

# Clean specific target
python3 tools/build.py --clean --targets android-arm64
```

## 🧪 Testing Builds

### Quick Validation

```bash
# Build and run basic tests
python3 tools/build.py --test

# Build specific target and validate
python3 tools/build.py --targets linux-x64 --test
```

### Integration Testing

```bash
# Run comprehensive test suite
cd tests/
./run_integration_tests.sh

# Test C FFI interface
make -C tests/c_integration
./tests/c_integration/test_runner
```

## 📊 Build Performance

### Optimization Tips

1. **Parallel Builds**: The build system automatically uses all available CPU cores
2. **Incremental Builds**: Only modified components are rebuilt
3. **Target Caching**: Build artifacts are cached per target
4. **Dependency Caching**: External dependencies are cached globally

### Build Times (Reference)

| Target | Mode | Time (M1 MacBook Pro) | Time (Intel i7) |
|--------|------|----------------------|-----------------|
| macOS ARM64 | Debug | ~45s | ~60s |
| macOS ARM64 | Release | ~90s | ~120s |
| Linux x64 | Debug | ~50s | ~65s |
| Android ARM64 | Release | ~75s | ~100s |
| All Desktop | Release | ~180s | ~240s |

## 🚨 Troubleshooting

### Common Issues

#### "Rust toolchain not found"
```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

#### "Android NDK not found"
```bash
# Set NDK path explicitly
export ANDROID_NDK_HOME=/path/to/android-ndk-r25c
python3 tools/build.py --targets android-arm64
```

#### "Permission denied" on Linux/macOS
```bash
# Make build script executable
chmod +x build.py
python3 tools/build.py
```

#### "Visual Studio not found" on Windows
```bash
# Install Visual Studio Build Tools
# Or set VS environment variables
call "C:\Program Files\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
python build.py
```

### Debug Build Issues

```bash
# Enable verbose logging
python3 tools/build.py --verbose --targets your-target

# Check system requirements
python3 tools/build.py --check-deps

# Validate environment
python3 tools/build.py --doctor
```

## 🔄 Continuous Integration

The build system is designed for CI/CD environments:

```yaml
# GitHub Actions example
- name: Build rVPNSE
  run: |
    python3 tools/build.py --all-desktop --mode release
    python3 tools/build.py --test
```

### CI-Specific Options

```bash
# Non-interactive mode (no prompts)
python3 tools/build.py --ci --mode release

# Generate build reports
python3 tools/build.py --report --output build-report.json

# Fail fast on errors
python3 tools/build.py --fail-fast
```

## 📖 Integration Examples

After building, integrate the library into your application:

- **[C/C++ Integration](../integration/c-cpp.md)**
- **[Flutter Integration](../integration/flutter.md)**
- **[iOS Integration](../integration/ios.md)**
- **[Android Integration](../integration/android.md)**

---

**Need help?** Check the [troubleshooting guide](../troubleshooting.md) or [open an issue](https://github.com/devstroop/rvpnse/issues).
