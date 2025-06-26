# 🔧 Native Integration Guide

Complete integration guides for native development with rVPNSE. This covers direct C FFI usage and Rust library integration for maximum performance and control.

## 📋 Platform Support

| Platform | Architecture | C API | Rust API | Status |
|----------|-------------|-------|----------|--------|
| **Linux** | x86_64, ARM64 | ✅ | ✅ | Production Ready |
| **Windows** | x86_64, ARM64 | ✅ | ✅ | Production Ready |
| **macOS** | x86_64, ARM64 (Apple Silicon) | ✅ | ✅ | Production Ready |
| **Android** | ARM64, ARMv7 | ✅ | ✅ | Production Ready |
| **iOS** | ARM64 | ✅ | ✅ | Production Ready |

## 🚀 Quick Start

### C/C++ Library
```bash
# Download precompiled library
curl -L -o librvpnse.so \
  https://github.com/devstroop/rvpnse/releases/latest/download/librvpnse-linux-x64.so

# Download header file
curl -L -o rvpnse.h \
  https://github.com/devstroop/rvpnse/releases/latest/download/rvpnse.h
```

### Rust Crate
```bash
# Add to Cargo.toml
cargo add rvpnse
```

## 📚 API Documentation

- [⚙️ **C/C++ Integration**](cpp.md) - Complete C FFI guide with examples
- [🦀 **Rust Integration**](rust.md) - Native Rust library usage
- [📋 **C API Reference**](../04-api/c-ffi.md) - Detailed API documentation
- [🔗 **FFI Bindings**](ffi-bindings.md) - Creating bindings for other languages

## 🎯 Key Native Features

### **Performance**
- Zero-copy data operations where possible
- Direct memory management control
- Minimal overhead over raw sockets
- Optimized for high-throughput scenarios

### **Flexibility**
- Complete control over connection lifecycle
- Custom certificate validation callbacks
- Pluggable authentication mechanisms
- Fine-grained error handling

### **Platform Integration**
- Native OS credential storage integration
- Platform-specific networking optimizations
- System service/daemon support
- Low-level network interface access

### **Memory Safety**
- RAII patterns in C++ wrapper
- Rust ownership system prevents memory leaks
- Clear resource management guidelines
- Comprehensive cleanup functions

## 🔧 Architecture Overview

### **Library Structure**
```
rVPNSE Native Library
├── Core Engine (Rust)
│   ├── Protocol Implementation
│   ├── Crypto Operations
│   ├── Network Stack
│   └── Connection Management
├── C FFI Layer
│   ├── Type Conversions
│   ├── Error Handling
│   ├── Memory Management
│   └── Callback Interface
└── Platform Adapters
    ├── TUN/TAP Interface
    ├── System Integration
    └── Security Context
```

### **Threading Model**
```c
// Main thread - Application logic
RvpnseClient* client = rvpnse_client_new(config);

// Background thread - Network I/O (handled internally)
// - Connection management
// - Packet processing
// - Keep-alive handling

// Callback thread - Event notifications
void on_state_changed(RvpnseConnectionState state, void* userdata) {
    // Called from internal callback thread
    // Must be thread-safe
}
```

## 🛡️ Security Architecture

### **Memory Safety**
```c
// Secure memory handling
void secure_zero_memory(void* ptr, size_t size) {
    volatile unsigned char* p = ptr;
    while (size--) {
        *p++ = 0;
    }
}

// Always clear sensitive data
char password[256];
// ... use password ...
secure_zero_memory(password, sizeof(password));
```

### **Certificate Validation**
```c
// Custom certificate validation callback
RvpnseResult validate_certificate(const char* cert_pem, 
                                 size_t cert_len,
                                 void* userdata) {
    // Parse certificate
    X509* cert = PEM_read_bio_X509(bio, NULL, NULL, NULL);
    if (!cert) {
        return RVPNSE_ERROR_INVALID_CERT;
    }
    
    // Check expiration
    if (X509_cmp_current_time(X509_get_notAfter(cert)) <= 0) {
        X509_free(cert);
        return RVPNSE_ERROR_CERT_EXPIRED;
    }
    
    // Check against known good certificates
    if (!is_trusted_certificate(cert)) {
        X509_free(cert);
        return RVPNSE_ERROR_CERT_UNTRUSTED;
    }
    
    X509_free(cert);
    return RVPNSE_SUCCESS;
}

// Set validation callback
rvpnse_client_set_cert_validator(client, validate_certificate, userdata);
```

## 📊 Performance Optimization

### **Zero-Copy Operations**
```c
// Use provided buffers when possible
typedef struct {
    uint8_t* data;
    size_t length;
    size_t capacity;
} RvpnseBuffer;

// Zero-copy packet handling
RvpnseResult process_packet(RvpnseClient* client, 
                           const RvpnseBuffer* input,
                           RvpnseBuffer* output) {
    // Process packet in-place when possible
    // Avoid unnecessary memory allocations
    return rvpnse_client_process_packet(client, input, output);
}
```

### **Batch Operations**
```c
// Process multiple packets in one call
typedef struct {
    RvpnseBuffer* packets;
    size_t count;
} RvpnsePacketBatch;

RvpnseResult rvpnse_client_send_batch(RvpnseClient* client,
                                     const RvpnsePacketBatch* batch) {
    // Optimized batch processing
    // Reduces system call overhead
    // Better cache locality
}
```

### **Memory Pool Management**
```c
// Pre-allocate packet buffers
typedef struct {
    RvpnseBuffer* buffers;
    size_t count;
    size_t next_free;
} RvpnseMemoryPool;

RvpnseMemoryPool* pool = rvpnse_memory_pool_create(1024, 2048); // 1024 buffers, 2KB each

// Get buffer from pool
RvpnseBuffer* buffer = rvpnse_memory_pool_get(pool);
// ... use buffer ...
rvpnse_memory_pool_return(pool, buffer);

rvpnse_memory_pool_destroy(pool);
```

## 🔄 Asynchronous Programming

### **Event-Driven Model**
```c
// Callback structure
typedef struct {
    void (*on_connected)(void* userdata);
    void (*on_disconnected)(void* userdata);
    void (*on_error)(RvpnseError error, void* userdata);
    void (*on_data_received)(const uint8_t* data, size_t len, void* userdata);
    void (*on_stats_updated)(const RvpnseStats* stats, void* userdata);
} RvpnseCallbacks;

// Application context
typedef struct {
    bool running;
    pthread_mutex_t mutex;
    pthread_cond_t condition;
} AppContext;

// Event handlers
void on_connected(void* userdata) {
    AppContext* ctx = (AppContext*)userdata;
    pthread_mutex_lock(&ctx->mutex);
    printf("✅ VPN Connected!\n");
    pthread_cond_signal(&ctx->condition);
    pthread_mutex_unlock(&ctx->mutex);
}

void on_error(RvpnseError error, void* userdata) {
    AppContext* ctx = (AppContext*)userdata;
    pthread_mutex_lock(&ctx->mutex);
    printf("❌ VPN Error: %s\n", rvpnse_error_string(error));
    ctx->running = false;
    pthread_cond_signal(&ctx->condition);
    pthread_mutex_unlock(&ctx->mutex);
}

// Main application loop
int main() {
    AppContext ctx = {.running = true};
    pthread_mutex_init(&ctx.mutex, NULL);
    pthread_cond_init(&ctx.condition, NULL);
    
    RvpnseCallbacks callbacks = {
        .on_connected = on_connected,
        .on_error = on_error,
        // ... other callbacks ...
    };
    
    RvpnseClient* client = rvpnse_client_new(config);
    rvpnse_client_set_callbacks(client, &callbacks, &ctx);
    
    // Start connection (non-blocking)
    rvpnse_client_connect_async(client);
    
    // Wait for events
    pthread_mutex_lock(&ctx.mutex);
    while (ctx.running) {
        pthread_cond_wait(&ctx.condition, &ctx.mutex);
    }
    pthread_mutex_unlock(&ctx.mutex);
    
    // Cleanup
    rvpnse_client_free(client);
    pthread_mutex_destroy(&ctx.mutex);
    pthread_cond_destroy(&ctx.condition);
    
    return 0;
}
```

### **Thread-Safe Operations**
```c
// Thread-safe client operations
typedef struct {
    RvpnseClient* client;
    pthread_mutex_t mutex;
    pthread_t worker_thread;
    bool stop_requested;
} ThreadSafeClient;

// Worker thread function
void* worker_thread(void* arg) {
    ThreadSafeClient* tsc = (ThreadSafeClient*)arg;
    
    while (!tsc->stop_requested) {
        pthread_mutex_lock(&tsc->mutex);
        
        // Process pending operations
        rvpnse_client_process_events(tsc->client, 100); // 100ms timeout
        
        pthread_mutex_unlock(&tsc->mutex);
        
        usleep(10000); // 10ms sleep
    }
    
    return NULL;
}

// Thread-safe connection function
RvpnseResult threadsafe_connect(ThreadSafeClient* tsc) {
    pthread_mutex_lock(&tsc->mutex);
    RvpnseResult result = rvpnse_client_connect(tsc->client);
    pthread_mutex_unlock(&tsc->mutex);
    return result;
}
```

## 🧪 Testing and Debugging

### **Unit Testing Framework**
```c
// Test framework
#include <assert.h>
#include <stdio.h>

typedef struct {
    const char* name;
    void (*test_func)(void);
} TestCase;

void test_basic_connection() {
    printf("Testing basic connection...\n");
    
    RvpnseConfig* config = create_test_config();
    RvpnseClient* client = rvpnse_client_new(config);
    
    assert(client != NULL);
    
    RvpnseResult result = rvpnse_client_connect(client);
    assert(result == RVPNSE_SUCCESS);
    
    RvpnseConnectionState state = rvpnse_client_get_state(client);
    assert(state == RVPNSE_STATE_CONNECTED);
    
    rvpnse_client_disconnect(client);
    rvpnse_client_free(client);
    rvpnse_config_free(config);
    
    printf("✅ Basic connection test passed\n");
}

void test_error_handling() {
    printf("Testing error handling...\n");
    
    // Test with invalid config
    RvpnseConfig* config = rvpnse_config_new();
    rvpnse_config_set_server_host(config, "invalid.server.com");
    rvpnse_config_set_server_port(config, 1); // Invalid port
    
    RvpnseClient* client = rvpnse_client_new(config);
    RvpnseResult result = rvpnse_client_connect(client);
    
    assert(result != RVPNSE_SUCCESS);
    assert(rvpnse_client_get_last_error(client) != RVPNSE_ERROR_NONE);
    
    rvpnse_client_free(client);
    rvpnse_config_free(config);
    
    printf("✅ Error handling test passed\n");
}

void test_memory_management() {
    printf("Testing memory management...\n");
    
    // Create and destroy multiple clients
    for (int i = 0; i < 100; i++) {
        RvpnseConfig* config = create_test_config();
        RvpnseClient* client = rvpnse_client_new(config);
        
        assert(client != NULL);
        
        rvpnse_client_free(client);
        rvpnse_config_free(config);
    }
    
    printf("✅ Memory management test passed\n");
}

// Test runner
TestCase tests[] = {
    {"Basic Connection", test_basic_connection},
    {"Error Handling", test_error_handling},
    {"Memory Management", test_memory_management},
    {NULL, NULL}
};

int main() {
    printf("Running rVPNSE Native Tests\n");
    printf("============================\n");
    
    rvpnse_init();
    
    for (int i = 0; tests[i].name != NULL; i++) {
        printf("\n%s:\n", tests[i].name);
        tests[i].test_func();
    }
    
    rvpnse_cleanup();
    
    printf("\n✅ All tests passed!\n");
    return 0;
}
```

### **Memory Leak Detection**
```c
// Custom allocator for debugging
static size_t total_allocated = 0;
static size_t allocation_count = 0;

void* debug_malloc(size_t size) {
    void* ptr = malloc(size + sizeof(size_t));
    if (ptr) {
        *(size_t*)ptr = size;
        total_allocated += size;
        allocation_count++;
        return (char*)ptr + sizeof(size_t);
    }
    return NULL;
}

void debug_free(void* ptr) {
    if (ptr) {
        char* real_ptr = (char*)ptr - sizeof(size_t);
        size_t size = *(size_t*)real_ptr;
        total_allocated -= size;
        allocation_count--;
        free(real_ptr);
    }
}

// Set custom allocator
rvpnse_set_allocator(debug_malloc, debug_free, realloc);

// Check for leaks at exit
void check_memory_leaks() {
    if (total_allocated > 0) {
        printf("⚠️ Memory leak detected: %zu bytes in %zu allocations\n", 
               total_allocated, allocation_count);
    } else {
        printf("✅ No memory leaks detected\n");
    }
}

atexit(check_memory_leaks);
```

### **Performance Profiling**
```c
#include <sys/time.h>

typedef struct {
    struct timeval start;
    struct timeval end;
    const char* operation;
} ProfileTimer;

void profile_start(ProfileTimer* timer, const char* operation) {
    timer->operation = operation;
    gettimeofday(&timer->start, NULL);
}

void profile_end(ProfileTimer* timer) {
    gettimeofday(&timer->end, NULL);
    
    long seconds = timer->end.tv_sec - timer->start.tv_sec;
    long microseconds = timer->end.tv_usec - timer->start.tv_usec;
    double elapsed = seconds + microseconds * 1e-6;
    
    printf("⏱️ %s took %.3f ms\n", timer->operation, elapsed * 1000);
}

// Usage example
void benchmark_connection() {
    ProfileTimer timer;
    
    profile_start(&timer, "Connection establishment");
    
    RvpnseClient* client = rvpnse_client_new(config);
    rvpnse_client_connect(client);
    
    profile_end(&timer);
    
    rvpnse_client_disconnect(client);
    rvpnse_client_free(client);
}
```

## 🚀 Production Deployment

### **System Service Integration**

#### Linux Systemd
```c
// systemd-notify integration
#include <systemd/sd-daemon.h>

void notify_systemd_ready() {
    sd_notify(0, "READY=1");
}

void notify_systemd_status(const char* status) {
    char msg[256];
    snprintf(msg, sizeof(msg), "STATUS=%s", status);
    sd_notify(0, msg);
}

void notify_systemd_watchdog() {
    sd_notify(0, "WATCHDOG=1");
}

// Main daemon loop
int main() {
    // ... initialization ...
    
    notify_systemd_status("Connecting to VPN...");
    rvpnse_client_connect(client);
    
    notify_systemd_status("VPN Connected");
    notify_systemd_ready();
    
    // Main loop
    while (running) {
        notify_systemd_watchdog();
        // ... process events ...
        sleep(1);
    }
    
    notify_systemd_status("Shutting down...");
    return 0;
}
```

#### Windows Service
```c
#include <windows.h>
#include <winsvc.h>

SERVICE_STATUS g_ServiceStatus = {0};
SERVICE_STATUS_HANDLE g_StatusHandle = NULL;
HANDLE g_ServiceStopEvent = INVALID_HANDLE_VALUE;

VOID WINAPI ServiceMain(DWORD argc, LPTSTR* argv) {
    g_StatusHandle = RegisterServiceCtrlHandler(L"RvpnseService", ServiceCtrlHandler);
    
    // Set service status to starting
    g_ServiceStatus.dwServiceType = SERVICE_WIN32_OWN_PROCESS;
    g_ServiceStatus.dwCurrentState = SERVICE_START_PENDING;
    SetServiceStatus(g_StatusHandle, &g_ServiceStatus);
    
    // Create stop event
    g_ServiceStopEvent = CreateEvent(NULL, TRUE, FALSE, NULL);
    
    // Initialize VPN
    RvpnseClient* client = rvpnse_client_new(config);
    rvpnse_client_connect(client);
    
    // Set service status to running
    g_ServiceStatus.dwCurrentState = SERVICE_RUNNING;
    SetServiceStatus(g_StatusHandle, &g_ServiceStatus);
    
    // Main service loop
    while (WaitForSingleObject(g_ServiceStopEvent, 1000) != WAIT_OBJECT_0) {
        // Process VPN events
        rvpnse_client_process_events(client, 100);
    }
    
    // Cleanup
    rvpnse_client_disconnect(client);
    rvpnse_client_free(client);
    
    // Set service status to stopped
    g_ServiceStatus.dwCurrentState = SERVICE_STOPPED;
    SetServiceStatus(g_StatusHandle, &g_ServiceStatus);
}

VOID WINAPI ServiceCtrlHandler(DWORD dwCtrl) {
    switch (dwCtrl) {
        case SERVICE_CONTROL_STOP:
            g_ServiceStatus.dwCurrentState = SERVICE_STOP_PENDING;
            SetServiceStatus(g_StatusHandle, &g_ServiceStatus);
            SetEvent(g_ServiceStopEvent);
            break;
    }
}
```

### **Cross-Platform Build System**

#### CMake Configuration
```cmake
# CMakeLists.txt
cmake_minimum_required(VERSION 3.16)
project(rvpnse_app)

set(CMAKE_C_STANDARD 11)
set(CMAKE_CXX_STANDARD 17)

# Find rVPNSE library
find_library(RVPNSE_LIBRARY 
    NAMES rvpnse librvpnse
    PATHS /usr/local/lib /opt/rvpnse/lib
)

find_path(RVPNSE_INCLUDE_DIR
    NAMES rvpnse.h
    PATHS /usr/local/include /opt/rvpnse/include
)

if(NOT RVPNSE_LIBRARY OR NOT RVPNSE_INCLUDE_DIR)
    message(FATAL_ERROR "rVPNSE library not found")
endif()

# Platform-specific settings
if(WIN32)
    add_definitions(-DWIN32_LEAN_AND_MEAN)
    set(PLATFORM_LIBS ws2_32 iphlpapi)
elseif(APPLE)
    set(PLATFORM_LIBS "-framework CoreFoundation" "-framework SystemConfiguration")
else()
    set(PLATFORM_LIBS pthread dl)
endif()

# Create executable
add_executable(rvpnse_app
    src/main.c
    src/vpn_manager.c
    src/config_loader.c
)

target_include_directories(rvpnse_app PRIVATE ${RVPNSE_INCLUDE_DIR})
target_link_libraries(rvpnse_app ${RVPNSE_LIBRARY} ${PLATFORM_LIBS})

# Install rules
install(TARGETS rvpnse_app DESTINATION bin)
install(FILES config/example.toml DESTINATION etc/rvpnse)
```

#### Makefile (Alternative)
```makefile
# Makefile
CC = gcc
CFLAGS = -Wall -Wextra -std=c11 -O2
INCLUDES = -I/usr/local/include
LIBS = -lrvpnse -lpthread

# Platform detection
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Linux)
    LIBS += -ldl
endif
ifeq ($(UNAME_S),Darwin)
    LIBS += -framework CoreFoundation -framework SystemConfiguration
endif

SRCDIR = src
SOURCES = $(wildcard $(SRCDIR)/*.c)
OBJECTS = $(SOURCES:.c=.o)
TARGET = rvpnse_app

.PHONY: all clean install

all: $(TARGET)

$(TARGET): $(OBJECTS)
	$(CC) $(OBJECTS) -o $@ $(LIBS)

%.o: %.c
	$(CC) $(CFLAGS) $(INCLUDES) -c $< -o $@

clean:
	rm -f $(OBJECTS) $(TARGET)

install: $(TARGET)
	install -D $(TARGET) /usr/local/bin/$(TARGET)
	install -D config/example.toml /etc/rvpnse/config.toml

test: $(TARGET)
	./$(TARGET) --test
```

## 📚 Language Bindings

### **Creating Language Bindings**
```c
// Generic FFI wrapper for language bindings
typedef struct {
    void* language_context;
    void (*callback)(void* context, int event_type, void* data);
} LanguageBinding;

// Generic callback wrapper
void language_binding_callback(RvpnseEvent event, void* userdata) {
    LanguageBinding* binding = (LanguageBinding*)userdata;
    if (binding && binding->callback) {
        binding->callback(binding->language_context, event.type, event.data);
    }
}

// Language-agnostic client wrapper
typedef struct {
    RvpnseClient* native_client;
    LanguageBinding* binding;
} LanguageClient;

LanguageClient* language_client_new(void* language_context,
                                   void (*callback)(void*, int, void*)) {
    LanguageClient* client = malloc(sizeof(LanguageClient));
    
    client->binding = malloc(sizeof(LanguageBinding));
    client->binding->language_context = language_context;
    client->binding->callback = callback;
    
    client->native_client = rvpnse_client_new(config);
    rvpnse_client_set_event_callback(client->native_client,
                                    language_binding_callback,
                                    client->binding);
    
    return client;
}
```

## 🆘 Troubleshooting

### **Common Issues**

| Issue | Symptoms | Solution |
|-------|----------|----------|
| **Library not found** | Link error: "undefined reference" | Check library path and -L flags |
| **Header not found** | Compile error: "rvpnse.h: No such file" | Check include path and -I flags |
| **Permission denied** | Runtime error on connection | Run with appropriate privileges |
| **Segmentation fault** | Crash during operation | Check pointer validity and memory management |
| **Memory leak** | Increasing memory usage | Ensure all rvpnse_*_free() calls are made |

### **Debug Build Configuration**
```c
// Debug configuration
#ifdef DEBUG
#define DBG_PRINT(fmt, ...) printf("[DEBUG] " fmt "\n", ##__VA_ARGS__)
#else
#define DBG_PRINT(fmt, ...)
#endif

// Enable debug logging
RvpnseConfig* config = rvpnse_config_new();
rvpnse_config_set_log_level(config, RVPNSE_LOG_DEBUG);
rvpnse_config_set_log_output(config, RVPNSE_LOG_CONSOLE);
```

## 📚 Additional Resources

- [C API Reference Documentation](../04-api/c-ffi.md)
- [Rust Library Documentation](rust.md)
- [Example Projects Repository](https://github.com/devstroop/rvpnse-examples)
- [Performance Benchmarks](../../benchmark-reports/)

## 🎯 Next Steps

1. **Choose your language** (C/C++ or Rust)
2. **Download the appropriate library** for your platform
3. **Set up your build system** using our CMake/Makefile examples
4. **Implement basic connection logic** using our examples
5. **Add error handling and logging** for production robustness
6. **Profile and optimize** your implementation

**Need help?** Check our [Native development troubleshooting](../../07-troubleshooting/native.md) or [ask questions](https://github.com/devstroop/rvpnse/discussions).
