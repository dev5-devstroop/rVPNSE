#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// Test the IP detection function
int main() {
    char buffer[256];
    
    printf("Testing IP detection...\n");
    
    // Try the same command the C library uses
    FILE* fp = popen("curl -s --max-time 10 https://api.ipify.org 2>/dev/null", "r");
    if (fp && fgets(buffer, sizeof(buffer), fp)) {
        // Clean up response
        char* newline = strchr(buffer, '\n');
        if (newline) *newline = '\0';
        
        printf("Raw response: '%s'\n", buffer);
        printf("Length: %zu\n", strlen(buffer));
        
        // Check if it contains a dot (IPv4)
        if (strchr(buffer, '.')) {
            printf("✅ Valid IPv4 detected: %s\n", buffer);
        } else {
            printf("❌ Invalid response\n");
        }
        
        pclose(fp);
    } else {
        printf("❌ Command failed\n");
        if (fp) pclose(fp);
    }
    
    return 0;
}
