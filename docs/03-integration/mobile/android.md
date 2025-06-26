# Android Integration Guide

Integrate rVPNSEust into Android applications using Kotlin/Java and the VpnService framework.

## 📋 Prerequisites

- **Android Studio** with API level 21+ (Android 5.0+)
- **NDK** for native library integration
- **rVPNSEust static library** built for Android targets
- **VPN permissions** in your app manifest

## 🛠️ Step 1: Build for Android

```bash
# Add Android targets
rustup target add aarch64-linux-android     # ARM64 (modern devices)
rustup target add armv7-linux-androideabi   # ARMv7 (older devices)
rustup target add x86_64-linux-android      # x86_64 (emulator)
rustup target add i686-linux-android        # x86 (emulator)

# Configure cargo for cross-compilation
export ANDROID_NDK_HOME=/path/to/android-ndk
export AR_aarch64_linux_android=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android-ar
export CC_aarch64_linux_android=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android21-clang

# Build for Android
cargo build --release --target aarch64-linux-android
```

## 📦 Step 2: Create JNI Bindings

Create `app/src/main/cpp/vpnse_jni.cpp`:

```cpp
#include <jni.h>
#include <string>
#include "rvpnse.h"

extern "C" {

JNIEXPORT jlong JNICALL
Java_com_yourpackage_VPNSEClient_nativeCreateClient(JNIEnv *env, jobject /* this */, jstring config) {
    const char *configStr = env->GetStringUTFChars(config, nullptr);
    vpnse_client_t* client = vpnse_client_new(configStr);
    env->ReleaseStringUTFChars(config, configStr);
    return reinterpret_cast<jlong>(client);
}

JNIEXPORT jint JNICALL
Java_com_yourpackage_VPNSEClient_nativeConnect(JNIEnv *env, jobject /* this */, 
    jlong clientHandle, jstring server, jint port) {
    vpnse_client_t* client = reinterpret_cast<vpnse_client_t*>(clientHandle);
    const char *serverStr = env->GetStringUTFChars(server, nullptr);
    
    int result = vpnse_client_connect(client, serverStr, (uint16_t)port);
    
    env->ReleaseStringUTFChars(server, serverStr);
    return result;
}

JNIEXPORT jint JNICALL
Java_com_yourpackage_VPNSEClient_nativeAuthenticate(JNIEnv *env, jobject /* this */,
    jlong clientHandle, jstring username, jstring password) {
    vpnse_client_t* client = reinterpret_cast<vpnse_client_t*>(clientHandle);
    const char *userStr = env->GetStringUTFChars(username, nullptr);
    const char *passStr = env->GetStringUTFChars(password, nullptr);
    
    int result = vpnse_client_authenticate(client, userStr, passStr);
    
    env->ReleaseStringUTFChars(username, userStr);
    env->ReleaseStringUTFChars(password, passStr);
    return result;
}

JNIEXPORT void JNICALL
Java_com_yourpackage_VPNSEClient_nativeDisconnect(JNIEnv *env, jobject /* this */, jlong clientHandle) {
    vpnse_client_t* client = reinterpret_cast<vpnse_client_t*>(clientHandle);
    vpnse_client_disconnect(client);
}

JNIEXPORT void JNICALL
Java_com_yourpackage_VPNSEClient_nativeFreeClient(JNIEnv *env, jobject /* this */, jlong clientHandle) {
    vpnse_client_t* client = reinterpret_cast<vpnse_client_t*>(clientHandle);
    vpnse_client_free(client);
}

}
```

## 🎯 Step 3: Kotlin VPN Client

Create `VPNSEClient.kt`:

```kotlin
package com.yourpackage

class VPNSEClient {
    private var clientHandle: Long = 0
    
    companion object {
        init {
            System.loadLibrary("rvpnse")
            System.loadLibrary("vpnse_jni")
        }
        
        const val VPNSE_SUCCESS = 0
        const val VPNSE_ERROR_INVALID_CONFIG = -1
        const val VPNSE_ERROR_CONNECTION_FAILED = -2
        const val VPNSE_ERROR_AUTH_FAILED = -3
    }
    
    fun createClient(config: String): Boolean {
        clientHandle = nativeCreateClient(config)
        return clientHandle != 0L
    }
    
    fun connect(server: String, port: Int): Boolean {
        if (clientHandle == 0L) return false
        val result = nativeConnect(clientHandle, server, port)
        return result == VPNSE_SUCCESS
    }
    
    fun authenticate(username: String, password: String): Boolean {
        if (clientHandle == 0L) return false
        val result = nativeAuthenticate(clientHandle, username, password)
        return result == VPNSE_SUCCESS
    }
    
    fun disconnect() {
        if (clientHandle != 0L) {
            nativeDisconnect(clientHandle)
        }
    }
    
    fun release() {
        if (clientHandle != 0L) {
            nativeFreeClient(clientHandle)
            clientHandle = 0
        }
    }
    
    // Native method declarations
    private external fun nativeCreateClient(config: String): Long
    private external fun nativeConnect(clientHandle: Long, server: String, port: Int): Int
    private external fun nativeAuthenticate(clientHandle: Long, username: String, password: String): Int
    private external fun nativeDisconnect(clientHandle: Long)
    private external fun nativeFreeClient(clientHandle: Long)
}
```

## 🔌 Step 4: Android VPN Service

Create `VPNSEVpnService.kt`:

```kotlin
package com.yourpackage

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.content.Intent
import android.net.VpnService
import android.os.Build
import android.os.ParcelFileDescriptor
import android.util.Log
import kotlinx.coroutines.*
import java.io.FileInputStream
import java.io.FileOutputStream
import java.nio.ByteBuffer

class VPNSEVpnService : VpnService() {
    companion object {
        private const val TAG = "VPNSEVpnService"
        private const val NOTIFICATION_ID = 1
        private const val CHANNEL_ID = "VPNSE_CHANNEL"
    }
    
    private var vpnInterface: ParcelFileDescriptor? = null
    private var rvpnseClient: VPNSEClient? = null
    private var isRunning = false
    private var serviceScope = CoroutineScope(Dispatchers.IO + SupervisorJob())
    
    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()
    }
    
    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        val action = intent?.action
        
        when (action) {
            "CONNECT" -> {
                val config = intent.getStringExtra("config") ?: return START_NOT_STICKY
                val server = intent.getStringExtra("server") ?: return START_NOT_STICKY
                val username = intent.getStringExtra("username") ?: return START_NOT_STICKY
                val password = intent.getStringExtra("password") ?: return START_NOT_STICKY
                
                connectVpn(config, server, username, password)
            }
            "DISCONNECT" -> {
                disconnectVpn()
            }
        }
        
        return START_STICKY
    }
    
    private fun connectVpn(config: String, server: String, username: String, password: String) {
        serviceScope.launch {
            try {
                // 1. Create rVPNSEust client
                rvpnseClient = VPNSEClient().apply {
                    if (!createClient(config)) {
                        Log.e(TAG, "Failed to create VPNSE client")
                        return@launch
                    }
                    
                    // 2. Connect to SoftEther server
                    if (!connect(server, 443)) {
                        Log.e(TAG, "Failed to connect to server")
                        return@launch
                    }
                    
                    // 3. Authenticate
                    if (!authenticate(username, password)) {
                        Log.e(TAG, "Authentication failed")
                        return@launch
                    }
                }
                
                // 4. Create VPN interface
                createVpnInterface(server)
                
                // 5. Start packet forwarding
                startPacketForwarding()
                
                Log.i(TAG, "VPN connected successfully")
                
            } catch (e: Exception) {
                Log.e(TAG, "VPN connection failed", e)
                disconnectVpn()
            }
        }
    }
    
    private fun createVpnInterface(server: String) {
        val builder = Builder()
            .setSession("VPNSE")
            .addAddress("10.0.0.2", 24)
            .addRoute("0.0.0.0", 0)
            .addDnsServer("8.8.8.8")
            .addDnsServer("8.8.4.4")
            .setMtu(1500)
        
        // Allow all apps to use VPN
        try {
            builder.addAllowedApplication(packageName)
        } catch (e: Exception) {
            Log.w(TAG, "Failed to add allowed application", e)
        }
        
        vpnInterface = builder.establish()
        
        if (vpnInterface == null) {
            throw RuntimeException("Failed to create VPN interface")
        }
        
        startForeground(NOTIFICATION_ID, createNotification("Connected to $server"))
        isRunning = true
    }
    
    private fun startPacketForwarding() {
        val vpnFd = vpnInterface ?: return
        val inputStream = FileInputStream(vpnFd.fileDescriptor)
        val outputStream = FileOutputStream(vpnFd.fileDescriptor)
        
        // Forward packets from VPN interface to SoftEther
        serviceScope.launch {
            val buffer = ByteArray(32767)
            
            while (isRunning) {
                try {
                    val length = inputStream.read(buffer)
                    if (length > 0) {
                        // Forward packet to SoftEther server via rVPNSEust
                        // Note: This is simplified - you need to implement packet forwarding
                        forwardPacketToServer(buffer, length)
                    }
                } catch (e: Exception) {
                    if (isRunning) {
                        Log.e(TAG, "Error reading from VPN interface", e)
                    }
                    break
                }
            }
        }
        
        // Forward packets from SoftEther to VPN interface
        serviceScope.launch {
            while (isRunning) {
                try {
                    // Receive packet from SoftEther server via rVPNSEust
                    val packet = receivePacketFromServer()
                    if (packet != null) {
                        outputStream.write(packet)
                    }
                } catch (e: Exception) {
                    if (isRunning) {
                        Log.e(TAG, "Error writing to VPN interface", e)
                    }
                    break
                }
            }
        }
    }
    
    private fun forwardPacketToServer(buffer: ByteArray, length: Int) {
        // TODO: Implement packet forwarding to SoftEther via rVPNSEust
        // This requires extending the C API to handle packet I/O
    }
    
    private fun receivePacketFromServer(): ByteArray? {
        // TODO: Implement packet receiving from SoftEther via rVPNSEust
        // This requires extending the C API to handle packet I/O
        return null
    }
    
    private fun disconnectVpn() {
        isRunning = false
        
        rvpnseClient?.apply {
            disconnect()
            release()
        }
        rvpnseClient = null
        
        vpnInterface?.close()
        vpnInterface = null
        
        serviceScope.cancel()
        serviceScope = CoroutineScope(Dispatchers.IO + SupervisorJob())
        
        stopForeground(true)
        stopSelf()
        
        Log.i(TAG, "VPN disconnected")
    }
    
    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                CHANNEL_ID,
                "VPNSE Service",
                NotificationManager.IMPORTANCE_LOW
            )
            val manager = getSystemService(NotificationManager::class.java)
            manager.createNotificationChannel(channel)
        }
    }
    
    private fun createNotification(text: String): Notification {
        return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            Notification.Builder(this, CHANNEL_ID)
                .setContentTitle("VPNSE")
                .setContentText(text)
                .setSmallIcon(android.R.drawable.ic_lock_lock)
                .build()
        } else {
            @Suppress("DEPRECATION")
            Notification.Builder(this)
                .setContentTitle("VPNSE")
                .setContentText(text)
                .setSmallIcon(android.R.drawable.ic_lock_lock)
                .build()
        }
    }
    
    override fun onDestroy() {
        disconnectVpn()
        super.onDestroy()
    }
}
```

## 📱 Step 5: MainActivity Integration

Create `MainActivity.kt`:

```kotlin
package com.yourpackage

import android.app.Activity
import android.content.Intent
import android.net.VpnService
import android.os.Bundle
import android.widget.Toast
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp

class MainActivity : ComponentActivity() {
    private var isVpnConnected by mutableStateOf(false)
    
    private val vpnPermissionLauncher = registerForActivityResult(
        ActivityResultContracts.StartActivityForResult()
    ) { result ->
        if (result.resultCode == Activity.RESULT_OK) {
            startVpnConnection()
        } else {
            Toast.makeText(this, "VPN permission denied", Toast.LENGTH_SHORT).show()
        }
    }
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        setContent {
            VpnControlScreen(
                isConnected = isVpnConnected,
                onConnect = { requestVpnPermission() },
                onDisconnect = { disconnectVpn() }
            )
        }
    }
    
    private fun requestVpnPermission() {
        val intent = VpnService.prepare(this)
        if (intent != null) {
            vpnPermissionLauncher.launch(intent)
        } else {
            startVpnConnection()
        }
    }
    
    private fun startVpnConnection() {
        val config = """
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
        """.trimIndent()
        
        val intent = Intent(this, VPNSEVpnService::class.java).apply {
            action = "CONNECT"
            putExtra("config", config)
            putExtra("server", "vpn.example.com")
            putExtra("username", "testuser")
            putExtra("password", "testpass")
        }
        
        startService(intent)
        isVpnConnected = true
    }
    
    private fun disconnectVpn() {
        val intent = Intent(this, VPNSEVpnService::class.java).apply {
            action = "DISCONNECT"
        }
        startService(intent)
        isVpnConnected = false
    }
}

@Composable
fun VpnControlScreen(
    isConnected: Boolean,
    onConnect: () -> Unit,
    onDisconnect: () -> Unit
) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(16.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        Card(
            modifier = Modifier.padding(16.dp),
            elevation = CardDefaults.cardElevation(defaultElevation = 8.dp)
        ) {
            Column(
                modifier = Modifier.padding(24.dp),
                horizontalAlignment = Alignment.CenterHorizontally
            ) {
                Text(
                    text = if (isConnected) "🔒 Connected" else "🔓 Disconnected",
                    style = MaterialTheme.typography.headlineMedium,
                    color = if (isConnected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.error
                )
                
                Spacer(modifier = Modifier.height(16.dp))
                
                Button(
                    onClick = if (isConnected) onDisconnect else onConnect,
                    colors = ButtonDefaults.buttonColors(
                        containerColor = if (isConnected) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.primary
                    )
                ) {
                    Text(if (isConnected) "Disconnect" else "Connect")
                }
            }
        }
        
        Spacer(modifier = Modifier.height(24.dp))
        
        Card(
            modifier = Modifier.fillMaxWidth(),
            elevation = CardDefaults.cardElevation(defaultElevation = 4.dp)
        ) {
            Column(
                modifier = Modifier.padding(16.dp)
            ) {
                Text(
                    text = "VPN Configuration",
                    style = MaterialTheme.typography.titleMedium
                )
                Spacer(modifier = Modifier.height(8.dp))
                Text("Server: vpn.example.com:443")
                Text("Hub: VPN")
                Text("Protocol: SoftEther SSL-VPN")
                Text("Status: ${if (isConnected) "Connected" else "Disconnected"}")
            }
        }
    }
}
```

## 📝 Step 6: Android Manifest

Update `AndroidManifest.xml`:

```xml
<manifest xmlns:android="http://schemas.android.com/apk/res/android">
    
    <!-- VPN permissions -->
    <uses-permission android:name="android.permission.BIND_VPN_SERVICE" />
    <uses-permission android:name="android.permission.INTERNET" />
    <uses-permission android:name="android.permission.ACCESS_NETWORK_STATE" />
    <uses-permission android:name="android.permission.FOREGROUND_SERVICE" />
    
    <application
        android:allowBackup="true"
        android:icon="@mipmap/ic_launcher"
        android:label="@string/app_name"
        android:theme="@style/AppTheme">
        
        <activity
            android:name=".MainActivity"
            android:exported="true">
            <intent-filter>
                <action android:name="android.intent.action.MAIN" />
                <category android:name="android.intent.category.LAUNCHER" />
            </intent-filter>
        </activity>
        
        <service
            android:name=".VPNSEVpnService"
            android:permission="android.permission.BIND_VPN_SERVICE"
            android:exported="false">
            <intent-filter>
                <action android:name="android.net.VpnService" />
            </intent-filter>
        </service>
        
    </application>
    
</manifest>
```

## 🔧 Step 7: Build Configuration

Update `app/build.gradle`:

```gradle
android {
    compileSdk 34
    
    defaultConfig {
        applicationId "com.yourpackage.vpnse"
        minSdk 21
        targetSdk 34
        versionCode 1
        versionName "1.0"
        
        ndk {
            abiFilters 'arm64-v8a', 'armeabi-v7a', 'x86', 'x86_64'
        }
    }
    
    externalNativeBuild {
        cmake {
            path "src/main/cpp/CMakeLists.txt"
            version "3.18.1"
        }
    }
    
    sourceSets {
        main {
            jniLibs.srcDirs = ['src/main/jniLibs']
        }
    }
}

dependencies {
    implementation 'androidx.core:core-ktx:1.12.0'
    implementation 'androidx.activity:activity-compose:1.8.2'
    implementation 'androidx.compose.ui:ui:1.5.4'
    implementation 'androidx.compose.material3:material3:1.1.2'
    implementation 'org.jetbrains.kotlinx:kotlinx-coroutines-android:1.7.3'
}
```

Create `app/src/main/cpp/CMakeLists.txt`:

```cmake
cmake_minimum_required(VERSION 3.18.1)
project("vpnse")

add_library(vpnse_jni SHARED vpnse_jni.cpp)

# Link with rVPNSEust static library
add_library(rvpnse STATIC IMPORTED)
set_target_properties(rvpnse PROPERTIES
    IMPORTED_LOCATION ${CMAKE_SOURCE_DIR}/../jniLibs/${ANDROID_ABI}/librvpnse.a)

target_link_libraries(vpnse_jni rvpnse log)
```

## 🧪 Step 8: Testing

### **Test Configuration Parsing**

```kotlin
class VPNSETest {
    @Test
    fun testConfigurationParsing() {
        val config = """
        [server]
        hostname = "test.example.com"
        port = 443
        hub = "VPN"
        
        [auth]
        method = "password"
        username = "test"
        password = "test"
        """.trimIndent()
        
        val client = VPNSEClient()
        assertTrue(client.createClient(config))
        client.release()
    }
}
```

## 🚨 Common Issues

### **NDK Build Issues**
```
Error: Library not found: librvpnse.a
```
**Solution**: Ensure the static library is copied to the correct ABI folders in `src/main/jniLibs/`.

### **JNI Linking Issues**
```
UnsatisfiedLinkError: No implementation found for native method
```
**Solution**: Check that the native library is properly loaded and method signatures match.

### **VPN Permission Issues**
```
SecurityException: VpnService not prepared
```
**Solution**: Ensure `VpnService.prepare()` is called and user grants VPN permission.

### **Packet Forwarding Issues**
```
VPN connected but no internet access
```
**Solution**: Implement proper packet forwarding between VPN interface and rVPNSEust.

## 📚 Related Documentation

- [Quick Start Guide](../quick-start.md) - Build rVPNSEust for Android
- [Configuration Reference](../configuration.md) - TOML configuration options
- [Android Platform Guide](../platforms/android.md) - Android-specific networking details
- [C API Reference](../api/c-api.md) - Complete API documentation

---

**🎉 Your Android app is now ready to use rVPNSEust for SoftEther VPN connectivity!**
