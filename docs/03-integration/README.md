# 🔗 Integration Guides

Complete integration guides for all platforms and programming languages. Choose your platform and language below.

## 📑 Contents

- [📱 Mobile Applications](mobile/README.md) - iOS, Android, Flutter
- [🖥️ Desktop Applications](desktop/README.md) - Windows, macOS, Linux
- [🔧 Native Integration](native/README.md) - C/C++, Rust
- [🌐 Web Applications](web/README.md) - WebAssembly, Electron
- [🖼️ Platform Guides](platforms/README.md) - Platform-specific details

## 🎯 Choose Your Platform

<table>
<tr>
<td width="50%">

### 📱 **Mobile Development**
- [📱 iOS (Swift)](mobile/ios.md)
- [🤖 Android (Kotlin/Java)](mobile/android.md)
- [🎯 Flutter (Dart)](mobile/flutter.md)
- [⚛️ React Native](mobile/react-native.md)
- [📱 Xamarin (.NET)](mobile/xamarin.md)

### 🖥️ **Desktop Development**
- [🐍 Python](desktop/python.md)
- [⚡ C# / .NET](desktop/dotnet.md)
- [☕ Java](desktop/java.md)
- [🟦 TypeScript/Node.js](desktop/nodejs.md)
- [🔷 Go](desktop/go.md)

</td>
<td width="50%">

### 🔧 **Native Development**
- [⚙️ C/C++](native/cpp.md)
- [🦀 Rust](native/rust.md)
- [📋 C API Reference](../04-api/c-ffi.md)

### 🌐 **Web Development**
- [🕸️ WebAssembly](web/wasm.md)
- [⚡ Electron](web/electron.md)
- [📦 Node.js](web/nodejs.md)

### 🖼️ **Platform-Specific**
- [🐧 Linux](platforms/linux.md)
- [🪟 Windows](platforms/windows.md)
- [🍎 macOS](platforms/macos.md)

</td>
</tr>
</table>

## 🚀 Quick Start by Language

### **C/C++** (Native)
```c
#include "rvpnse.h"

int main() {
    rvpnse_init();
    
    RvpnseConfig* config = rvpnse_config_from_file("config.toml");
    RvpnseClient* client = rvpnse_client_new(config);
    
    if (rvpnse_client_connect(client) == RVPNSE_SUCCESS) {
        printf("Connected!\\n");
    }
    
    rvpnse_client_free(client);
    rvpnse_config_free(config);
    rvpnse_cleanup();
    return 0;
}
```
👉 [Full C/C++ Guide](native/cpp.md)

### **Python** (ctypes)
```python
import ctypes
from rvpnse import RvpnseClient, RvpnseConfig

# Load configuration
config = RvpnseConfig.from_file("config.toml")

# Create client and connect
client = RvpnseClient(config)
client.connect()

print("Connected to VPN!")
```
👉 [Full Python Guide](desktop/python.md)

### **C#/.NET** (P/Invoke)
```csharp
using RvpnseSharp;

var config = RvpnseConfig.FromFile("config.toml");
var client = new RvpnseClient(config);

await client.ConnectAsync();
Console.WriteLine("Connected to VPN!");
```
👉 [Full .NET Guide](desktop/dotnet.md)

### **Swift** (iOS)
```swift
import RvpnseFramework

class VPNManager {
    private var client: OpaquePointer?
    
    func connect() async throws {
        let config = rvpnse_config_from_file("config.toml")
        client = rvpnse_client_new(config)
        
        let result = rvpnse_client_connect(client)
        if result != RVPNSE_SUCCESS {
            throw VPNError.connectionFailed
        }
    }
}
```
👉 [Full iOS Guide](mobile/ios.md)

### **Kotlin** (Android)
```kotlin
class VpnManager {
    private external fun connect(config: String): Boolean
    private external fun disconnect(): Boolean
    
    companion object {
        init { System.loadLibrary("rvpnse") }
    }
    
    fun connectToVpn(configPath: String) {
        if (connect(configPath)) {
            println("Connected to VPN!")
        }
    }
}
```
👉 [Full Android Guide](mobile/android.md)

### **Rust** (Native)
```rust
use rvpnse::{Client, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_file("config.toml")?;
    let client = Client::new(config)?;
    
    client.connect().await?;
    println!("Connected to VPN!");
    
    Ok(())
}
```
👉 [Full Rust Guide](native/rust.md)

## 📱 Mobile Integration Patterns

### **iOS NetworkExtension**
```swift
// Network Extension Provider
class PacketTunnelProvider: NEPacketTunnelProvider {
    private var client: OpaquePointer?
    
    override func startTunnel(options: [String : NSObject]?) async throws {
        // Initialize RVPNSE
        let config = loadConfiguration()
        client = rvpnse_client_new(config)
        
        // Connect
        try await connect()
        
        // Configure tunnel
        let settings = createTunnelSettings()
        try await setTunnelNetworkSettings(settings)
    }
}
```

### **Android VpnService**
```kotlin
class RvpnseVpnService : VpnService() {
    private var vpnInterface: ParcelFileDescriptor? = null
    
    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        // Create VPN interface
        val builder = Builder()
            .setSession("RVPNSE")
            .addAddress("10.0.0.2", 24)
            .addRoute("0.0.0.0", 0)
            .addDnsServer("8.8.8.8")
            
        vpnInterface = builder.establish()
        
        // Connect RVPNSE
        connectVpn()
        
        return START_STICKY
    }
}
```

## 🖥️ Desktop Integration Patterns

### **System Service (Linux)**
```bash
# systemd service file
[Unit]
Description=RVPNSE VPN Service
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/rvpnse-daemon --config /etc/rvpnse/config.toml
Restart=always

[Install]
WantedBy=multi-user.target
```

### **Windows Service**
```csharp
public class RvpnseWindowsService : ServiceBase
{
    private RvpnseClient client;
    
    protected override void OnStart(string[] args)
    {
        var config = RvpnseConfig.FromFile(@"C:\\Program Files\\RVPNSE\\config.toml");
        client = new RvpnseClient(config);
        
        Task.Run(async () => await client.ConnectAsync());
    }
    
    protected override void OnStop()
    {
        client?.DisconnectAsync().Wait();
        client?.Dispose();
    }
}
```

## 🌐 Web Integration Patterns

### **Electron Main Process**
```typescript
import { app, BrowserWindow, ipcMain } from 'electron';
import { RvpnseClient } from 'rvpnse-electron';

let vpnClient: RvpnseClient;

app.whenReady().then(() => {
    vpnClient = new RvpnseClient();
    
    ipcMain.handle('vpn-connect', async (event, config) => {
        return await vpnClient.connect(config);
    });
    
    ipcMain.handle('vpn-disconnect', async () => {
        return await vpnClient.disconnect();
    });
});
```

### **WebAssembly (Experimental)**
```javascript
import { RvpnseWasm } from 'rvpnse-wasm';

async function connectVPN() {
    const rvpnse = await RvpnseWasm.init();
    const config = await fetch('/config.toml').then(r => r.text());
    
    await rvpnse.connect(config);
    console.log('Connected via WebAssembly!');
}
```

## 🛡️ Security Considerations

### **Credential Management**
```toml
# Use environment variables for sensitive data
[credentials]
username = "${VPN_USERNAME}"
password = "${VPN_PASSWORD}"

# Or use external credential providers
[credentials]
provider = "keychain"  # macOS Keychain
# provider = "credential_manager"  # Windows Credential Manager
# provider = "keyring"  # Linux keyring
```

### **Certificate Validation**
```c
// Custom certificate validation
RvpnseResult validate_certificate(const char* cert_pem, void* userdata) {
    // Implement custom validation logic
    // Check certificate against known good certificates
    // Verify certificate chain
    // Check certificate expiration
    
    return RVPNSE_SUCCESS;
}

rvpnse_client_set_cert_validator(client, validate_certificate, NULL);
```

## 📊 Monitoring and Debugging

### **Connection Monitoring**
```c
void on_state_changed(RvpnseConnectionState state, void* userdata) {
    switch (state) {
        case RVPNSE_STATE_CONNECTING:
            printf("Connecting to VPN...\\n");
            break;
        case RVPNSE_STATE_CONNECTED:
            printf("VPN connected successfully\\n");
            break;
        case RVPNSE_STATE_DISCONNECTED:
            printf("VPN disconnected\\n");
            break;
        case RVPNSE_STATE_ERROR:
            printf("VPN connection error\\n");
            break;
    }
}

RvpnseCallbacks callbacks = {
    .on_state_changed = on_state_changed,
    .on_error = on_error,
    .on_stats_updated = on_stats_updated
};
rvpnse_client_set_callbacks(client, &callbacks, NULL);
```

### **Performance Monitoring**
```python
import time
from rvpnse import RvpnseClient

client = RvpnseClient(config)
client.connect()

while True:
    stats = client.get_statistics()
    print(f"Throughput: {stats.bytes_per_second / 1024 / 1024:.2f} MB/s")
    print(f"Latency: {stats.latency_ms}ms")
    print(f"Packet Loss: {stats.packet_loss_percent:.1f}%")
    
    time.sleep(1)
```

## 🎯 Platform-Specific Guides

| Platform | Key Features | Documentation |
|----------|--------------|---------------|
| **Linux** | TUN/TAP, systemd, NetworkManager | [Linux Guide](platforms/linux.md) |
| **Windows** | WinTUN, Windows Service, UWP | [Windows Guide](platforms/windows.md) |
| **macOS** | utun, launchd, NetworkExtension | [macOS Guide](platforms/macos.md) |
| **Android** | VpnService, NDK, Permissions | [Android Guide](mobile/android.md) |
| **iOS** | NetworkExtension, App Store | [iOS Guide](mobile/ios.md) |

## 🧪 Testing Your Integration

### **Unit Tests**
```c
// Test basic connection
void test_basic_connection() {
    rvpnse_init();
    
    RvpnseConfig* config = create_test_config();
    RvpnseClient* client = rvpnse_client_new(config);
    
    RvpnseResult result = rvpnse_client_connect(client);
    assert(result == RVPNSE_SUCCESS);
    
    rvpnse_client_disconnect(client);
    rvpnse_client_free(client);
    rvpnse_config_free(config);
    rvpnse_cleanup();
}
```

### **Integration Tests**
```python
import pytest
from rvpnse import RvpnseClient, RvpnseConfig

@pytest.fixture
def vpn_client():
    config = RvpnseConfig.from_file("test_config.toml")
    client = RvpnseClient(config)
    yield client
    client.disconnect()

def test_connection_lifecycle(vpn_client):
    # Test connection
    vpn_client.connect()
    assert vpn_client.state == "connected"
    
    # Test disconnection
    vpn_client.disconnect()
    assert vpn_client.state == "disconnected"
```

## 🆘 Troubleshooting

### **Common Integration Issues**

| Issue | Solution |
|-------|----------|
| **Library not found** | Check LD_LIBRARY_PATH / PATH / DYLD_LIBRARY_PATH |
| **Permission denied** | Ensure VPN permissions on mobile platforms |
| **Connection timeout** | Check firewall and network configuration |
| **Certificate errors** | Verify server certificate and CA chain |
| **Memory leaks** | Ensure proper cleanup of RVPNSE objects |

### **Debug Logging**
```toml
[logging]
level = "debug"
output = "console"  # or "file", "syslog"
format = "json"     # or "text"

[logging.file]
path = "/var/log/rvpnse.log"
max_size = "100MB"
max_files = 5
```

## 🎯 Next Steps

1. **Choose your platform** from the guides above
2. **Follow the specific integration guide** for your technology stack  
3. **Test your integration** using our testing guidelines
4. **Deploy securely** following our security best practices
5. **Monitor performance** using our observability features

**Need help?** Check our [Troubleshooting Guide](../07-troubleshooting/README.md) or [ask questions](https://github.com/devstroop/rvpnse/discussions).
