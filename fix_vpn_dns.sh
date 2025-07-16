#!/bin/bash
# fix_vpn_dns.sh - Script to fix VPN DNS resolution issues
# This script focuses specifically on fixing DNS problems that occur with the VPN connection

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

echo -e "${BLUE}==========================================${NC}"
echo -e "${BLUE}  rVPNSE VPN DNS Resolution Fix Tool     ${NC}"
echo -e "${BLUE}==========================================${NC}"

# Check for VPN interface
echo -e "${YELLOW}Checking for VPN interface...${NC}"
TUN_IF=$(ip addr | grep -o "tun[0-9]*\|vpnse[0-9]*" | head -1)

if [ -z "$TUN_IF" ]; then
  echo -e "${RED}No VPN interface found. Is the VPN connected?${NC}"
  echo "Please connect the VPN first and then run this script."
  exit 1
fi

echo -e "${GREEN}Found VPN interface: ${TUN_IF}${NC}"

# Get VPN IP and gateway
VPN_IP=$(ip addr show ${TUN_IF} | grep "inet\b" | awk '{print $2}' | cut -d/ -f1)
echo -e "${GREEN}VPN IP address: ${VPN_IP}${NC}"

# Extract gateway IP
VPN_GATEWAY=$(ip route show dev ${TUN_IF} | grep -v "link" | head -1 | awk '{print $1}' | sed 's/\/.*//g')
if [ -z "$VPN_GATEWAY" ]; then
  # Try alternative method
  VPN_GATEWAY=$(ip route show | grep ${TUN_IF} | grep via | head -1 | awk '{print $3}')
fi

if [ -z "$VPN_GATEWAY" ]; then
  # Use first 3 octets of VPN IP and add .1 as a fallback
  VPN_GATEWAY=$(echo ${VPN_IP} | cut -d. -f1-3).1
  echo -e "${YELLOW}Could not detect gateway directly, using educated guess: ${VPN_GATEWAY}${NC}"
else
  echo -e "${GREEN}VPN gateway: ${VPN_GATEWAY}${NC}"
fi

echo -e "\n${BLUE}Fixing DNS resolution issues...${NC}"

# Check if systemd-resolved is in use
if systemctl is-active systemd-resolved >/dev/null 2>&1; then
  echo -e "${YELLOW}Detected systemd-resolved - configuring...${NC}"
  
  # Configure DNS for the VPN interface
  resolvectl dns ${TUN_IF} ${VPN_GATEWAY} 1.1.1.1 8.8.8.8 8.8.4.4
  
  # Set domain to ensure queries go through VPN
  resolvectl domain ${TUN_IF} ~.
  
  # Flush the DNS cache
  resolvectl flush-caches
  
  # Create a resolved conf file
  mkdir -p /etc/systemd/resolved.conf.d/
  cat > /etc/systemd/resolved.conf.d/vpn-dns.conf << EOF
[Resolve]
DNS=${VPN_GATEWAY} 1.1.1.1 8.8.8.8 8.8.4.4
DNSStubListener=yes
Cache=yes
DNSOverTLS=opportunistic
DNSSEC=allow-downgrade
EOF

  # Restart systemd-resolved
  systemctl restart systemd-resolved
  
  echo -e "${GREEN}✅ systemd-resolved configured for VPN DNS${NC}"
else
  echo -e "${YELLOW}Using direct resolv.conf configuration...${NC}"
  
  # Backup original resolv.conf if it hasn't been backed up yet
  if [ ! -f /etc/resolv.conf.vpn_backup ]; then
    cp /etc/resolv.conf /etc/resolv.conf.vpn_backup
  fi
  
  # Create new resolv.conf with improved settings
  cat > /etc/resolv.conf << EOF
# DNS Configuration for rVPNSE VPN
options timeout:1 attempts:3 rotate edns0
nameserver ${VPN_GATEWAY}
nameserver 1.1.1.1
nameserver 8.8.8.8
nameserver 8.8.4.4
search local vpn internal
EOF

  chmod 644 /etc/resolv.conf
  echo -e "${GREEN}✅ Created optimized resolv.conf for VPN DNS${NC}"
fi

# Ensure nsswitch.conf has DNS properly configured
if ! grep -q "hosts:.*dns" /etc/nsswitch.conf; then
  echo -e "${YELLOW}Adding 'dns' to nsswitch.conf hosts entry...${NC}"
  sed -i '/hosts:/s/$/ dns/' /etc/nsswitch.conf
  echo -e "${GREEN}✅ Updated nsswitch.conf${NC}"
else
  echo -e "${GREEN}✅ nsswitch.conf already properly configured${NC}"
fi

# Disable IPv6 temporarily to avoid DNS leaks
echo -e "\n${YELLOW}Temporarily disabling IPv6 to prevent DNS leaks...${NC}"
sysctl -w net.ipv6.conf.all.disable_ipv6=1
sysctl -w net.ipv6.conf.default.disable_ipv6=1
sysctl -w net.ipv6.conf.lo.disable_ipv6=1
echo -e "${GREEN}✅ IPv6 temporarily disabled${NC}"

# Fix /etc/hosts to ensure localhost resolution
echo -e "\n${YELLOW}Checking /etc/hosts file...${NC}"
if ! grep -q "127.0.0.1.*localhost" /etc/hosts; then
  echo -e "${YELLOW}Adding localhost entry to /etc/hosts...${NC}"
  echo "127.0.0.1 localhost" >> /etc/hosts
  echo -e "${GREEN}✅ Added localhost to /etc/hosts${NC}"
else
  echo -e "${GREEN}✅ /etc/hosts already properly configured${NC}"
fi

# Test DNS resolution
echo -e "\n${YELLOW}Testing DNS resolution...${NC}"
echo -n "Resolving google.com with host: "
if host google.com > /dev/null 2>&1; then
  echo -e "${GREEN}SUCCESS${NC}"
else
  echo -e "${RED}FAILED${NC}"
fi

echo -n "Resolving cloudflare.com with ping: "
if ping -c 1 -W 3 cloudflare.com > /dev/null 2>&1; then
  echo -e "${GREEN}SUCCESS${NC}"
else
  echo -e "${RED}FAILED${NC}"
fi

# Show current DNS configuration
echo -e "\n${YELLOW}Current DNS configuration:${NC}"
if command -v resolvectl &> /dev/null; then
  resolvectl status
else
  echo -e "${BLUE}Content of /etc/resolv.conf:${NC}"
  cat /etc/resolv.conf
fi

echo -e "\n${BLUE}==========================================${NC}"
echo -e "${GREEN}DNS configuration complete!${NC}"
echo -e "${BLUE}==========================================${NC}"
echo -e "If DNS issues persist, try the following:"
echo -e "1. Reconnect to the VPN"
echo -e "2. Check firewall rules that might block DNS traffic"
echo -e "3. Try 'sudo ./fix_vpn_connection.sh' for a complete fix"
echo -e "4. Manually add DNS servers to your network settings"
