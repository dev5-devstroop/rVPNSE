#!/bin/bash
# verify_vpn_connectivity.sh - Check if VPN connection is working properly

# Text colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=========================================${NC}"
echo -e "${BLUE}🔍 rVPNSE VPN Connection Verification Tool${NC}"
echo -e "${BLUE}=========================================${NC}"
echo ""

# Check VPN interface
echo -e "${BLUE}Checking VPN Interface:${NC}"
VPN_IF=$(ip link show | grep -E 'tun|vpnse' | head -n 1 | cut -d: -f2 | tr -d ' ')
if [ -z "$VPN_IF" ]; then
  echo -e "${RED}❌ No VPN interface found. Is the VPN connected?${NC}"
  exit 1
else
  echo -e "${GREEN}✅ Found VPN interface: $VPN_IF${NC}"
  
  # Get VPN IP address
  VPN_IP=$(ip addr show dev "$VPN_IF" | grep -oP 'inet \K[\d.]+')
  echo -e "${GREEN}✅ VPN IP address: $VPN_IP${NC}"
fi
echo ""

# Check routing
echo -e "${BLUE}Checking Routing Configuration:${NC}"
# Is the VPN the default route?
DEFAULT_ROUTE=$(ip route show default | head -n 1)
if echo "$DEFAULT_ROUTE" | grep -q "$VPN_IF"; then
  echo -e "${GREEN}✅ VPN is set as default route: $DEFAULT_ROUTE${NC}"
else
  echo -e "${RED}❌ VPN is NOT the default route: $DEFAULT_ROUTE${NC}"
  echo -e "${YELLOW}   This may cause traffic to bypass the VPN tunnel${NC}"
fi

# Check VPN subnet and gateway
VPN_NET=$(ip route show dev "$VPN_IF" | grep -v default | head -n 1)
echo -e "${BLUE}VPN Network:${NC} $VPN_NET"
VPN_OCTETS=($(echo "$VPN_IP" | tr '.' ' '))
if [ "${VPN_OCTETS[0]}" == "10" ] && [ "${VPN_OCTETS[1]}" == "21" ]; then
  echo -e "${GREEN}✅ Detected 10.21.*.* range from DHCP - This is expected${NC}"
fi

# Check for multiple default routes
DEFAULT_ROUTES=$(ip route show default | wc -l)
if [ "$DEFAULT_ROUTES" -gt 1 ]; then
  echo -e "${RED}❌ Multiple default routes detected ($DEFAULT_ROUTES)! This can cause routing conflicts:${NC}"
  ip route show default
else
  echo -e "${GREEN}✅ Single default route configuration (good)${NC}"
fi
echo ""

# Check DNS configuration
echo -e "${BLUE}Checking DNS Configuration:${NC}"
echo -e "${YELLOW}Current resolv.conf:${NC}"
cat /etc/resolv.conf
echo ""

# Test DNS resolution
echo -e "${BLUE}Testing DNS Resolution:${NC}"
echo -n "Resolving google.com... "
if host google.com >/dev/null 2>&1; then
  echo -e "${GREEN}✅ Success${NC}"
else
  echo -e "${RED}❌ Failed${NC}"
fi

echo -n "Resolving cloudflare.com... "
if host cloudflare.com >/dev/null 2>&1; then
  echo -e "${GREEN}✅ Success${NC}"
else
  echo -e "${RED}❌ Failed${NC}"
fi
echo ""

# Check for traffic leaks
echo -e "${BLUE}Checking for Potential Traffic Leaks:${NC}"
# Look for non-VPN outgoing connections
NON_VPN_CONNS=$(netstat -tn | grep ESTABLISHED | grep -v "$VPN_IP" | grep -v "127.0.0.1" | wc -l)
if [ "$NON_VPN_CONNS" -gt 0 ]; then
  echo -e "${YELLOW}⚠️ Found $NON_VPN_CONNS connections not going through the VPN:${NC}"
  netstat -tn | grep ESTABLISHED | grep -v "$VPN_IP" | grep -v "127.0.0.1" | head -n 5
else
  echo -e "${GREEN}✅ No non-VPN connections detected${NC}"
fi
echo ""

# Check actual public IP
echo -e "${BLUE}Checking Public IP Address:${NC}"
echo -e "${YELLOW}Your current public IP is:${NC}"
PUBLIC_IP=$(curl -s https://ipinfo.io/ip)
echo -e "${GREEN}$PUBLIC_IP${NC}"
echo -e "${YELLOW}(If this matches your regular IP, traffic is NOT going through the VPN!)${NC}"
echo ""

# IP leak test
echo -e "${BLUE}Running IP Leak Test:${NC}"
echo -e "${YELLOW}Testing WebRTC leak...${NC}"
echo "Please visit https://ipleak.net in your browser to check for WebRTC leaks"
echo ""

# Final status
echo -e "${BLUE}====================================${NC}"
echo -e "${BLUE}Summary:${NC}"

if echo "$DEFAULT_ROUTE" | grep -q "$VPN_IF" && [ "$DEFAULT_ROUTES" -eq 1 ] && host google.com >/dev/null 2>&1; then
  echo -e "${GREEN}✅ VPN appears to be configured correctly!${NC}"
else
  echo -e "${RED}❌ VPN configuration issues detected!${NC}"
  echo -e "${YELLOW}   Run sudo ./fix_vpn_connection.sh to fix common issues${NC}"
fi
echo -e "${BLUE}====================================${NC}"
