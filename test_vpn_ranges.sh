#!/bin/bash
# test_vpn_ranges.sh - Script to test VPN connectivity with different IP ranges
# This script helps test VPN connections with 10.216.48.* and 10.21.*.* IP ranges

# Exit on error
set -e

# Color definitions
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check if script is run as root
if [ "$EUID" -ne 0 ]; then
  echo -e "${RED}Please run as root${NC}"
  exit 1
fi

echo -e "${BLUE}===========================================${NC}"
echo -e "${BLUE}  rVPNSE VPN IP Range Test Tool           ${NC}"
echo -e "${BLUE}===========================================${NC}"
echo -e "${YELLOW}This script will test connectivity with both:${NC}"
echo -e "  - 10.216.48.* IP range"
echo -e "  - 10.21.*.* IP range"
echo

# Check if the VPN is connected
echo -e "${YELLOW}Checking for active VPN connection...${NC}"
ip addr | grep -q tun || {
  echo -e "${RED}No TUN interface found. Is the VPN connected?${NC}"
  echo "Please connect the VPN first and then run this script."
  exit 1
}

# Get the current VPN interface
TUN_IF=$(ip addr | grep -o "tun[0-9]*" | head -1)
echo -e "${GREEN}Found VPN interface: ${TUN_IF}${NC}"

# Get the assigned IP address
VPN_IP=$(ip addr show ${TUN_IF} | grep "inet\b" | awk '{print $2}' | cut -d/ -f1)
echo -e "${GREEN}VPN IP address: ${VPN_IP}${NC}"

# Check subnet for our ranges of interest
echo -e "\n${YELLOW}Analyzing VPN IP subnet...${NC}"
IP_PARTS=(${VPN_IP//./ })

if [[ "${IP_PARTS[0]}.${IP_PARTS[1]}" == "10.216" && "${IP_PARTS[2]}" == "48" ]]; then
  echo -e "${GREEN}✓ VPN is using the 10.216.48.* range${NC}"
  DETECTED_RANGE="10.216.48.*"
elif [[ "${IP_PARTS[0]}.${IP_PARTS[1]}" == "10.21" ]]; then
  echo -e "${GREEN}✓ VPN is using the 10.21.*.* range${NC}"
  DETECTED_RANGE="10.21.*.*"
else
  echo -e "${YELLOW}! VPN is using a different range: ${IP_PARTS[0]}.${IP_PARTS[1]}.${IP_PARTS[2]}.*${NC}"
  DETECTED_RANGE="${IP_PARTS[0]}.${IP_PARTS[1]}.${IP_PARTS[2]}.*"
fi

# Test basic connectivity
echo -e "\n${YELLOW}Testing basic connectivity...${NC}"
echo -n "Pinging 8.8.8.8: "
if ping -c 1 -W 2 8.8.8.8 > /dev/null 2>&1; then
  echo -e "${GREEN}SUCCESS${NC}"
else
  echo -e "${RED}FAILED${NC}"
fi

# Test DNS resolution
echo -e "\n${YELLOW}Testing DNS resolution...${NC}"
echo -n "Resolving google.com: "
if nslookup google.com > /dev/null 2>&1; then
  echo -e "${GREEN}SUCCESS${NC}"
else
  echo -e "${RED}FAILED${NC}"
fi

# Show routing table
echo -e "\n${YELLOW}Current routing table:${NC}"
ip route

# Show DNS configuration
echo -e "\n${YELLOW}DNS configuration:${NC}"
if command -v resolvectl &> /dev/null; then
  resolvectl status
else
  echo -e "${BLUE}Content of /etc/resolv.conf:${NC}"
  cat /etc/resolv.conf
fi

# Test for split tunneling coverage
echo -e "\n${YELLOW}Testing for split tunneling coverage...${NC}"
echo -n "Checking for 0.0.0.0/1 route: "
if ip route | grep -q "0.0.0.0/1"; then
  echo -e "${GREEN}FOUND${NC}"
else
  echo -e "${YELLOW}NOT FOUND${NC}"
fi

echo -n "Checking for 128.0.0.0/1 route: "
if ip route | grep -q "128.0.0.0/1"; then
  echo -e "${GREEN}FOUND${NC}"
else
  echo -e "${YELLOW}NOT FOUND${NC}"
fi

# Check for proper VPN exclusion
echo -e "\n${YELLOW}Checking for proper VPN subnet exclusion...${NC}"
# Extract the first three octets of the VPN IP
VPN_SUBNET="${IP_PARTS[0]}.${IP_PARTS[1]}.${IP_PARTS[2]}.0/24"
echo -n "Looking for exclusion of ${VPN_SUBNET}: "
if ip route | grep -q "${VPN_SUBNET}"; then
  echo -e "${GREEN}FOUND${NC}"
else
  echo -e "${YELLOW}NOT FOUND${NC}"
fi

echo -e "\n${BLUE}===========================================${NC}"
echo -e "${GREEN}Test completed for ${DETECTED_RANGE} range${NC}"
echo -e "${BLUE}===========================================${NC}"
