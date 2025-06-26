# 🖥️ Desktop Integration Guide

Complete integration guides for desktop applications across Windows, macOS, and Linux. rVPNSE provides native libraries and bindings for all major desktop development frameworks.

## 📋 Platform Support

| Platform | Minimum Version | Technologies | Status |
|----------|----------------|--------------|--------|
| **Windows** | Windows 10 | .NET, C++, Python, Node.js | ✅ Production Ready |
| **macOS** | macOS 10.14 | Swift, Objective-C, Python, Node.js | ✅ Production Ready |
| **Linux** | Ubuntu 18.04+ | C++, Python, Node.js, Go | ✅ Production Ready |
| **Cross-Platform** | - | Electron, Tauri, Qt | ✅ Production Ready |

## 🚀 Quick Start

### Windows (.NET)
```bash
# Install NuGet package
dotnet add package Rvpnse.NET
```

### macOS (Swift)
```bash
# Download framework
curl -L -o RvpnseFramework.zip \
  https://github.com/devstroop/rvpnse/releases/latest/download/rvpnse-macos-framework.zip
```

### Linux (Python)
```bash
# Install Python package
pip install rvpnse-python
```

### Cross-Platform (Electron)
```bash
# Install npm package
npm install rvpnse-electron
```

## 📚 Framework Guides

- [🐍 **Python Integration**](python.md) - Cross-platform Python applications
- [⚡ **C# / .NET Integration**](dotnet.md) - Windows and cross-platform .NET
- [☕ **Java Integration**](java.md) - Cross-platform Java applications
- [🟦 **TypeScript/Node.js**](nodejs.md) - Server-side and desktop apps
- [🔷 **Go Integration**](go.md) - High-performance Go applications
- [🔧 **C++ Integration**](../native/cpp.md) - Native C++ applications
- [🌐 **Electron Integration**](../web/electron.md) - Cross-platform web-based apps

## 🎯 Key Desktop Features

### **System Integration**
- System tray integration and notifications
- Auto-start on system boot
- System VPN settings integration
- Native look and feel

### **Performance**
- Multi-threaded connection handling
- Efficient memory management
- Background processing optimization
- System resource monitoring

### **Security**
- OS credential storage integration
- Certificate store integration
- Privilege escalation handling
- Secure inter-process communication

### **User Experience**
- Native UI components
- Keyboard shortcuts and accessibility
- Multi-monitor support
- System theme adaptation

## 🛡️ Security Considerations

### **Credential Management**

#### Windows (Credential Manager)
```csharp
using System.Security.Cryptography;
using Microsoft.Win32;

public class WindowsCredentialManager {
    public static void StoreCredentials(string username, string password) {
        // Use Windows Credential Manager
        var credential = new Credential {
            Target = "RvpnseVPN",
            Username = username,
            Password = password,
            Type = CredentialType.Generic,
            Persist = CredentialPersist.LocalMachine
        };
        credential.Save();
    }
}
```

#### macOS (Keychain)
```swift
import Security

class KeychainManager {
    static func storeCredentials(username: String, password: String) -> Bool {
        let query: [String: Any] = [
            kSecClass as String: kSecClassInternetPassword,
            kSecAttrAccount as String: username,
            kSecAttrServer as String: "rvpnse.local",
            kSecValueData as String: password.data(using: .utf8)!
        ]
        
        SecItemDelete(query as CFDictionary)
        return SecItemAdd(query as CFDictionary, nil) == errSecSuccess
    }
}
```

#### Linux (Secret Service)
```python
import secretstorage

class LinuxCredentialManager:
    def __init__(self):
        self.connection = secretstorage.dbus_init()
        self.collection = secretstorage.get_default_collection(self.connection)
    
    def store_credentials(self, username: str, password: str):
        attributes = {
            'application': 'rvpnse',
            'username': username
        }
        self.collection.create_item(
            'RvpnseVPN Credentials',
            attributes,
            password,
            replace=True
        )
```

## 🎨 UI Frameworks Integration

### **System Tray Application**

#### Windows (WPF)
```csharp
using System.Windows.Forms;
using System.Drawing;

public partial class VpnTrayApp : Form {
    private NotifyIcon trayIcon;
    private RvpnseClient vpnClient;
    
    public VpnTrayApp() {
        InitializeComponent();
        SetupTrayIcon();
        InitializeVpn();
    }
    
    private void SetupTrayIcon() {
        trayIcon = new NotifyIcon {
            Icon = SystemIcons.Shield,
            Text = "rVPNSE VPN",
            Visible = true
        };
        
        var contextMenu = new ContextMenuStrip();
        contextMenu.Items.Add("Connect", null, OnConnect);
        contextMenu.Items.Add("Disconnect", null, OnDisconnect);
        contextMenu.Items.Add("Settings", null, OnSettings);
        contextMenu.Items.Add("Exit", null, OnExit);
        
        trayIcon.ContextMenuStrip = contextMenu;
    }
    
    private async void OnConnect(object sender, EventArgs e) {
        try {
            await vpnClient.ConnectAsync();
            trayIcon.Icon = SystemIcons.Shield; // Green shield
            trayIcon.ShowBalloonTip(3000, "rVPNSE", "Connected to VPN", ToolTipIcon.Info);
        } catch (Exception ex) {
            MessageBox.Show($"Connection failed: {ex.Message}", "Error");
        }
    }
}
```

#### macOS (SwiftUI)
```swift
import SwiftUI
import ServiceManagement

@main
struct VpnTrayApp: App {
    @StateObject private var vpnManager = VpnManager()
    
    var body: some Scene {
        MenuBarExtra("rVPNSE", systemImage: vpnManager.isConnected ? "shield.fill" : "shield") {
            VpnMenuView(vpnManager: vpnManager)
        }
    }
}

struct VpnMenuView: View {
    @ObservedObject var vpnManager: VpnManager
    
    var body: some View {
        VStack {
            if vpnManager.isConnected {
                Text("Connected")
                    .foregroundColor(.green)
                Button("Disconnect") {
                    vpnManager.disconnect()
                }
            } else {
                Text("Disconnected")
                    .foregroundColor(.red)
                Button("Connect") {
                    vpnManager.connect()
                }
            }
            
            Divider()
            
            Button("Settings") {
                NSApp.activate(ignoringOtherApps: true)
                // Show settings window
            }
            
            Button("Quit") {
                NSApplication.shared.terminate(nil)
            }
        }
        .padding()
    }
}
```

#### Linux (GTK)
```python
import gi
gi.require_version('Gtk', '3.0')
gi.require_version('AppIndicator3', '0.1')
from gi.repository import Gtk, AppIndicator3, GObject
import threading

class VpnTrayApp:
    def __init__(self):
        self.indicator = AppIndicator3.Indicator.new(
            "rvpnse-vpn",
            "network-vpn",
            AppIndicator3.IndicatorCategory.SYSTEM_SERVICES
        )
        self.indicator.set_status(AppIndicator3.IndicatorStatus.ACTIVE)
        self.indicator.set_menu(self.create_menu())
        
        self.vpn_client = RvpnseClient()
    
    def create_menu(self):
        menu = Gtk.Menu()
        
        connect_item = Gtk.MenuItem("Connect")
        connect_item.connect("activate", self.on_connect)
        menu.append(connect_item)
        
        disconnect_item = Gtk.MenuItem("Disconnect")
        disconnect_item.connect("activate", self.on_disconnect)
        menu.append(disconnect_item)
        
        menu.append(Gtk.SeparatorMenuItem())
        
        settings_item = Gtk.MenuItem("Settings")
        settings_item.connect("activate", self.on_settings)
        menu.append(settings_item)
        
        quit_item = Gtk.MenuItem("Quit")
        quit_item.connect("activate", self.on_quit)
        menu.append(quit_item)
        
        menu.show_all()
        return menu
    
    def on_connect(self, widget):
        threading.Thread(target=self.connect_vpn).start()
    
    def connect_vpn(self):
        try:
            self.vpn_client.connect()
            GObject.idle_add(self.update_icon, "network-vpn-symbolic")
        except Exception as e:
            print(f"Connection failed: {e}")
```

### **Full Desktop Application**

#### Electron (TypeScript)
```typescript
// main.ts
import { app, BrowserWindow, Menu, Tray, nativeImage } from 'electron';
import { RvpnseClient } from 'rvpnse-electron';

class VpnDesktopApp {
    private mainWindow: BrowserWindow | null = null;
    private tray: Tray | null = null;
    private vpnClient: RvpnseClient;
    
    constructor() {
        this.vpnClient = new RvpnseClient();
        this.setupApp();
    }
    
    private setupApp() {
        app.whenReady().then(() => {
            this.createWindow();
            this.createTray();
        });
        
        app.on('window-all-closed', () => {
            if (process.platform !== 'darwin') {
                app.quit();
            }
        });
    }
    
    private createWindow() {
        this.mainWindow = new BrowserWindow({
            width: 400,
            height: 600,
            webPreferences: {
                nodeIntegration: true,
                contextIsolation: false
            },
            show: false,
            titleBarStyle: 'hiddenInset'
        });
        
        this.mainWindow.loadFile('dist/index.html');
        
        // Setup IPC handlers
        this.setupIpcHandlers();
    }
    
    private createTray() {
        const icon = nativeImage.createFromPath('assets/tray-icon.png');
        this.tray = new Tray(icon);
        
        const contextMenu = Menu.buildFromTemplate([
            { 
                label: 'Show Window', 
                click: () => this.mainWindow?.show() 
            },
            { 
                label: 'Connect', 
                click: () => this.connectVpn() 
            },
            { 
                label: 'Disconnect', 
                click: () => this.disconnectVpn() 
            },
            { type: 'separator' },
            { 
                label: 'Quit', 
                click: () => app.quit() 
            }
        ]);
        
        this.tray.setContextMenu(contextMenu);
        this.tray.setToolTip('rVPNSE VPN Client');
    }
    
    private async connectVpn() {
        try {
            await this.vpnClient.connect();
            this.updateTrayIcon(true);
        } catch (error) {
            console.error('VPN connection failed:', error);
        }
    }
    
    private async disconnectVpn() {
        try {
            await this.vpnClient.disconnect();
            this.updateTrayIcon(false);
        } catch (error) {
            console.error('VPN disconnection failed:', error);
        }
    }
    
    private updateTrayIcon(connected: boolean) {
        const iconPath = connected ? 'assets/tray-icon-connected.png' : 'assets/tray-icon.png';
        const icon = nativeImage.createFromPath(iconPath);
        this.tray?.setImage(icon);
    }
}

new VpnDesktopApp();
```

## 🔧 System Service Integration

### **Windows Service**
```csharp
using System.ServiceProcess;
using Microsoft.Extensions.Hosting;
using Microsoft.Extensions.DependencyInjection;

public class Program {
    public static void Main(string[] args) {
        var host = CreateHostBuilder(args).Build();
        
        if (Environment.UserInteractive) {
            // Run as console app for debugging
            host.Run();
        } else {
            // Run as Windows Service
            host.RunAsService();
        }
    }
    
    private static IHostBuilder CreateHostBuilder(string[] args) =>
        Host.CreateDefaultBuilder(args)
            .UseWindowsService()
            .ConfigureServices((context, services) => {
                services.AddHostedService<VpnBackgroundService>();
            });
}

public class VpnBackgroundService : BackgroundService {
    private readonly RvpnseClient vpnClient;
    
    public VpnBackgroundService() {
        vpnClient = new RvpnseClient();
    }
    
    protected override async Task ExecuteAsync(CancellationToken stoppingToken) {
        while (!stoppingToken.IsCancellationRequested) {
            // Monitor VPN connection and reconnect if needed
            if (!vpnClient.IsConnected) {
                await vpnClient.ConnectAsync();
            }
            
            await Task.Delay(TimeSpan.FromSeconds(30), stoppingToken);
        }
    }
}
```

### **macOS LaunchAgent**
```xml
<!-- ~/Library/LaunchAgents/com.devstroop.rvpnse.plist -->
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.devstroop.rvpnse</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/rvpnse-daemon</string>
        <string>--config</string>
        <string>/usr/local/etc/rvpnse/config.toml</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/usr/local/var/log/rvpnse.log</string>
    <key>StandardErrorPath</key>
    <string>/usr/local/var/log/rvpnse.error.log</string>
</dict>
</plist>
```

### **Linux Systemd Service**
```ini
# /etc/systemd/system/rvpnse.service
[Unit]
Description=rVPNSE VPN Service
After=network.target
Wants=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/rvpnse-daemon --config /etc/rvpnse/config.toml
Restart=always
RestartSec=5
User=rvpnse
Group=rvpnse

# Security settings
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
PrivateTmp=true

[Install]
WantedBy=multi-user.target
```

## 📊 Performance Monitoring

### **System Resource Monitoring**
```python
import psutil
import time
from rvpnse import RvpnseClient

class VpnPerformanceMonitor:
    def __init__(self, client: RvpnseClient):
        self.client = client
        self.process = psutil.Process()
    
    def monitor_performance(self):
        while True:
            # CPU usage
            cpu_percent = self.process.cpu_percent()
            
            # Memory usage
            memory_info = self.process.memory_info()
            memory_mb = memory_info.rss / 1024 / 1024
            
            # Network stats
            stats = self.client.get_statistics()
            
            print(f"CPU: {cpu_percent:.1f}% | "
                  f"Memory: {memory_mb:.1f}MB | "
                  f"Throughput: {stats.bytes_per_second / 1024 / 1024:.2f} MB/s")
            
            time.sleep(1)
```

### **Connection Quality Monitoring**
```csharp
public class ConnectionQualityMonitor {
    private readonly RvpnseClient client;
    private readonly Timer monitorTimer;
    
    public ConnectionQualityMonitor(RvpnseClient client) {
        this.client = client;
        this.monitorTimer = new Timer(MonitorConnection, null, TimeSpan.Zero, TimeSpan.FromSeconds(5));
    }
    
    private void MonitorConnection(object state) {
        var stats = client.GetStatistics();
        
        // Calculate connection quality metrics
        var latency = stats.Latency;
        var packetLoss = stats.PacketLossPercentage;
        var throughput = stats.BytesPerSecond;
        
        // Log quality metrics
        Console.WriteLine($"Latency: {latency}ms, Loss: {packetLoss}%, Throughput: {throughput / 1024 / 1024:F2} MB/s");
        
        // Trigger reconnection if quality is poor
        if (latency > 500 || packetLoss > 5) {
            Console.WriteLine("Poor connection quality detected, reconnecting...");
            Task.Run(async () => {
                await client.DisconnectAsync();
                await Task.Delay(2000);
                await client.ConnectAsync();
            });
        }
    }
}
```

## 🧪 Testing Desktop Applications

### **Automated UI Testing**
```csharp
// Using Microsoft.VisualStudio.TestTools.UnitTesting for WPF
[TestClass]
public class VpnDesktopAppTests {
    private Application app;
    private MainWindow mainWindow;
    
    [TestInitialize]
    public void Setup() {
        app = new Application();
        mainWindow = new MainWindow();
        app.MainWindow = mainWindow;
    }
    
    [TestMethod]
    public async Task ConnectButton_ShouldConnectToVpn() {
        // Arrange
        var connectButton = mainWindow.FindName("ConnectButton") as Button;
        
        // Act
        connectButton.RaiseEvent(new RoutedEventArgs(Button.ClickEvent));
        await Task.Delay(2000); // Wait for connection
        
        // Assert
        var statusLabel = mainWindow.FindName("StatusLabel") as Label;
        Assert.AreEqual("Connected", statusLabel.Content);
    }
}
```

### **Cross-Platform Testing**
```python
# Using pytest for cross-platform testing
import pytest
import platform
from rvpnse import RvpnseClient

class TestCrossPlatform:
    def setup_method(self):
        self.client = RvpnseClient()
    
    def test_connection_on_current_platform(self):
        config = self.get_platform_config()
        result = self.client.connect(config)
        assert result.success
        
        # Test platform-specific features
        if platform.system() == "Windows":
            self.test_windows_specific_features()
        elif platform.system() == "Darwin":
            self.test_macos_specific_features()
        elif platform.system() == "Linux":
            self.test_linux_specific_features()
    
    def get_platform_config(self):
        # Return platform-specific test configuration
        pass
```

## 🚀 Deployment and Distribution

### **Windows Installer (WiX)**
```xml
<!-- installer.wxs -->
<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
  <Product Id="*" Language="1033" Version="1.0.0" Manufacturer="Your Company"
           Name="rVPNSE Desktop Client" UpgradeCode="YOUR-UPGRADE-CODE">
    
    <Package InstallerVersion="200" Compressed="yes" InstallScope="perMachine" />
    
    <Directory Id="TARGETDIR" Name="SourceDir">
      <Directory Id="ProgramFilesFolder">
        <Directory Id="INSTALLFOLDER" Name="rVPNSE" />
      </Directory>
    </Directory>
    
    <DirectoryRef Id="INSTALLFOLDER">
      <Component Id="MainExecutable" Guid="YOUR-GUID">
        <File Id="RvpnseDesktop.exe" Source="bin\Release\RvpnseDesktop.exe" KeyPath="yes">
          <Shortcut Id="StartMenuShortcut" Directory="ProgramMenuFolder" 
                    Name="rVPNSE VPN Client" Advertise="yes" />
        </File>
      </Component>
    </DirectoryRef>
    
    <Feature Id="Complete" Level="1">
      <ComponentRef Id="MainExecutable" />
    </Feature>
  </Product>
</Wix>
```

### **macOS App Bundle**
```bash
#!/bin/bash
# build-macos-app.sh

# Build the application
swift build --configuration release

# Create app bundle structure
mkdir -p "rVPNSE.app/Contents/MacOS"
mkdir -p "rVPNSE.app/Contents/Resources"

# Copy executable
cp ".build/release/rvpnse-desktop" "rVPNSE.app/Contents/MacOS/"

# Create Info.plist
cat > "rVPNSE.app/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>rvpnse-desktop</string>
    <key>CFBundleIdentifier</key>
    <string>com.devstroop.rvpnse</string>
    <key>CFBundleName</key>
    <string>rVPNSE</string>
    <key>CFBundleVersion</key>
    <string>1.0.0</string>
</dict>
</plist>
EOF

# Sign the app (if you have a developer certificate)
codesign --force --deep --sign "Developer ID Application: Your Name" "rVPNSE.app"
```

### **Linux Package (Debian)**
```bash
#!/bin/bash
# build-debian-package.sh

# Create package structure
mkdir -p rvpnse-desktop_1.0.0-1_amd64/DEBIAN
mkdir -p rvpnse-desktop_1.0.0-1_amd64/usr/bin
mkdir -p rvpnse-desktop_1.0.0-1_amd64/usr/share/applications
mkdir -p rvpnse-desktop_1.0.0-1_amd64/usr/share/icons/hicolor/256x256/apps

# Create control file
cat > rvpnse-desktop_1.0.0-1_amd64/DEBIAN/control << EOF
Package: rvpnse-desktop
Version: 1.0.0-1
Section: net
Priority: optional
Architecture: amd64
Depends: libc6, libssl1.1
Maintainer: Your Name <your.email@example.com>
Description: rVPNSE Desktop VPN Client
 A secure and fast VPN client based on rVPNSE library
EOF

# Copy files
cp target/release/rvpnse-desktop rvpnse-desktop_1.0.0-1_amd64/usr/bin/
cp resources/rvpnse.desktop rvpnse-desktop_1.0.0-1_amd64/usr/share/applications/
cp resources/icon.png rvpnse-desktop_1.0.0-1_amd64/usr/share/icons/hicolor/256x256/apps/rvpnse.png

# Build package
dpkg-deb --build rvpnse-desktop_1.0.0-1_amd64
```

## 🆘 Troubleshooting

| Issue | Platform | Solution |
|-------|----------|----------|
| **Permission denied** | Windows | Run as Administrator or request elevation |
| **Code signing required** | macOS | Sign with Apple Developer certificate |
| **Missing dependencies** | Linux | Install required system libraries |
| **Tray icon not showing** | All | Check system tray settings and permissions |
| **Auto-start not working** | All | Verify service/daemon installation |

## 📚 Additional Resources

- [Windows Desktop App Development](https://docs.microsoft.com/en-us/windows/apps/)
- [macOS App Development Guide](https://developer.apple.com/macos/)
- [Linux Desktop Integration](https://specifications.freedesktop.org/)
- [Electron Documentation](https://www.electronjs.org/docs)
- [Cross-Platform UI Frameworks](https://flutter.dev/desktop)

## 🎯 Next Steps

1. **Choose your framework** from the guides above
2. **Set up development environment** for your target platforms
3. **Implement basic VPN functionality** using our examples
4. **Add platform-specific features** (system tray, auto-start, etc.)
5. **Test on all target platforms**
6. **Package and distribute** your application

**Need help?** Check our [Desktop-specific troubleshooting](../../07-troubleshooting/desktop.md) or [ask questions](https://github.com/devstroop/rvpnse/discussions).
