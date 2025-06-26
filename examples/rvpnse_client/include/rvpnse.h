/**
 * Rust VPNSE - C Header for Static Library Integration
 * 
 * This header provides C-compatible functions for integrating Rust VPNSE
 * into applications written in other languages (Swift, Kotlin, C#, etc.).
 * 
 * Usage:
 * 1. Parse and validate configuration with vpnse_parse_config()
 * 2. Create client instance with vpnse_client_new()
 * 3. Connect to server with vpnse_client_connect()
 * 4. Authenticate with vpnse_client_authenticate()
 * 5. Handle packet forwarding in your application
 * 6. Disconnect with vpnse_client_disconnect()
 * 7. Free client with vpnse_client_free()
 */

#ifndef rVPNSE_H
#define rVPNSE_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Error codes returned by Rust VPNSE functions
 */
typedef enum {
    VPNSE_SUCCESS = 0,
    VPNSE_INVALID_CONFIG = 1,
    VPNSE_CONNECTION_FAILED = 2,
    VPNSE_AUTHENTICATION_FAILED = 3,
    VPNSE_NETWORK_ERROR = 4,
    VPNSE_INVALID_PARAMETER = 5,
    VPNSE_INTERNAL_ERROR = 99
} vpnse_error_t;

/**
 * Connection status values
 */
typedef enum {
    VPNSE_DISCONNECTED = 0,
    VPNSE_CONNECTING = 1,
    VPNSE_CONNECTED = 2
} vpnse_status_t;

/**
 * Opaque VPN client handle
 */
typedef struct vpnse_client vpnse_client_t;

/**
 * Parse and validate a SoftEther VPN configuration
 * 
 * @param config_str TOML configuration string (null-terminated)
 * @param error_msg Output buffer for error messages (can be NULL)
 * @param error_msg_len Size of error message buffer
 * @return VPNSE_SUCCESS on success, error code on failure
 */
int vpnse_parse_config(const char* config_str, char* error_msg, size_t error_msg_len);

/**
 * Create a new VPN client instance
 * 
 * @param config_str TOML configuration string (null-terminated)
 * @return Opaque pointer to VPN client on success, NULL on failure
 */
vpnse_client_t* vpnse_client_new(const char* config_str);

/**
 * Connect to SoftEther VPN server
 * 
 * @param client VPN client instance from vpnse_client_new()
 * @param server Server hostname or IP address (null-terminated)
 * @param port Server port number
 * @return VPNSE_SUCCESS on success, error code on failure
 */
int vpnse_client_connect(vpnse_client_t* client, const char* server, uint16_t port);

/**
 * Authenticate with SoftEther VPN server
 * 
 * @param client VPN client instance
 * @param username Username for authentication (null-terminated)
 * @param password Password for authentication (null-terminated)
 * @return VPNSE_SUCCESS on success, error code on failure
 */
int vpnse_client_authenticate(vpnse_client_t* client, const char* username, const char* password);

/**
 * Disconnect from VPN server
 * 
 * @param client VPN client instance
 * @return VPNSE_SUCCESS on success, error code on failure
 */
int vpnse_client_disconnect(vpnse_client_t* client);

/**
 * Free VPN client instance
 * 
 * @param client VPN client instance to free
 */
void vpnse_client_free(vpnse_client_t* client);

/**
 * Get library version
 * 
 * @return Version string (caller must not free)
 */
const char* vpnse_version(void);

/**
 * Get connection status
 * 
 * @param client VPN client instance
 * @return Connection status (VPNSE_DISCONNECTED, VPNSE_CONNECTING, VPNSE_CONNECTED, or -1 for error)
 */
int vpnse_client_status(const vpnse_client_t* client);

#ifdef __cplusplus
}
#endif

#endif /* rVPNSE_H */
