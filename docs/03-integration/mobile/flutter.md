# 🎯 Flutter Integration Guide

Complete guide for integrating rVPNSE into Flutter applications. This guide covers both the Dart plugin implementation and platform-specific native code.

## 📋 Prerequisites

- Flutter 3.0 or later
- iOS 12.0+ / Android API 21+
- Xcode 14+ (for iOS development)
- Android Studio with NDK (for Android development)

## 🚀 Installation

### 1. Add Dependency

Add to your `pubspec.yaml`:

```yaml
dependencies:
  rvpnse_flutter: ^1.0.0
```

Then run:
```bash
flutter pub get
```

### 2. Platform Setup

#### iOS Setup
Add to `ios/Runner/Info.plist`:
```xml
<key>NSAppTransportSecurity</key>
<dict>
    <key>NSAllowsArbitraryLoads</key>
    <true/>
</dict>
<key>NEVPNManager</key>
<string>Used for VPN connection management</string>
```

#### Android Setup
Add to `android/app/src/main/AndroidManifest.xml`:
```xml
<uses-permission android:name="android.permission.INTERNET" />
<uses-permission android:name="android.permission.ACCESS_NETWORK_STATE" />
<uses-permission android:name="android.permission.BIND_VPN_SERVICE" />

<service android:name="com.devstroop.rvpnse_flutter.VpnService"
         android:permission="android.permission.BIND_VPN_SERVICE">
    <intent-filter>
        <action android:name="android.net.VpnService" />
    </intent-filter>
</service>
```

## 💻 Basic Usage

### 1. Initialize the Plugin

```dart
import 'package:rvpnse_flutter/rvpnse_flutter.dart';

class VpnManager {
  static final RvpnseFlutter _rvpnse = RvpnseFlutter();
  
  static Future<void> initialize() async {
    await _rvpnse.initialize();
  }
}
```

### 2. Configure VPN Settings

```dart
class VpnConfig {
  final String serverHost;
  final int serverPort;
  final String username;
  final String password;
  final String hubName;
  
  VpnConfig({
    required this.serverHost,
    required this.serverPort,
    required this.username,
    required this.password,
    required this.hubName,
  });
  
  Map<String, dynamic> toMap() {
    return {
      'server_host': serverHost,
      'server_port': serverPort,
      'username': username,
      'password': password,
      'hub_name': hubName,
    };
  }
}
```

### 3. Connect to VPN

```dart
class VpnService {
  static final RvpnseFlutter _rvpnse = RvpnseFlutter();
  
  static Future<bool> connect(VpnConfig config) async {
    try {
      final result = await _rvpnse.connect(config.toMap());
      return result['success'] ?? false;
    } catch (e) {
      print('VPN connection failed: $e');
      return false;
    }
  }
  
  static Future<bool> disconnect() async {
    try {
      final result = await _rvpnse.disconnect();
      return result['success'] ?? false;
    } catch (e) {
      print('VPN disconnection failed: $e');
      return false;
    }
  }
  
  static Future<VpnStatus> getStatus() async {
    try {
      final result = await _rvpnse.getStatus();
      return VpnStatus.fromMap(result);
    } catch (e) {
      print('Failed to get VPN status: $e');
      return VpnStatus.disconnected();
    }
  }
}
```

## 🔧 Advanced Configuration

### 1. Custom Configuration Class

```dart
class AdvancedVpnConfig extends VpnConfig {
  final bool enableCompression;
  final int keepAliveInterval;
  final int connectionTimeout;
  final String? customCertificate;
  final bool enableLogging;
  
  AdvancedVpnConfig({
    required String serverHost,
    required int serverPort,
    required String username,
    required String password,
    required String hubName,
    this.enableCompression = true,
    this.keepAliveInterval = 30,
    this.connectionTimeout = 10,
    this.customCertificate,
    this.enableLogging = false,
  }) : super(
    serverHost: serverHost,
    serverPort: serverPort,
    username: username,
    password: password,
    hubName: hubName,
  );
  
  @override
  Map<String, dynamic> toMap() {
    final map = super.toMap();
    map.addAll({
      'enable_compression': enableCompression,
      'keep_alive_interval': keepAliveInterval,
      'connection_timeout': connectionTimeout,
      'enable_logging': enableLogging,
    });
    
    if (customCertificate != null) {
      map['custom_certificate'] = customCertificate;
    }
    
    return map;
  }
}
```

### 2. Connection Status Monitoring

```dart
class VpnStatus {
  final VpnState state;
  final String? serverIp;
  final String? localIp;
  final int bytesIn;
  final int bytesOut;
  final DateTime? connectedSince;
  final String? errorMessage;
  
  VpnStatus({
    required this.state,
    this.serverIp,
    this.localIp,
    this.bytesIn = 0,
    this.bytesOut = 0,
    this.connectedSince,
    this.errorMessage,
  });
  
  factory VpnStatus.fromMap(Map<String, dynamic> map) {
    return VpnStatus(
      state: VpnState.values[map['state'] ?? 0],
      serverIp: map['server_ip'],
      localIp: map['local_ip'],
      bytesIn: map['bytes_in'] ?? 0,
      bytesOut: map['bytes_out'] ?? 0,
      connectedSince: map['connected_since'] != null 
        ? DateTime.fromMillisecondsSinceEpoch(map['connected_since'])
        : null,
      errorMessage: map['error_message'],
    );
  }
  
  factory VpnStatus.disconnected() {
    return VpnStatus(state: VpnState.disconnected);
  }
}

enum VpnState {
  disconnected,
  connecting,
  connected,
  disconnecting,
  error,
}
```

### 3. Real-time Status Updates

```dart
class VpnStatusProvider extends ChangeNotifier {
  VpnStatus _status = VpnStatus.disconnected();
  Timer? _statusTimer;
  
  VpnStatus get status => _status;
  
  void startMonitoring() {
    _statusTimer = Timer.periodic(Duration(seconds: 1), (timer) async {
      final newStatus = await VpnService.getStatus();
      if (_status.state != newStatus.state || 
          _status.bytesIn != newStatus.bytesIn ||
          _status.bytesOut != newStatus.bytesOut) {
        _status = newStatus;
        notifyListeners();
      }
    });
  }
  
  void stopMonitoring() {
    _statusTimer?.cancel();
    _statusTimer = null;
  }
  
  @override
  void dispose() {
    stopMonitoring();
    super.dispose();
  }
}
```

## 🎨 UI Integration

### 1. VPN Connection Widget

```dart
class VpnConnectionWidget extends StatefulWidget {
  final VpnConfig config;
  
  const VpnConnectionWidget({Key? key, required this.config}) : super(key: key);
  
  @override
  _VpnConnectionWidgetState createState() => _VpnConnectionWidgetState();
}

class _VpnConnectionWidgetState extends State<VpnConnectionWidget> {
  bool _isConnecting = false;
  VpnStatus _status = VpnStatus.disconnected();
  
  @override
  void initState() {
    super.initState();
    _updateStatus();
  }
  
  Future<void> _updateStatus() async {
    final status = await VpnService.getStatus();
    setState(() {
      _status = status;
    });
  }
  
  Future<void> _toggleConnection() async {
    if (_status.state == VpnState.connected) {
      setState(() => _isConnecting = true);
      await VpnService.disconnect();
    } else if (_status.state == VpnState.disconnected) {
      setState(() => _isConnecting = true);
      await VpnService.connect(widget.config);
    }
    
    await _updateStatus();
    setState(() => _isConnecting = false);
  }
  
  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: EdgeInsets.all(16),
        child: Column(
          children: [
            Row(
              children: [
                Icon(
                  _getStatusIcon(),
                  color: _getStatusColor(),
                  size: 24,
                ),
                SizedBox(width: 12),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        _getStatusText(),
                        style: Theme.of(context).textTheme.titleMedium,
                      ),
                      if (_status.serverIp != null)
                        Text(
                          'Server: ${_status.serverIp}',
                          style: Theme.of(context).textTheme.bodySmall,
                        ),
                    ],
                  ),
                ),
              ],
            ),
            SizedBox(height: 16),
            if (_status.state == VpnState.connected) ...[
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceAround,
                children: [
                  Column(
                    children: [
                      Text('Downloaded'),
                      Text(_formatBytes(_status.bytesIn)),
                    ],
                  ),
                  Column(
                    children: [
                      Text('Uploaded'),
                      Text(_formatBytes(_status.bytesOut)),
                    ],
                  ),
                ],
              ),
              SizedBox(height: 16),
            ],
            SizedBox(
              width: double.infinity,
              child: ElevatedButton(
                onPressed: _isConnecting ? null : _toggleConnection,
                child: _isConnecting
                  ? SizedBox(
                      height: 20,
                      width: 20,
                      child: CircularProgressIndicator(strokeWidth: 2),
                    )
                  : Text(_getButtonText()),
              ),
            ),
          ],
        ),
      ),
    );
  }
  
  IconData _getStatusIcon() {
    switch (_status.state) {
      case VpnState.connected:
        return Icons.vpn_key;
      case VpnState.connecting:
      case VpnState.disconnecting:
        return Icons.sync;
      case VpnState.error:
        return Icons.error;
      default:
        return Icons.vpn_key_off;
    }
  }
  
  Color _getStatusColor() {
    switch (_status.state) {
      case VpnState.connected:
        return Colors.green;
      case VpnState.error:
        return Colors.red;
      default:
        return Colors.grey;
    }
  }
  
  String _getStatusText() {
    switch (_status.state) {
      case VpnState.connected:
        return 'Connected';
      case VpnState.connecting:
        return 'Connecting...';
      case VpnState.disconnecting:
        return 'Disconnecting...';
      case VpnState.error:
        return 'Error: ${_status.errorMessage ?? 'Unknown error'}';
      default:
        return 'Disconnected';
    }
  }
  
  String _getButtonText() {
    switch (_status.state) {
      case VpnState.connected:
        return 'Disconnect';
      default:
        return 'Connect';
    }
  }
  
  String _formatBytes(int bytes) {
    if (bytes < 1024) return '$bytes B';
    if (bytes < 1024 * 1024) return '${(bytes / 1024).toStringAsFixed(1)} KB';
    return '${(bytes / (1024 * 1024)).toStringAsFixed(1)} MB';
  }
}
```

### 2. Settings Screen

```dart
class VpnSettingsScreen extends StatefulWidget {
  @override
  _VpnSettingsScreenState createState() => _VpnSettingsScreenState();
}

class _VpnSettingsScreenState extends State<VpnSettingsScreen> {
  final _formKey = GlobalKey<FormState>();
  final _hostController = TextEditingController();
  final _portController = TextEditingController(text: '443');
  final _usernameController = TextEditingController();
  final _passwordController = TextEditingController();
  final _hubController = TextEditingController();
  
  bool _enableCompression = true;
  bool _enableLogging = false;
  int _keepAliveInterval = 30;
  
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text('VPN Settings')),
      body: Form(
        key: _formKey,
        child: ListView(
          padding: EdgeInsets.all(16),
          children: [
            TextFormField(
              controller: _hostController,
              decoration: InputDecoration(
                labelText: 'Server Host',
                hintText: 'vpn.example.com',
              ),
              validator: (value) {
                if (value?.isEmpty ?? true) {
                  return 'Please enter server host';
                }
                return null;
              },
            ),
            SizedBox(height: 16),
            TextFormField(
              controller: _portController,
              decoration: InputDecoration(labelText: 'Port'),
              keyboardType: TextInputType.number,
              validator: (value) {
                if (value?.isEmpty ?? true) {
                  return 'Please enter port';
                }
                final port = int.tryParse(value!);
                if (port == null || port < 1 || port > 65535) {
                  return 'Please enter valid port (1-65535)';
                }
                return null;
              },
            ),
            SizedBox(height: 16),
            TextFormField(
              controller: _usernameController,
              decoration: InputDecoration(labelText: 'Username'),
              validator: (value) {
                if (value?.isEmpty ?? true) {
                  return 'Please enter username';
                }
                return null;
              },
            ),
            SizedBox(height: 16),
            TextFormField(
              controller: _passwordController,
              decoration: InputDecoration(labelText: 'Password'),
              obscureText: true,
              validator: (value) {
                if (value?.isEmpty ?? true) {
                  return 'Please enter password';
                }
                return null;
              },
            ),
            SizedBox(height: 16),
            TextFormField(
              controller: _hubController,
              decoration: InputDecoration(
                labelText: 'Hub Name',
                hintText: 'VPN',
              ),
              validator: (value) {
                if (value?.isEmpty ?? true) {
                  return 'Please enter hub name';
                }
                return null;
              },
            ),
            SizedBox(height: 24),
            Text(
              'Advanced Settings',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            SwitchListTile(
              title: Text('Enable Compression'),
              subtitle: Text('Compress data to reduce bandwidth usage'),
              value: _enableCompression,
              onChanged: (value) => setState(() => _enableCompression = value),
            ),
            SwitchListTile(
              title: Text('Enable Logging'),
              subtitle: Text('Log connection events for debugging'),
              value: _enableLogging,
              onChanged: (value) => setState(() => _enableLogging = value),
            ),
            ListTile(
              title: Text('Keep Alive Interval'),
              subtitle: Text('${_keepAliveInterval}s'),
              trailing: Slider(
                value: _keepAliveInterval.toDouble(),
                min: 10,
                max: 120,
                divisions: 11,
                label: '${_keepAliveInterval}s',
                onChanged: (value) => setState(() => _keepAliveInterval = value.toInt()),
              ),
            ),
            SizedBox(height: 24),
            ElevatedButton(
              onPressed: _saveSettings,
              child: Text('Save Settings'),
            ),
          ],
        ),
      ),
    );
  }
  
  void _saveSettings() {
    if (_formKey.currentState?.validate() ?? false) {
      final config = AdvancedVpnConfig(
        serverHost: _hostController.text,
        serverPort: int.parse(_portController.text),
        username: _usernameController.text,
        password: _passwordController.text,
        hubName: _hubController.text,
        enableCompression: _enableCompression,
        enableLogging: _enableLogging,
        keepAliveInterval: _keepAliveInterval,
      );
      
      // Save configuration (implement your storage logic)
      Navigator.pop(context, config);
    }
  }
  
  @override
  void dispose() {
    _hostController.dispose();
    _portController.dispose();
    _usernameController.dispose();
    _passwordController.dispose();
    _hubController.dispose();
    super.dispose();
  }
}
```

## 🔒 Security Best Practices

### 1. Credential Storage

```dart
import 'package:flutter_secure_storage/flutter_secure_storage.dart';

class SecureVpnStorage {
  static const _storage = FlutterSecureStorage();
  
  static Future<void> saveCredentials(String username, String password) async {
    await _storage.write(key: 'vpn_username', value: username);
    await _storage.write(key: 'vpn_password', value: password);
  }
  
  static Future<Map<String, String?>> getCredentials() async {
    final username = await _storage.read(key: 'vpn_username');
    final password = await _storage.read(key: 'vpn_password');
    return {'username': username, 'password': password};
  }
  
  static Future<void> clearCredentials() async {
    await _storage.delete(key: 'vpn_username');
    await _storage.delete(key: 'vpn_password');
  }
}
```

### 2. Certificate Validation

```dart
class CertificateValidator {
  static Future<bool> validateCertificate(String certificate) async {
    try {
      // Implement your certificate validation logic
      // Check against known good certificates
      // Verify certificate chain
      // Check expiration date
      return true;
    } catch (e) {
      print('Certificate validation failed: $e');
      return false;
    }
  }
}
```

## 🧪 Testing

### 1. Unit Tests

```dart
// test/vpn_service_test.dart
import 'package:flutter_test/flutter_test.dart';
import 'package:mockito/mockito.dart';
import 'package:rvpnse_flutter/rvpnse_flutter.dart';

class MockRvpnseFlutter extends Mock implements RvpnseFlutter {}

void main() {
  group('VpnService Tests', () {
    late MockRvpnseFlutter mockRvpnse;
    
    setUp(() {
      mockRvpnse = MockRvpnseFlutter();
    });
    
    test('should connect successfully with valid config', () async {
      // Arrange
      final config = VpnConfig(
        serverHost: 'test.example.com',
        serverPort: 443,
        username: 'testuser',
        password: 'testpass',
        hubName: 'VPN',
      );
      
      when(mockRvpnse.connect(any))
          .thenAnswer((_) async => {'success': true});
      
      // Act
      final result = await VpnService.connect(config);
      
      // Assert
      expect(result, isTrue);
      verify(mockRvpnse.connect(config.toMap())).called(1);
    });
    
    test('should handle connection failure gracefully', () async {
      // Arrange
      final config = VpnConfig(
        serverHost: 'invalid.example.com',
        serverPort: 443,
        username: 'testuser',
        password: 'testpass',
        hubName: 'VPN',
      );
      
      when(mockRvpnse.connect(any))
          .thenThrow(Exception('Connection failed'));
      
      // Act
      final result = await VpnService.connect(config);
      
      // Assert
      expect(result, isFalse);
    });
  });
}
```

### 2. Integration Tests

```dart
// integration_test/vpn_integration_test.dart
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:myapp/main.dart' as app;

void main() {
  IntegrationTestWidgetsBinding.ensureInitialized();
  
  group('VPN Integration Tests', () {
    testWidgets('should connect and disconnect from VPN', (tester) async {
      app.main();
      await tester.pumpAndSettle();
      
      // Find connection button and tap it
      final connectButton = find.text('Connect');
      expect(connectButton, findsOneWidget);
      
      await tester.tap(connectButton);
      await tester.pumpAndSettle();
      
      // Wait for connection
      await tester.pump(Duration(seconds: 5));
      
      // Verify connection status
      expect(find.text('Connected'), findsOneWidget);
      
      // Disconnect
      final disconnectButton = find.text('Disconnect');
      await tester.tap(disconnectButton);
      await tester.pumpAndSettle();
      
      // Verify disconnection
      expect(find.text('Disconnected'), findsOneWidget);
    });
  });
}
```

## 📱 Platform-Specific Code

### iOS Platform Implementation

```objc
// ios/Classes/RvpnseFlutterPlugin.m
#import "RvpnseFlutterPlugin.h"
#import "rvpnse.h"

@implementation RvpnseFlutterPlugin {
    RvpnseClient* _client;
    RvpnseConfig* _config;
}

+ (void)registerWithRegistrar:(NSObject<FlutterPluginRegistrar>*)registrar {
    FlutterMethodChannel* channel = [FlutterMethodChannel
        methodChannelWithName:@"rvpnse_flutter"
              binaryMessenger:[registrar messenger]];
    RvpnseFlutterPlugin* instance = [[RvpnseFlutterPlugin alloc] init];
    [registrar addMethodCallDelegate:instance channel:channel];
}

- (void)handleMethodCall:(FlutterMethodCall*)call result:(FlutterResult)result {
    if ([@"initialize" isEqualToString:call.method]) {
        [self initialize:result];
    } else if ([@"connect" isEqualToString:call.method]) {
        [self connect:call.arguments result:result];
    } else if ([@"disconnect" isEqualToString:call.method]) {
        [self disconnect:result];
    } else if ([@"getStatus" isEqualToString:call.method]) {
        [self getStatus:result];
    } else {
        result(FlutterMethodNotImplemented);
    }
}

- (void)initialize:(FlutterResult)result {
    rvpnse_init();
    result(@{@"success": @YES});
}

- (void)connect:(NSDictionary*)config result:(FlutterResult)result {
    @try {
        _config = rvpnse_config_new();
        
        rvpnse_config_set_server_host(_config, [config[@"server_host"] UTF8String]);
        rvpnse_config_set_server_port(_config, [config[@"server_port"] intValue]);
        rvpnse_config_set_username(_config, [config[@"username"] UTF8String]);
        rvpnse_config_set_password(_config, [config[@"password"] UTF8String]);
        rvpnse_config_set_hub_name(_config, [config[@"hub_name"] UTF8String]);
        
        _client = rvpnse_client_new(_config);
        RvpnseResult connectResult = rvpnse_client_connect(_client);
        
        if (connectResult == rVPNSE_SUCCESS) {
            result(@{@"success": @YES});
        } else {
            result(@{@"success": @NO, @"error": @"Connection failed"});
        }
    } @catch (NSException *exception) {
        result(@{@"success": @NO, @"error": exception.reason});
    }
}

- (void)disconnect:(FlutterResult)result {
    if (_client) {
        rvpnse_client_disconnect(_client);
        rvpnse_client_free(_client);
        _client = NULL;
        
        if (_config) {
            rvpnse_config_free(_config);
            _config = NULL;
        }
    }
    result(@{@"success": @YES});
}

- (void)getStatus:(FlutterResult)result {
    if (_client) {
        RvpnseConnectionState state = rvpnse_client_get_state(_client);
        RvpnseStats stats = rvpnse_client_get_stats(_client);
        
        result(@{
            @"state": @(state),
            @"server_ip": @"192.168.1.1", // Get actual server IP
            @"local_ip": @"10.0.0.2",     // Get actual local IP
            @"bytes_in": @(stats.bytes_received),
            @"bytes_out": @(stats.bytes_sent),
            @"connected_since": @([[NSDate date] timeIntervalSince1970] * 1000)
        });
    } else {
        result(@{@"state": @0}); // Disconnected
    }
}

@end
```

### Android Platform Implementation

```kotlin
// android/src/main/kotlin/com/devstroop/rvpnse_flutter/RvpnseFlutterPlugin.kt
package com.devstroop.rvpnse_flutter

import io.flutter.plugin.common.MethodCall
import io.flutter.plugin.common.MethodChannel
import io.flutter.plugin.common.MethodChannel.MethodCallHandler
import io.flutter.plugin.common.MethodChannel.Result

class RvpnseFlutterPlugin: MethodCallHandler {
    companion object {
        init {
            System.loadLibrary("rvpnse")
        }
    }
    
    private external fun nativeInit(): Boolean
    private external fun nativeConnect(
        host: String,
        port: Int,
        username: String,
        password: String,
        hubName: String
    ): Boolean
    private external fun nativeDisconnect(): Boolean
    private external fun nativeGetStatus(): Map<String, Any>
    
    override fun onMethodCall(call: MethodCall, result: Result) {
        when (call.method) {
            "initialize" -> {
                val success = nativeInit()
                result.success(mapOf("success" to success))
            }
            "connect" -> {
                val args = call.arguments as Map<String, Any>
                val success = nativeConnect(
                    args["server_host"] as String,
                    args["server_port"] as Int,
                    args["username"] as String,
                    args["password"] as String,
                    args["hub_name"] as String
                )
                result.success(mapOf("success" to success))
            }
            "disconnect" -> {
                val success = nativeDisconnect()
                result.success(mapOf("success" to success))
            }
            "getStatus" -> {
                val status = nativeGetStatus()
                result.success(status)
            }
            else -> {
                result.notImplemented()
            }
        }
    }
}
```

## 🚀 Publishing

### 1. Prepare for Publishing

```yaml
# pubspec.yaml
name: your_vpn_app
description: A secure VPN application using rVPNSE
version: 1.0.0+1

flutter:
  uses-material-design: true
  
dependencies:
  flutter:
    sdk: flutter
  rvpnse_flutter: ^1.0.0
  flutter_secure_storage: ^9.0.0
  provider: ^6.0.0
```

### 2. Build Release

```bash
# iOS
flutter build ios --release

# Android
flutter build apk --release
flutter build appbundle --release
```

## 🆘 Troubleshooting

| Issue | Solution |
|-------|----------|
| **Plugin not found** | Run `flutter clean && flutter pub get` |
| **iOS build fails** | Check Xcode version and iOS deployment target |
| **Android permission denied** | Request VPN permission from user |
| **Connection timeout** | Check network connectivity and server status |
| **Native library not loaded** | Verify platform-specific setup is correct |

## 📚 Next Steps

1. **Set up development environment** for your target platforms
2. **Implement basic VPN functionality** using the examples above
3. **Add UI components** for connection management
4. **Implement secure credential storage**
5. **Test on real devices** with various network conditions
6. **Prepare for app store submission**

**Need help?** Check our [Flutter-specific troubleshooting](../../07-troubleshooting/flutter.md) or [join our community](https://github.com/devstroop/rvpnse/discussions).
