#!/bin/bash
# detect_vpn_dhcp.sh - Detect and configure DHCP-assigned VPN IP ranges

# Colorful output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=========================================${NC}"
echo -e "${BLUE}🔍 rVPNSE VPN DHCP Detection Tool${NC}"
echo -e "${BLUE}=========================================${NC}"
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then
  echo -e "${RED}This script must be run as root (use sudo)${NC}"
  exit 1
fi

# Detect VPN interface
VPN_IF=$(ip link show | grep -E 'tun|vpnse' | head -n 1 | cut -d: -f2 | tr -d ' ')
if [ -z "$VPN_IF" ]; then
  echo -e "${RED}❌ No VPN interface found. Is the VPN connected?${NC}"
  exit 1
else
  echo -e "${GREEN}✅ Found VPN interface: $VPN_IF${NC}"
fi

# Get IP address assigned to VPN interface
VPN_IP=$(ip addr show dev "$VPN_IF" | grep -oP 'inet \K[\d.]+')
if [ -z "$VPN_IP" ]; then
  echo -e "${RED}❌ No IP address found on $VPN_IF${NC}"
  exit 1
fi

echo -e "${GREEN}✅ VPN IP address: $VPN_IP${NC}"

# Extract network details
IFS='.' read -r -a IP_PARTS <<< "$VPN_IP"
echo -e "${BLUE}IP Analysis:${NC}"
echo -e "  First octet: ${YELLOW}${IP_PARTS[0]}${NC}"
echo -e "  Second octet: ${YELLOW}${IP_PARTS[1]}${NC}"
echo -e "  Network class: ${YELLOW}$(
  if [ "${IP_PARTS[0]}" -lt 128 ]; then
    echo "Class A"
  elif [ "${IP_PARTS[0]}" -lt 192 ]; then
    echo "Class B"
  elif [ "${IP_PARTS[0]}" -lt 224 ]; then
    echo "Class C"
  else
    echo "Class D or E"
  fi
)${NC}"

# Determine network type
echo -e "${BLUE}Network type:${NC}"
if [ "${IP_PARTS[0]}" -eq 10 ]; then
  echo -e "  ${GREEN}Private network (10.0.0.0/8)${NC}"
  if [ "${IP_PARTS[1]}" -eq 21 ]; then
    echo -e "  ${GREEN}✅ Detected expected 10.21.*.* network range${NC}"
    echo -e "  ${GREEN}✅ This is the DHCP-assigned range we're looking for!${NC}"
  fi
elif [ "${IP_PARTS[0]}" -eq 172 ] && [ "${IP_PARTS[1]}" -ge 16 ] && [ "${IP_PARTS[1]}" -le 31 ]; then
  echo -e "  ${GREEN}Private network (172.16.0.0/12)${NC}"
elif [ "${IP_PARTS[0]}" -eq 192 ] && [ "${IP_PARTS[1]}" -eq 168 ]; then
  echo -e "  ${GREEN}Private network (192.168.0.0/16)${NC}"
elif [ "${IP_PARTS[0]}" -ge 100 ] && [ "${IP_PARTS[0]}" -le 127 ]; then
  echo -e "  ${GREEN}VPN provider range (${IP_PARTS[0]}.*.*.*)${NC}"
else
  echo -e "  ${YELLOW}Other network type${NC}"
fi

# Get gateway information
VPN_GW=$(ip route show dev "$VPN_IF" | grep -oP 'via \K[\d.]+')
if [ -z "$VPN_GW" ]; then
  # If no 'via' found, determine gateway from interface route
  VPN_NET=$(ip route show dev "$VPN_IF" | grep -v 'default' | grep -oP '[\d.]+/[\d]+' | head -n 1)
  if [ -n "$VPN_NET" ]; then
    # Extract gateway from network
    VPN_GW=$(echo "$VPN_NET" | cut -d/ -f1 | sed -r 's/([0-9]+\.[0-9]+\.[0-9]+)\.[0-9]+/\1.1/')
    echo -e "${YELLOW}Inferred gateway: $VPN_GW${NC}"
  else
    # Fallback - use first address in network
    VPN_GW="${IP_PARTS[0]}.${IP_PARTS[1]}.${IP_PARTS[2]}.1"
    echo -e "${YELLOW}Fallback gateway: $VPN_GW${NC}"
  fi
else
  echo -e "${GREEN}Explicit gateway: $VPN_GW${NC}"
fi

# Get route information
echo -e "\n${BLUE}Route information:${NC}"
ip route show dev "$VPN_IF"

# Get default route
DEFAULT_ROUTE=$(ip route show default | head -n 1)
echo -e "\n${BLUE}Current default route:${NC}"
echo "$DEFAULT_ROUTE"

# Check for DNS settings
echo -e "\n${BLUE}Current DNS configuration:${NC}"
cat /etc/resolv.conf

echo -e "\n${BLUE}=========================================${NC}"
echo -e "${BLUE}DHCP-assigned VPN configuration:${NC}"
echo -e "VPN Interface: ${GREEN}$VPN_IF${NC}"
echo -e "IP Address: ${GREEN}$VPN_IP${NC}"
echo -e "Gateway: ${GREEN}$VPN_GW${NC}"
echo -e "Network: ${GREEN}$VPN_NET${NC}"
echo -e "${BLUE}=========================================${NC}"

# Write the configuration for future use
echo -e "\n${BLUE}Saving detected configuration...${NC}"
cat > /tmp/vpn_dhcp_config.env << EOF
VPN_INTERFACE=$VPN_IF
VPN_IP=$VPN_IP
VPN_GW=$VPN_GW
VPN_NET=$VPN_NET
EOF

cp /tmp/vpn_dhcp_config.env ./vpn_dhcp_config.env
chmod 644 ./vpn_dhcp_config.env
echo -e "${GREEN}Configuration saved to vpn_dhcp_config.env${NC}"
echo -e "You can use this file as input for VPN configuration scripts"
