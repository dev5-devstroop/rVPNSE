/*
 * C FFI Example: Testing SoftEther SSL-VPN connection with VPN Gate
 * 
 * This example demonstrates how to use the rvpnse-rust static library
 * from C applications to connect to a VPN Gate server.
 * 
 * Compile: gcc -o test_vpngate_c examples/test_vpngate_connection.c -L./target/release -lrvpnse -lpthread -ldl -lm
 * Run: ./test_vpngate_c
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

// Include the generated C header
#include "../../include/rvpnse.h"

// VPN Gate configuration as TOML string
const char* vpngate_config = 
    "[server]\n"
    "hostname = \"public-vpn-247.opengw.net\"\n"
    "port = 443\n"
    "hub = \"VPNGATE\"\n"
    "use_ssl = true\n"
    "verify_certificate = false\n"
    "timeout = 30\n"
    "keepalive_interval = 50\n"
    "\n"
    "[auth]\n"
    "method = \"password\"\n"
    "username = \"vpn\"\n"
    "password = \"vpn\"\n"
    "\n"
    "[network]\n"
    "auto_route = false\n"
    "dns_override = false\n"
    "mtu = 1500\n"
    "\n"
    "[logging]\n"
    "level = \"info\"\n";

int main() {
    printf("Rust VPNSE C FFI - VPN Gate Connection Test\n");
    printf("==========================================\n");
    
    // Show library version
    printf("Library version: %s\n", vpnse_version());
    
    // Validate configuration first
    printf("\nValidating configuration...\n");
    char error_msg[256] = {0};
    int parse_result = vpnse_parse_config(vpngate_config, error_msg, sizeof(error_msg));
    
    if (parse_result != VPNSE_SUCCESS) {
        printf("✗ Configuration validation failed: %s\n", error_msg);
        return 1;
    }
    printf("✓ Configuration validated successfully\n");
    
    printf("Server: public-vpn-247.opengw.net:443\n");
    printf("Hub: VPNGATE\n");
    printf("Username: vpn\n");
    
    // Create VPN client
    printf("\nCreating VPN client...\n");
    vpnse_client_t* client = vpnse_client_new(vpngate_config);
    if (!client) {
        printf("✗ Failed to create VPN client\n");
        return 1;
    }
    printf("✓ VPN client created\n");
    
    // Test connection (protocol level only)
    printf("\nTesting SoftEther SSL-VPN protocol connection...\n");
    printf("Note: This is a protocol-level test only.\n");
    printf("Actual packet routing requires platform-specific implementation.\n");
    
    int result = vpnse_client_connect(client, "public-vpn-247.opengw.net", 443);
    
    if (result == VPNSE_SUCCESS) {
        printf("✓ Protocol connection successful!\n");
        printf("✓ SoftEther SSL-VPN handshake completed\n");
        
        // Check connection status
        printf("\nChecking connection status...\n");
        int status = vpnse_client_status(client);
        switch (status) {
            case VPNSE_DISCONNECTED:
                printf("Status: Disconnected\n");
                break;
            case VPNSE_CONNECTING:
                printf("Status: Connecting\n");
                break;
            case VPNSE_CONNECTED:
                printf("Status: Connected\n");
                break;
            default:
                printf("Status: Unknown (%d)\n", status);
                break;
        }
        
        // Wait a moment to maintain connection
        printf("Maintaining connection for 5 seconds...\n");
        sleep(5);
        
        // Graceful disconnect
        printf("\nDisconnecting...\n");
        if (vpnse_client_disconnect(client) == VPNSE_SUCCESS) {
            printf("✓ Disconnected successfully\n");
        }
    } else {
        printf("✗ Connection failed (error code: %d)\n", result);
        printf("This may be expected - the library provides protocol implementation only.\n");
        printf("Actual VPN functionality requires platform-specific networking integration.\n");
    }
    
    // Clean up
    vpnse_client_free(client);
    
    printf("\n==========================================\n");
    printf("Test completed. This library provides:\n");
    printf("• SoftEther SSL-VPN protocol implementation\n");
    printf("• Authentication and session management\n");
    printf("• C FFI interface for integration\n");
    printf("\n");
    printf("For full VPN functionality, integrate with:\n");
    printf("• TUN/TAP interface creation\n");
    printf("• Packet routing and forwarding\n");
    printf("• DNS configuration\n");
    printf("• Platform-specific networking\n");
    
    return 0;
}
