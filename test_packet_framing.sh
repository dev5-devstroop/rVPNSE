#!/bin/bash
# Test script for VPN packet framing and tunnel testing

set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}===== VPN Packet Framing Test =====${NC}"
echo "This script tests the VPN connection with proper packet framing"

# 1. Build the project
echo -e "${YELLOW}Building project...${NC}"
cargo build --release

# 2. Set up environment for testing
echo -e "${YELLOW}Setting up test environment...${NC}"

# Check if we have root permissions
if [ "$(id -u)" -ne 0 ]; then
    echo -e "${RED}This script must be run as root${NC}"
    exit 1
fi

# Store the original route for restoration later
ORIG_GATEWAY=$(ip route | grep default | awk '{print $3}')
ORIG_INTERFACE=$(ip route | grep default | awk '{print $5}')
echo -e "${BLUE}Original gateway:${NC} $ORIG_GATEWAY via $ORIG_INTERFACE"

# 3. Run the VPN client test
echo -e "${YELLOW}Testing VPN connection with different IP ranges...${NC}"

# Function to clean up and restore original settings
cleanup() {
    echo -e "${YELLOW}Cleaning up and restoring original configuration...${NC}"
    # Restore original route
    ip route add default via $ORIG_GATEWAY dev $ORIG_INTERFACE
    
    # Restore DNS if modified
    if [ -f /tmp/resolv.conf.backup ]; then
        cp /tmp/resolv.conf.backup /etc/resolv.conf
        rm /tmp/resolv.conf.backup
    fi
    
    echo -e "${GREEN}Original configuration restored${NC}"
}

# Set up trap to ensure cleanup on exit
trap cleanup EXIT

# Test VPN connection with server-assigned IP ranges
test_vpn_connection() {
    local ip_range=$1
    echo -e "${BLUE}Testing VPN connection with IP range:${NC} $ip_range"
    
    # Back up DNS configuration
    cp /etc/resolv.conf /tmp/resolv.conf.backup
    
    # Create a test file to capture packet framing logs
    touch /tmp/vpn_packet_framing.log
    chmod 666 /tmp/vpn_packet_framing.log
    
    echo -e "${YELLOW}Starting VPN connection...${NC}"
    # Set environment variable to enable packet framing debug
    export RVPNSE_DEBUG_PACKET_FRAMING=1
    
    # Run the VPN connection test (replace with actual command to start your VPN)
    RUST_LOG=debug ./target/release/rvpnse --test-connection --ip-range "$ip_range" --debug-packet-framing 2>&1 | tee /tmp/vpn_connection.log &
    VPN_PID=$!
    
    echo -e "${YELLOW}Waiting for VPN connection to establish...${NC}"
    sleep 5
    
    # Test connectivity through VPN
    echo -e "${YELLOW}Testing connectivity...${NC}"
    ping -c 4 8.8.8.8 || echo -e "${RED}Ping failed${NC}"
    
    # Test DNS resolution
    echo -e "${YELLOW}Testing DNS resolution...${NC}"
    dig +short google.com || echo -e "${RED}DNS resolution failed${NC}"
    
    # Check packet framing logs
    echo -e "${YELLOW}Checking packet framing logs...${NC}"
    if grep -q "Packet framed successfully" /tmp/vpn_packet_framing.log; then
        echo -e "${GREEN}Packet framing working correctly${NC}"
    else
        echo -e "${RED}Packet framing issues detected${NC}"
    fi
    
    # Stop VPN connection
    echo -e "${YELLOW}Stopping VPN connection...${NC}"
    kill $VPN_PID || true
    sleep 2
}

# Test with different IP ranges
test_vpn_connection "10.21.0.0/16"
test_vpn_connection "10.216.48.0/24"
test_vpn_connection "10.244.0.0/16"

echo -e "${GREEN}===== VPN Packet Framing Test Complete =====${NC}"
