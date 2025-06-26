# 📱 Mobile Integration Guide

Complete integration guides for mobile platforms. rVPNSE supports both iOS and Android with platform-specific optimizations for battery efficiency and network handling.

## 📋 Platform Support

| Platform | Minimum Version | Technologies | Status |
|----------|----------------|--------------|--------|
| **iOS** | iOS 12.0+ | Swift, Objective-C, NetworkExtension | ✅ Production Ready |
| **Android** | API 21+ (Android 5.0) | Kotlin, Java, NDK | ✅ Production Ready |
| **Flutter** | Flutter 3.0+ | Dart, Platform Channels | ✅ Production Ready |
| **React Native** | RN 0.68+ | TypeScript, Native Modules | 🔄 Beta |
| **Xamarin** | Xamarin.iOS 15+, Xamarin.Android 12+ | C# | 🔄 Beta |

## 🚀 Quick Start

### iOS (Swift)
```bash
# Add rVPNSE framework to your iOS project
curl -L -o RvpnseFramework.zip \
  https://github.com/devstroop/rvpnse/releases/latest/download/rvpnse-ios-framework.zip
unzip RvpnseFramework.zip
```

### Android (Kotlin)
```bash
# Add to your app/build.gradle
implementation 'com.devstroop:rvpnse-android:1.0.0'
```

### Flutter
```bash
# Add to pubspec.yaml
flutter pub add rvpnse_flutter
```

## 📚 Platform Guides

- [📱 **iOS Integration**](ios.md) - NetworkExtension, App Store compliance, Swift/Objective-C
- [🤖 **Android Integration**](android.md) - VpnService, Play Store compliance, Kotlin/Java
- [🎯 **Flutter Integration**](flutter.md) - Cross-platform with platform channels
- [⚛️ **React Native**](react-native.md) - Native modules and bridge setup
- [📱 **Xamarin Integration**](xamarin.md) - C# wrapper and platform-specific code

## 🔑 Key Mobile Features

### **Battery Optimization**
- Smart connection management based on network conditions
- Background processing optimization
- Intelligent retry mechanisms to minimize battery drain

### **Network Awareness**
- Automatic handling of WiFi/cellular transitions
- Network quality detection and adaptation
- Seamless roaming between networks

### **Security**
- Certificate pinning support
- Secure credential storage (Keychain/Keystore)
- Biometric authentication integration

### **Platform Integration**
- iOS NetworkExtension framework support
- Android VpnService implementation
- System VPN settings integration

## 🛡️ Security Considerations

### **Credential Management**
```swift
// iOS Keychain integration
import Security

class VpnCredentialManager {
    static func storeCredentials(username: String, password: String) -> Bool {
        let query: [String: Any] = [
            kSecClass as String: kSecClassInternetPassword,
            kSecAttrAccount as String: username,
            kSecValueData as String: password.data(using: .utf8)!
        ]
        
        SecItemDelete(query as CFDictionary)
        return SecItemAdd(query as CFDictionary, nil) == errSecSuccess
    }
}
```

### **Android Secure Storage**
```kotlin
// Android EncryptedSharedPreferences
class VpnCredentialManager(context: Context) {
    private val sharedPrefs = EncryptedSharedPreferences.create(
        "vpn_credentials",
        MasterKeys.getOrCreate(MasterKeys.AES256_GCM_SPEC),
        context,
        EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
        EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM
    )
    
    fun storeCredentials(username: String, password: String) {
        sharedPrefs.edit()
            .putString("username", username)
            .putString("password", password)
            .apply()
    }
}
```

## 📱 App Store Guidelines

### **iOS App Store**
- ✅ Use NetworkExtension framework for system VPN integration
- ✅ Clearly describe VPN functionality in app description
- ✅ Include privacy policy explaining data handling
- ❌ Don't bypass iOS networking APIs
- ❌ Don't access other app's network traffic

### **Google Play Store**
- ✅ Use VpnService for system VPN integration
- ✅ Request VPN permission explicitly
- ✅ Include privacy policy and data safety information
- ❌ Don't use VPN for ad blocking only
- ❌ Don't access sensitive data without permission

## 🧪 Testing on Mobile

### **iOS Testing**
```swift
// Unit test example
import XCTest
@testable import YourApp

class VPNIntegrationTests: XCTestCase {
    func testVPNConnection() async throws {
        let config = try RvpnseConfig.fromFile("test-config.toml")
        let client = try RvpnseClient(config: config)
        
        try await client.connect()
        XCTAssertEqual(client.state, .connected)
        
        try await client.disconnect()
        XCTAssertEqual(client.state, .disconnected)
    }
}
```

### **Android Testing**
```kotlin
// Instrumented test example
@RunWith(AndroidJUnit4::class)
class VpnIntegrationTest {
    @Test
    fun testVpnConnection() = runTest {
        val config = RvpnseConfig.fromFile("test-config.toml")
        val client = RvpnseClient(config)
        
        client.connect()
        assertEquals(ConnectionState.CONNECTED, client.state)
        
        client.disconnect()
        assertEquals(ConnectionState.DISCONNECTED, client.state)
    }
}
```

## 🔄 Continuous Integration

### **iOS CI/CD**
```yaml
# .github/workflows/ios.yml
name: iOS Build
on: [push, pull_request]

jobs:
  ios_build:
    runs-on: macos-latest
    steps:
    - uses: actions/checkout@v3
    - name: Build iOS Framework
      run: |
        cd ios/
        xcodebuild -scheme RvpnseFramework -configuration Release
    - name: Run Tests
      run: |
        xcodebuild test -scheme RvpnseFramework -destination 'platform=iOS Simulator,name=iPhone 14'
```

### **Android CI/CD**
```yaml
# .github/workflows/android.yml
name: Android Build
on: [push, pull_request]

jobs:
  android_build:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - name: Set up JDK 11
      uses: actions/setup-java@v3
      with:
        java-version: '11'
        distribution: 'temurin'
    - name: Build AAR
      run: ./gradlew assembleRelease
    - name: Run Tests
      run: ./gradlew test
```

## 📊 Performance Optimization

### **Memory Management**
- Proper cleanup of native resources
- Background processing limitations
- Memory pressure handling

### **Network Efficiency**
- Connection pooling for multiple requests
- Intelligent retry with exponential backoff
- Data compression when beneficial

### **Power Management**
- Background app refresh optimization
- Smart scheduling for maintenance tasks
- Battery level aware operation

## 🆘 Troubleshooting

| Issue | Platform | Solution |
|-------|----------|----------|
| **Permission Denied** | Both | Check VPN permissions in system settings |
| **Connection Timeout** | Both | Verify network connectivity and firewall |
| **App Store Rejection** | iOS | Ensure NetworkExtension usage compliance |
| **Play Store Warning** | Android | Review VPN permission usage policy |
| **Battery Drain** | Both | Implement smart connection management |

## 📚 Additional Resources

- [iOS NetworkExtension Documentation](https://developer.apple.com/documentation/networkextension)
- [Android VpnService Documentation](https://developer.android.com/reference/android/net/VpnService)
- [App Store Review Guidelines - VPN Apps](https://developer.apple.com/app-store/review/guidelines/#vpn-apps)
- [Google Play Policy - VPN Services](https://support.google.com/googleplay/android-developer/answer/9888379)

## 🎯 Next Steps

1. **Choose your platform** from the guides above
2. **Set up development environment** following platform-specific setup
3. **Integrate rVPNSE** using platform-specific APIs
4. **Test thoroughly** on real devices and various network conditions
5. **Submit for review** following app store guidelines

**Need help?** Check our [Troubleshooting Guide](../../07-troubleshooting/README.md) or join our [community discussions](https://github.com/devstroop/rvpnse/discussions).
