# iOS Integration Guide

Integrate Rust VPNSE into iOS applications using Swift and the NetworkExtension framework.

## 📋 Prerequisites

- **Xcode 14+** with iOS 14+ deployment target
- **Apple Developer Account** with NetworkExtension entitlements
- **Rust VPNSE static library** built for iOS targets

## 🛠️ Step 1: Build for iOS

```bash
# Add iOS targets
rustup target add aarch64-apple-ios      # iPhone/iPad (ARM64)
rustup target add x86_64-apple-ios       # Simulator (Intel)
rustup target add aarch64-apple-ios-sim  # Simulator (Apple Silicon)

# Build for device
cargo build --release --target aarch64-apple-ios

# Build for simulator
cargo build --release --target aarch64-apple-ios-sim
```

## 📦 Step 2: Create Universal Library

Create a script to build universal library:

```bash
#!/bin/bash
# build_ios.sh

set -e

# Build for all iOS targets
cargo build --release --target aarch64-apple-ios
cargo build --release --target aarch64-apple-ios-sim
cargo build --release --target x86_64-apple-ios

# Create universal library for simulator
lipo -create \
    target/aarch64-apple-ios-sim/release/librvpnse.a \
    target/x86_64-apple-ios/release/librvpnse.a \
    -output target/ios-sim-universal/librvpnse.a

# Device library (single architecture)
cp target/aarch64-apple-ios/release/librvpnse.a target/ios-device/

echo "✅ iOS libraries built successfully"
echo "📱 Device: target/ios-device/librvpnse.a"
echo "🔄 Simulator: target/ios-sim-universal/librvpnse.a"
```

## 🎯 Step 3: Xcode Project Setup

### **Add Libraries to Xcode**

1. **Drag libraries** into your Xcode project:
   - Add `librvpnse.a` to "Link Binary With Libraries"
   - Add `rvpnse.h` to your project

2. **Configure Build Settings**:
   ```
   Library Search Paths: $(PROJECT_DIR)/libs
   Header Search Paths: $(PROJECT_DIR)/include
   Other Linker Flags: -lrvpnse
   ```

3. **Add Capabilities**:
   - Enable "Network Extensions" capability
   - Add "Personal VPN" entitlement

### **Create Bridging Header**

Create `YourApp-Bridging-Header.h`:

```c
#ifndef YourApp_Bridging_Header_h
#define YourApp_Bridging_Header_h

#include "rvpnse.h"

#endif
```

## 📱 Step 4: Swift VPN Manager

Create a VPN manager class:

```swift
import Foundation
import NetworkExtension

class VPNSEManager: ObservableObject {
    private var clientHandle: UnsafeMutablePointer<vpnse_client_t>?
    private var vpnManager: NETunnelProviderManager?
    
    @Published var connectionStatus: NEVPNStatus = .invalid
    @Published var isConnecting = false
    
    init() {
        setupVPNManager()
    }
    
    private func setupVPNManager() {
        NETunnelProviderManager.loadAllFromPreferences { [weak self] managers, error in
            if let error = error {
                print("❌ Failed to load VPN preferences: \(error)")
                return
            }
            
            if let manager = managers?.first {
                self?.vpnManager = manager
            } else {
                self?.createVPNConfiguration()
            }
            
            self?.observeVPNStatus()
        }
    }
    
    private func createVPNConfiguration() {
        let manager = NETunnelProviderManager()
        let protocolConfiguration = NETunnelProviderProtocol()
        
        // Configure the VPN
        protocolConfiguration.providerBundleIdentifier = "com.yourcompany.vpnextension"
        protocolConfiguration.serverAddress = "VPN Server"
        
        manager.protocolConfiguration = protocolConfiguration
        manager.localizedDescription = "Rust VPNSE VPN"
        manager.isEnabled = true
        
        manager.saveToPreferences { [weak self] error in
            if let error = error {
                print("❌ Failed to save VPN configuration: \(error)")
                return
            }
            
            self?.vpnManager = manager
            print("✅ VPN configuration saved")
        }
    }
    
    func connect(config: VPNSEConfig) {
        guard let manager = vpnManager else {
            print("❌ VPN manager not ready")
            return
        }
        
        isConnecting = true
        
        // 1. Create Rust VPNSE client
        let configString = config.toTOML()
        configString.withCString { configPtr in
            clientHandle = vpnse_client_new(configPtr)
        }
        
        guard let client = clientHandle else {
            print("❌ Failed to create Rust VPNSE client")
            isConnecting = false
            return
        }
        
        // 2. Start NetworkExtension VPN
        do {
            try manager.connection.startVPNTunnel(options: [
                "config": configString,
                "server": config.server.hostname,
                "username": config.auth.username ?? "",
                "password": config.auth.password ?? ""
            ])
        } catch {
            print("❌ Failed to start VPN tunnel: \(error)")
            cleanupClient()
            isConnecting = false
        }
    }
    
    func disconnect() {
        vpnManager?.connection.stopVPNTunnel()
        cleanupClient()
    }
    
    private func cleanupClient() {
        if let client = clientHandle {
            vpnse_client_free(client)
            clientHandle = nil
        }
        isConnecting = false
    }
    
    private func observeVPNStatus() {
        NotificationCenter.default.addObserver(
            forName: .NEVPNStatusDidChange,
            object: nil,
            queue: .main
        ) { [weak self] notification in
            guard let manager = self?.vpnManager else { return }
            self?.connectionStatus = manager.connection.status
            
            switch manager.connection.status {
            case .connected:
                self?.isConnecting = false
                print("✅ VPN Connected")
            case .disconnected:
                self?.isConnecting = false
                self?.cleanupClient()
                print("📴 VPN Disconnected")
            case .connecting:
                print("🔄 VPN Connecting...")
            default:
                break
            }
        }
    }
}
```

## 🔧 Step 5: Configuration Model

Create a configuration model:

```swift
struct VPNSEConfig: Codable {
    let server: ServerConfig
    let auth: AuthConfig
    let vpn: VPNConfig
    
    struct ServerConfig: Codable {
        let hostname: String
        let port: Int
        let hub: String
        let useSSL: Bool
        let timeout: Int
        
        enum CodingKeys: String, CodingKey {
            case hostname, port, hub
            case useSSL = "use_ssl"
            case timeout
        }
    }
    
    struct AuthConfig: Codable {
        let method: String
        let username: String?
        let password: String?
    }
    
    struct VPNConfig: Codable {
        let autoReconnect: Bool
        let keepaliveInterval: Int
        let mtu: Int
        
        enum CodingKeys: String, CodingKey {
            case autoReconnect = "auto_reconnect"
            case keepaliveInterval = "keepalive_interval"
            case mtu
        }
    }
    
    func toTOML() -> String {
        return """
        [server]
        hostname = "\(server.hostname)"
        port = \(server.port)
        hub = "\(server.hub)"
        use_ssl = \(server.useSSL)
        timeout = \(server.timeout)
        
        [auth]
        method = "\(auth.method)"
        username = "\(auth.username ?? "")"
        password = "\(auth.password ?? "")"
        
        [vpn]
        auto_reconnect = \(vpn.autoReconnect)
        keepalive_interval = \(vpn.keepaliveInterval)
        mtu = \(vpn.mtu)
        """
    }
}
```

## 🔌 Step 6: Network Extension Target

Create a new **Network Extension** target in Xcode:

1. **File → New → Target**
2. Choose **Network Extension**
3. Select **Packet Tunnel Provider**

### **Extension Provider Implementation**

```swift
import NetworkExtension

class PacketTunnelProvider: NEPacketTunnelProvider {
    private var clientHandle: UnsafeMutablePointer<vpnse_client_t>?
    private var packetFlow: NEPacketTunnelFlow?
    
    override func startTunnel(options: [String: NSObject]?) async throws {
        // Extract configuration from options
        guard let configString = options?["config"] as? String,
              let server = options?["server"] as? String,
              let username = options?["username"] as? String,
              let password = options?["password"] as? String else {
            throw NEVPNError(.configurationInvalid)
        }
        
        // 1. Create Rust VPNSE client
        clientHandle = configString.withCString { configPtr in
            return vpnse_client_new(configPtr)
        }
        
        guard let client = clientHandle else {
            throw NEVPNError(.configurationInvalid)
        }
        
        // 2. Connect to SoftEther server
        let result = server.withCString { serverPtr in
            return vpnse_client_connect(client, serverPtr, 443)
        }
        
        guard result == VPNSE_SUCCESS.rawValue else {
            throw NEVPNError(.serverAddressResolutionFailed)
        }
        
        // 3. Authenticate
        let authResult = username.withCString { userPtr in
            password.withCString { passPtr in
                return vpnse_client_authenticate(client, userPtr, passPtr)
            }
        }
        
        guard authResult == VPNSE_SUCCESS.rawValue else {
            throw NEVPNError(.authenticationFailed)
        }
        
        // 4. Configure network settings
        let networkSettings = NEPacketTunnelNetworkSettings(tunnelRemoteAddress: server)
        
        // Configure IPv4
        let ipv4Settings = NEIPv4Settings(addresses: ["10.0.0.2"], subnetMasks: ["255.255.255.0"])
        ipv4Settings.includedRoutes = [NEIPv4Route.default()]
        networkSettings.ipv4Settings = ipv4Settings
        
        // Configure DNS
        let dnsSettings = NEDNSSettings(servers: ["8.8.8.8", "8.8.4.4"])
        networkSettings.dnsSettings = dnsSettings
        
        try await setTunnelNetworkSettings(networkSettings)
        
        // 5. Start packet forwarding
        packetFlow = packetFlow
        startPacketForwarding()
        
        print("✅ VPN Tunnel started successfully")
    }
    
    override func stopTunnel(with reason: NEProviderStopReason) async {
        if let client = clientHandle {
            vpnse_client_disconnect(client)
            vpnse_client_free(client)
            clientHandle = nil
        }
        
        print("📴 VPN Tunnel stopped")
    }
    
    private func startPacketForwarding() {
        guard let client = clientHandle else { return }
        
        // Read packets from tunnel interface
        packetFlow?.readPackets { [weak self] packets, protocols in
            // Forward packets to SoftEther server via Rust VPNSE
            for (index, packet) in packets.enumerated() {
                packet.withUnsafeBytes { bytes in
                    vpnse_client_send_packet(client, bytes.bindMemory(to: UInt8.self).baseAddress, Int32(packet.count))
                }
            }
            
            // Continue reading
            self?.startPacketForwarding()
        }
        
        // Receive packets from SoftEther server
        // Note: This is a simplified example
        // In practice, you need to handle this asynchronously
        DispatchQueue.global(qos: .default).async { [weak self] in
            self?.receivePacketsFromServer(client: client)
        }
    }
    
    private func receivePacketsFromServer(client: UnsafeMutablePointer<vpnse_client_t>) {
        var buffer = [UInt8](repeating: 0, count: 2048)
        var packetSize: Int32 = 0
        
        while true {
            let result = vpnse_client_receive_packet(client, &buffer, Int32(buffer.count), &packetSize)
            
            if result == VPNSE_SUCCESS.rawValue && packetSize > 0 {
                let packetData = Data(buffer.prefix(Int(packetSize)))
                
                // Write packet to tunnel interface
                packetFlow?.writePackets([packetData], withProtocols: [NSNumber(value: AF_INET)])
            }
            
            // Small delay to prevent busy loop
            usleep(1000) // 1ms
        }
    }
}
```

## 📱 Step 7: UI Integration

Create a SwiftUI view for VPN control:

```swift
import SwiftUI
import NetworkExtension

struct VPNControlView: View {
    @StateObject private var vpnManager = VPNSEManager()
    @State private var config = VPNSEConfig(
        server: .init(hostname: "vpn.example.com", port: 443, hub: "VPN", useSSL: true, timeout: 30),
        auth: .init(method: "password", username: "user", password: "pass"),
        vpn: .init(autoReconnect: true, keepaliveInterval: 60, mtu: 1500)
    )
    
    var body: some View {
        VStack(spacing: 20) {
            VStack {
                Image(systemName: vpnStatusIcon)
                    .font(.system(size: 60))
                    .foregroundColor(vpnStatusColor)
                
                Text(vpnStatusText)
                    .font(.headline)
                    .foregroundColor(vpnStatusColor)
            }
            
            if vpnManager.isConnecting {
                ProgressView("Connecting...")
                    .progressViewStyle(CircularProgressViewStyle())
            } else {
                Button(action: toggleVPN) {
                    Text(vpnManager.connectionStatus == .connected ? "Disconnect" : "Connect")
                        .font(.headline)
                        .foregroundColor(.white)
                        .frame(maxWidth: .infinity)
                        .padding()
                        .background(vpnManager.connectionStatus == .connected ? Color.red : Color.blue)
                        .cornerRadius(10)
                }
            }
            
            VStack(alignment: .leading, spacing: 10) {
                Text("Server: \(config.server.hostname)")
                Text("Hub: \(config.server.hub)")
                Text("Status: \(vpnStatusText)")
            }
            .padding()
            .background(Color.gray.opacity(0.1))
            .cornerRadius(10)
            
            Spacer()
        }
        .padding()
    }
    
    private func toggleVPN() {
        switch vpnManager.connectionStatus {
        case .connected, .connecting:
            vpnManager.disconnect()
        default:
            vpnManager.connect(config: config)
        }
    }
    
    private var vpnStatusIcon: String {
        switch vpnManager.connectionStatus {
        case .connected:
            return "shield.fill"
        case .connecting:
            return "shield"
        case .disconnected:
            return "shield.slash"
        default:
            return "shield.slash"
        }
    }
    
    private var vpnStatusColor: Color {
        switch vpnManager.connectionStatus {
        case .connected:
            return .green
        case .connecting:
            return .orange
        case .disconnected:
            return .red
        default:
            return .gray
        }
    }
    
    private var vpnStatusText: String {
        switch vpnManager.connectionStatus {
        case .connected:
            return "Connected"
        case .connecting:
            return "Connecting"
        case .disconnected:
            return "Disconnected"
        case .disconnecting:
            return "Disconnecting"
        default:
            return "Not Configured"
        }
    }
}
```

## 🔒 Step 8: Entitlements and Permissions

### **Main App Entitlements**

Add to your main app's entitlements file:

```xml
<key>com.apple.developer.networking.networkextension</key>
<array>
    <string>packet-tunnel-provider</string>
</array>
```

### **Extension Entitlements**

Add to your Network Extension's entitlements file:

```xml
<key>com.apple.developer.networking.networkextension</key>
<array>
    <string>packet-tunnel-provider</string>
</array>
<key>com.apple.security.application-groups</key>
<array>
    <string>group.com.yourcompany.vpnapp</string>
</array>
```

## 🧪 Step 9: Testing

### **Test Configuration Parsing**

```swift
func testConfigurationParsing() {
    let config = """
    [server]
    hostname = "test.example.com"
    port = 443
    hub = "VPN"
    
    [auth]
    method = "password"
    username = "test"
    password = "test"
    """
    
    config.withCString { configPtr in
        var errorBuffer = [CChar](repeating: 0, count: 256)
        let result = vpnse_parse_config(configPtr, &errorBuffer, 256)
        
        if result == VPNSE_SUCCESS.rawValue {
            print("✅ Configuration parsing successful")
        } else {
            let errorString = String(cString: errorBuffer)
            print("❌ Configuration error: \(errorString)")
        }
    }
}
```

### **Test Client Creation**

```swift
func testClientCreation() {
    let config = VPNSEConfig(
        server: .init(hostname: "test.com", port: 443, hub: "VPN", useSSL: true, timeout: 30),
        auth: .init(method: "password", username: "test", password: "test"),
        vpn: .init(autoReconnect: true, keepaliveInterval: 60, mtu: 1500)
    )
    
    let configString = config.toTOML()
    let client = configString.withCString { configPtr in
        return vpnse_client_new(configPtr)
    }
    
    if client != nil {
        print("✅ Client creation successful")
        vpnse_client_free(client)
    } else {
        print("❌ Client creation failed")
    }
}
```

## 🚨 Common Issues

### **Library Linking Issues**
```
Undefined symbols for architecture arm64: "_vpnse_client_new"
```
**Solution**: Ensure the static library is properly linked and built for the correct architecture.

### **Bridging Header Issues**
```
'rvpnse.h' file not found
```
**Solution**: Add the header file to your project and configure the bridging header path.

### **NetworkExtension Permission Issues**
```
The operation couldn't be completed. (NEVPNErrorDomain error 1.)
```
**Solution**: Ensure NetworkExtension entitlements are properly configured and app is signed.

### **VPN Configuration Issues**
```
Cannot save VPN configuration
```
**Solution**: Check that the app has proper entitlements and the user has granted VPN permissions.

## 📚 Related Documentation

- [Quick Start Guide](../quick-start.md) - Build Rust VPNSE for iOS
- [Configuration Reference](../configuration.md) - TOML configuration options
- [C API Reference](../api/c-api.md) - Complete API documentation
- [iOS Platform Guide](../platforms/ios.md) - iOS-specific networking details

---

**🎉 Your iOS app is now ready to use Rust VPNSE for SoftEther VPN connectivity!**
