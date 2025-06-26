# 🛠️ Installation Guide

This guide covers installing rVPNSE on all supported platforms. Choose your platform below for specific instructions.

## 📦 Pre-built Binaries (Recommended)

The easiest way to get started is with our pre-built binaries from GitHub Releases.

### **DownloExpected output:
```
rVPNSE initialized successfully!
Version: 5.0.26
```

## 🐛 Troubleshootings**

| Platform | Architecture | Download Link |
|----------|--------------|---------------|
| **Linux** | x86_64 | [librvpnse-linux-x64.so](https://github.com/devstroop/rvpnse/releases/latest/download/librvpnse-linux-x64.so) |
| **Linux** | i686 | [librvpnse-linux-x86.so](https://github.com/devstroop/rvpnse/releases/latest/download/librvpnse-linux-x86.so) |
| **Linux** | aarch64 | [librvpnse-linux-arm64.so](https://github.com/devstroop/rvpnse/releases/latest/download/librvpnse-linux-arm64.so) |
| **Linux** | armv7 | [librvpnse-linux-armv7.so](https://github.com/devstroop/rvpnse/releases/latest/download/librvpnse-linux-armv7.so) |
| **Windows** | x86_64 | [librvpnse-windows-x64.dll](https://github.com/devstroop/rvpnse/releases/latest/download/librvpnse-windows-x64.dll) |
| **Windows** | i686 | [librvpnse-windows-x86.dll](https://github.com/devstroop/rvpnse/releases/latest/download/librvpnse-windows-x86.dll) |
| **Windows** | aarch64 | [librvpnse-windows-arm64.dll](https://github.com/devstroop/rvpnse/releases/latest/download/librvpnse-windows-arm64.dll) |
| **macOS** | Intel | [librvpnse-macos-x64.dylib](https://github.com/devstroop/rvpnse/releases/latest/download/librvpnse-macos-x64.dylib) |
| **macOS** | Apple Silicon | [librvpnse-macos-arm64.dylib](https://github.com/devstroop/rvpnse/releases/latest/download/librvpnse-macos-arm64.dylib) |

### **Mobile Bundles**

| Platform | Bundle | Download Link |
|----------|--------|---------------|
| **Android** | All ABIs | [rvpnse-android.tar.gz](https://github.com/devstroop/rvpnse/releases/latest/download/rvpnse-android.tar.gz) |
| **iOS** | Universal Framework | [rvpnse-ios.tar.gz](https://github.com/devstroop/rvpnse/releases/latest/download/rvpnse-ios.tar.gz) |

### **Header File**
All platforms need the C header file:
- [rvpnse.h](https://github.com/devstroop/rvpnse/releases/latest/download/rvpnse.h)

## 🐧 Linux Installation

### **Using curl/wget**
```bash
# Download library
curl -L -o librvpnse.so \\
  https://github.com/devstroop/rvpnse/releases/latest/download/librvpnse-linux-x64.so

# Download header
curl -L -o rvpnse.h \\
  https://github.com/devstroop/rvpnse/releases/latest/download/rvpnse.h

# Make library executable
chmod +x librvpnse.so

# Optional: Install system-wide
sudo cp librvpnse.so /usr/local/lib/
sudo cp rvpnse.h /usr/local/include/
sudo ldconfig
```

### **Package Manager Installation**
```bash
# Ubuntu/Debian (coming soon)
sudo apt install librvpnse-dev

# CentOS/RHEL (coming soon)
sudo yum install librvpnse-devel

# Arch Linux (coming soon)
sudo pacman -S librvpnse
```

### **Dependencies**
```bash
# Ubuntu/Debian
sudo apt-get install build-essential pkg-config libssl-dev

# CentOS/RHEL
sudo yum install gcc gcc-c++ pkgconfig openssl-devel

# Alpine
sudo apk add build-base pkgconfig openssl-dev
```

## 🪟 Windows Installation

### **Using PowerShell**
```powershell
# Download library
Invoke-WebRequest -Uri "https://github.com/devstroop/rvpnse/releases/latest/download/librvpnse-windows-x64.dll" -OutFile "librvpnse.dll"

# Download header
Invoke-WebRequest -Uri "https://github.com/devstroop/rvpnse/releases/latest/download/rvpnse.h" -OutFile "rvpnse.h"
```

### **Using curl (Windows)**
```cmd
curl -L -o librvpnse.dll https://github.com/devstroop/rvpnse/releases/latest/download/librvpnse-windows-x64.dll
curl -L -o rvpnse.h https://github.com/devstroop/rvpnse/releases/latest/download/rvpnse.h
```

### **Visual Studio Setup**
1. Place `librvpnse.dll` in your project directory
2. Place `rvpnse.h` in your include path
3. Add to project settings:
   - **Linker > Input > Additional Dependencies**: `librvpnse.lib`
   - **C/C++ > General > Additional Include Directories**: Path to `rvpnse.h`

### **MinGW/MSYS2**
```bash
# Install dependencies
pacman -S mingw-w64-x86_64-gcc mingw-w64-x86_64-pkg-config

# Download and compile as usual
gcc -o app.exe main.c -L. -lrvpnse
```

## 🍎 macOS Installation

### **Using curl**
```bash
# Intel Macs
curl -L -o librvpnse.dylib \\
  https://github.com/devstroop/rvpnse/releases/latest/download/librvpnse-macos-x64.dylib

# Apple Silicon Macs
curl -L -o librvpnse.dylib \\
  https://github.com/devstroop/rvpnse/releases/latest/download/librvpnse-macos-arm64.dylib

# Download header
curl -L -o rvpnse.h \\
  https://github.com/devstroop/rvpnse/releases/latest/download/rvpnse.h
```

### **Using Homebrew (coming soon)**
```bash
brew install rvpnse
```

### **Xcode Setup**
1. Add `librvpnse.dylib` to your project
2. Add `rvpnse.h` to your project
3. In Build Settings:
   - **Library Search Paths**: Add path to dylib
   - **Header Search Paths**: Add path to header

## 📱 Android Installation

### **Download Android Bundle**
```bash
curl -L -o rvpnse-android.tar.gz \\
  https://github.com/devstroop/rvpnse/releases/latest/download/rvpnse-android.tar.gz

# Extract
tar -xzf rvpnse-android.tar.gz
```

### **Gradle Integration**
```gradle
android {
    // ... other configuration
    
    sourceSets {
        main {
            jniLibs.srcDirs = ['src/main/jniLibs']
        }
    }
}
```

### **Copy Libraries**
```bash
# Copy to Android project
cp -r rvpnse-android/lib/* app/src/main/jniLibs/
```

### **Kotlin/Java Usage**
```kotlin
class RvpnseWrapper {
    companion object {
        init {
            System.loadLibrary("rvpnse")
        }
    }
    
    external fun connect(config: String): Boolean
    external fun disconnect(): Boolean
}
```

## 📱 iOS Installation

### **Download iOS Framework**
```bash
curl -L -o rvpnse-ios.tar.gz \\
  https://github.com/devstroop/rvpnse/releases/latest/download/rvpnse-ios.tar.gz

# Extract
tar -xzf rvpnse-ios.tar.gz
```

### **Xcode Integration**
1. Drag `RvpnseFramework.framework` into your Xcode project
2. In **Build Phases > Link Binary With Libraries**: Add the framework
3. In **Build Settings > Framework Search Paths**: Add framework path

### **Swift Usage**
```swift
import RvpnseFramework

class VPNManager {
    private var client: OpaquePointer?
    
    func connect() {
        let config = rvpnse_config_from_file("config.toml")
        client = rvpnse_client_new(config)
        rvpnse_client_connect(client)
    }
}
```

## 🏗️ Build from Source

### **Prerequisites**
- Rust 1.82.0 or later
- C compiler (GCC, Clang, MSVC)
- CMake 3.15+
- Git

### **Clone and Build**
```bash
# Clone repository
git clone https://github.com/devstroop/rvpnse.git
cd rvpnse

# Build release version
cargo build --release

# Built libraries will be in target/release/
```

### **Build for Specific Platform**
```bash
# Install target
rustup target add x86_64-unknown-linux-gnu

# Cross-compile
cargo build --target x86_64-unknown-linux-gnu --release
```

### **Build All Platforms**
```bash
# Use our build script
python3 build.py --all --release
```

## 🧪 Verify Installation

### **Test C Program**
```c
#include "rvpnse.h"
#include <stdio.h>

int main() {
    if (rvpnse_init() == rVPNSE_SUCCESS) {
        printf("rVPNSE initialized successfully!\\n");
        const char* version = rvpnse_version();
        printf("Version: %s\\n", version);
        rvpnse_cleanup();
        return 0;
    } else {
        printf("Failed to initialize rVPNSE\\n");
        return 1;
    }
}
```

### **Compile and Test**
```bash
# Linux/macOS
gcc -o test test.c -L. -lrvpnse
./test

# Windows
gcc -o test.exe test.c -L. -lrvpnse
test.exe
```

Expected output:
```
rVPNSE initialized successfully!
Version: 0.1.0
```

## 🐛 Troubleshooting

### **Library Not Found**
```bash
# Linux: Add to library path
export LD_LIBRARY_PATH=.:$LD_LIBRARY_PATH

# macOS: Add to library path
export DYLD_LIBRARY_PATH=.:$DYLD_LIBRARY_PATH

# Or install system-wide
sudo cp librvpnse.* /usr/local/lib/
sudo ldconfig  # Linux only
```

### **Permission Denied**
```bash
# Make library executable
chmod +x librvpnse.*

# On macOS, you might need to allow unsigned binaries
sudo spctl --master-disable
```

### **Missing Dependencies**
```bash
# Check dependencies (Linux)
ldd librvpnse.so

# Check dependencies (macOS)
otool -L librvpnse.dylib

# Check dependencies (Windows)
dumpbin /dependents librvpnse.dll
```

## 🎯 Next Steps

✅ **Installation Complete!** Now you can:

1. 📖 **Configure rVPNSE**: [Configuration Guide](configuration.md)
2. 🚀 **Make First Connection**: [First Connection](first-connection.md)
3. 📱 **Platform Integration**: [Integration Guides](../03-integration/README.md)
4. 📚 **API Reference**: [API Documentation](../04-api/README.md)

---

**Need Help?** Check our [Troubleshooting Guide](../07-troubleshooting/README.md) or [open an issue](https://github.com/devstroop/rvpnse/issues).
