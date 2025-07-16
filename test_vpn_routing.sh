# VPN Routing Test Script
# This script tests whether the VPN tunnel is correctly routing traffic

# Output color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}=== VPN Traffic Routing Test ===${NC}"

# Get original IP (without VPN)
echo -e "\n${YELLOW}Step 1: Checking your current public IP${NC}"
CURRENT_IP=$(curl -s https://api.ipify.org)
echo -e "Current public IP: ${GREEN}$CURRENT_IP${NC}"

# Check routing table
echo -e "\n${YELLOW}Step 2: Checking routing table${NC}"
echo "Default routes:"
ip route show | grep default
echo -e "\nVPN routes:"
ip route show | grep vpnse

# Check if VPN interface exists and is up
echo -e "\n${YELLOW}Step 3: Checking VPN interface status${NC}"
if ip link show vpnse0 &>/dev/null; then
    INTERFACE_STATUS=$(ip link show vpnse0 | grep -o "state [A-Z]*" | cut -d ' ' -f 2)
    if [ "$INTERFACE_STATUS" == "UP" ]; then
        echo -e "VPN interface: ${GREEN}UP${NC}"
    else
        echo -e "VPN interface: ${RED}$INTERFACE_STATUS${NC} (should be UP)"
    fi
    
    echo "Interface details:"
    ip addr show vpnse0
else
    echo -e "${RED}VPN interface vpnse0 not found${NC}"
fi

# Test DNS resolution
echo -e "\n${YELLOW}Step 4: Testing DNS resolution${NC}"
echo "Resolving google.com:"
host google.com

# Test traceroute to see traffic path
echo -e "\n${YELLOW}Step 5: Testing traffic routing (traceroute)${NC}"
if command -v traceroute &>/dev/null; then
    echo "First few hops to google.com:"
    traceroute -m 5 -n google.com
else
    echo -e "${RED}traceroute command not found${NC}"
fi

# Check for common routing issues
echo -e "\n${YELLOW}Step 6: Checking for common routing issues${NC}"

# Check reverse path filtering
RP_FILTER=$(sysctl -n net.ipv4.conf.all.rp_filter)
if [ "$RP_FILTER" -eq 1 ]; then
    echo -e "${RED}Reverse path filtering is enabled (value=$RP_FILTER)${NC}"
    echo "This can block VPN traffic. Consider running:"
    echo "sudo sysctl -w net.ipv4.conf.all.rp_filter=0"
else
    echo -e "Reverse path filtering: ${GREEN}Disabled${NC}"
fi

# Check IP forwarding
IP_FORWARD=$(sysctl -n net.ipv4.ip_forward)
if [ "$IP_FORWARD" -eq 0 ]; then
    echo -e "${RED}IP forwarding is disabled${NC}"
    echo "This prevents VPN traffic routing. Run:"
    echo "sudo sysctl -w net.ipv4.ip_forward=1"
else
    echo -e "IP forwarding: ${GREEN}Enabled${NC}"
fi

# Check for VPN server route
VPN_SERVER="62.24.65.211" # Update with your actual VPN server IP
if ip route show | grep -q "$VPN_SERVER"; then
    echo -e "VPN server route: ${GREEN}Present${NC}"
    ip route show | grep "$VPN_SERVER"
else
    echo -e "${RED}No explicit route to VPN server${NC}"
    echo "This can cause routing loops. Add route with:"
    echo "sudo ip route add $VPN_SERVER via <your_gateway> dev <your_interface>"
fi

echo -e "\n${YELLOW}Final test: Checking current public IP again${NC}"
TEST_IP=$(curl -s https://api.ipify.org)
echo -e "Public IP after tests: ${GREEN}$TEST_IP${NC}"

if [ "$CURRENT_IP" != "$TEST_IP" ]; then
    echo -e "\n${GREEN}IP has changed - VPN routing appears to be working!${NC}"
else
    echo -e "\n${RED}IP has not changed - VPN traffic routing is not working.${NC}"
    echo "Try running the fix_vpn_routing.sh script."
fi
