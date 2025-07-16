#!/bin/bash
# This script provides a step-by-step troubleshooting process for rVPNSE
# Based on analysis of SoftEtherVPN implementation

echo "=== rVPNSE Troubleshooting Tool ==="
echo "This script will help diagnose and fix common issues with rVPNSE"

# Check if running as root
if [ "$EUID" -ne 0 ]; then
  echo "⚠️ This script must be run as root to perform network configuration"
  echo "Please run with: sudo $0"
  exit 1
fi

# Step 1: Check if VPN interface exists
echo -e "\n1️⃣ Checking VPN interface..."
VPN_INTERFACE=$(ip addr | grep -o 'vpnse[0-9]*' | head -1)
if [ -z "$VPN_INTERFACE" ]; then
  echo "❌ No VPN interface found. The VPN tunnel may not be established."
  echo "    - Make sure you've started the VPN client"
  echo "    - Check for authentication errors in logs"
  exit 1
else
  echo "✅ Found VPN interface: $VPN_INTERFACE"
  echo "    Interface details:"
  ip addr show $VPN_INTERFACE
fi

# Step 2: Check routing
echo -e "\n2️⃣ Checking routing configuration..."
DEFAULT_ROUTE=$(ip route | grep default)
if echo "$DEFAULT_ROUTE" | grep -q "$VPN_INTERFACE"; then
  echo "✅ Default route is set to go through VPN"
  echo "    $DEFAULT_ROUTE"
else
  echo "❌ Default route is NOT set to go through VPN"
  echo "    Current default route: $DEFAULT_ROUTE"
  echo "    Fixing routing table..."
  
  # Get VPN gateway
  VPN_GATEWAY=$(ip addr show $VPN_INTERFACE | grep -oP 'peer \K\S+' | cut -d/ -f1)
  if [ -z "$VPN_GATEWAY" ]; then
    VPN_GATEWAY=$(ip route | grep $VPN_INTERFACE | grep -oP 'via \K\S+' | head -1)
  fi
  
  if [ -n "$VPN_GATEWAY" ]; then
    # Remove existing default route
    ip route del default
    
    # Add default route through VPN
    ip route add default via $VPN_GATEWAY dev $VPN_INTERFACE
    echo "✅ Default route fixed to use VPN"
    ip route | grep default
    
    # Add split routing
    ip route add 0.0.0.0/1 dev $VPN_INTERFACE
    ip route add 128.0.0.0/1 dev $VPN_INTERFACE
    echo "✅ Added split routing for comprehensive coverage"
  else
    echo "❌ Could not determine VPN gateway - please run fix_vpn_networking.sh"
  fi
fi

# Step 3: Check DNS
echo -e "\n3️⃣ Checking DNS configuration..."
DNS_SERVERS=$(cat /etc/resolv.conf | grep nameserver | awk '{print $2}')
echo "Current DNS servers:"
echo "$DNS_SERVERS"

# Test DNS resolution
echo "Testing DNS resolution..."
if host google.com > /dev/null 2>&1; then
  echo "✅ DNS resolution is working correctly"
else
  echo "❌ DNS resolution is NOT working"
  echo "    Fixing DNS configuration..."
  
  # Backup current resolv.conf
  cp /etc/resolv.conf /etc/resolv.conf.backup.$(date +%s)
  
  # Create new resolv.conf
  echo "# rVPNSE VPN DNS Configuration" > /etc/resolv.conf
  echo "nameserver 8.8.8.8" >> /etc/resolv.conf
  echo "nameserver 1.1.1.1" >> /etc/resolv.conf
  echo "nameserver 8.8.4.4" >> /etc/resolv.conf
  echo "options timeout:1" >> /etc/resolv.conf
  
  chmod 644 /etc/resolv.conf
  
  echo "✅ DNS configuration fixed"
  echo "Testing new DNS configuration..."
  if host google.com > /dev/null 2>&1; then
    echo "✅ DNS resolution now working correctly"
  else
    echo "❌ DNS still not working - there may be deeper networking issues"
    
    # Check for systemd-resolved
    if systemctl is-active systemd-resolved > /dev/null 2>&1; then
      echo "    systemd-resolved is active, configuring it for VPN..."
      
      mkdir -p /etc/systemd/resolved.conf.d/
      echo "[Resolve]" > /etc/systemd/resolved.conf.d/vpn-dns.conf
      echo "DNS=8.8.8.8 1.1.1.1" >> /etc/systemd/resolved.conf.d/vpn-dns.conf
      echo "DNSStubListener=yes" >> /etc/systemd/resolved.conf.d/vpn-dns.conf
      
      systemctl restart systemd-resolved
      echo "✅ systemd-resolved reconfigured for VPN"
    fi
  fi
fi

# Step 4: Check connectivity
echo -e "\n4️⃣ Checking internet connectivity through VPN..."
if ping -c 1 -W 2 8.8.8.8 > /dev/null 2>&1; then
  echo "✅ Can ping 8.8.8.8 - basic connectivity works"
else
  echo "❌ Cannot ping 8.8.8.8 - basic connectivity failing"
fi

# Step 5: Check for routing loops
echo -e "\n5️⃣ Checking for routing loops..."
VPN_SERVER=${VPN_SERVER_IP:-"62.24.65.211"}
echo "Testing connection to VPN server at $VPN_SERVER..."

ORIGINAL_GATEWAY=$(ip route | grep -v $VPN_INTERFACE | grep -oP 'default via \K\S+' | head -1)
ORIGINAL_INTERFACE=$(ip route | grep -v $VPN_INTERFACE | grep -oP 'default via .+ dev \K\S+' | head -1)

if [ -z "$ORIGINAL_GATEWAY" ] || [ -z "$ORIGINAL_INTERFACE" ]; then
  echo "⚠️ Cannot identify original gateway/interface for routing fix"
else
  echo "Adding direct route to VPN server via original gateway..."
  ip route del $VPN_SERVER 2>/dev/null
  ip route add $VPN_SERVER via $ORIGINAL_GATEWAY dev $ORIGINAL_INTERFACE
  
  if ping -c 1 -W 2 $VPN_SERVER > /dev/null 2>&1; then
    echo "✅ VPN server is reachable - routing looks good"
  else
    echo "❌ VPN server is not reachable - potential routing issue"
  fi
fi

# Step 6: Check packet flow
echo -e "\n6️⃣ Checking VPN packet flow..."
if command -v tcpdump > /dev/null 2>&1; then
  echo "Capturing brief packet sample on VPN interface..."
  timeout 3 tcpdump -i $VPN_INTERFACE -n -c 10 2>&1 | grep -v "listening"
  
  echo "Checking for encrypted VPN traffic..."
  if timeout 3 tcpdump -i $VPN_INTERFACE -n -c 10 2>&1 | grep -q "encrypted"; then
    echo "✅ Encrypted traffic detected on VPN interface"
  else
    echo "⚠️ No obvious encrypted traffic seen - this might be normal depending on traffic volume"
  fi
else
  echo "⚠️ tcpdump not available - skipping packet analysis"
fi

# Step 7: Check public IP
echo -e "\n7️⃣ Checking public IP address..."
echo "Fetching public IP (this may take a few seconds)..."
PUBLIC_IP=$(curl -s https://api.ipify.org)
if [ -n "$PUBLIC_IP" ]; then
  echo "Current public IP: $PUBLIC_IP"
  echo "⚠️ If this IP doesn't match your expected VPN exit point, traffic may not be routing through VPN"
else
  echo "❌ Could not determine public IP - connectivity issue"
fi

# Step 8: Apply recommended fixes
echo -e "\n8️⃣ Applying recommended fixes..."

# Fix RP filtering
echo "Disabling reverse path filtering for VPN..."
sysctl -w net.ipv4.conf.all.rp_filter=0
sysctl -w net.ipv4.conf.$VPN_INTERFACE.rp_filter=0

# Fix IP forwarding
echo "Enabling IP forwarding..."
sysctl -w net.ipv4.ip_forward=1

# Fix NAT
echo "Setting up NAT rules..."
iptables -t nat -A POSTROUTING -o $VPN_INTERFACE -j MASQUERADE
iptables -A FORWARD -i $VPN_INTERFACE -j ACCEPT

echo -e "\n✅ Troubleshooting and fixes completed"
echo "If issues persist, please run the full fix script: ./fix_vpn_networking.sh"
echo "You may also need to restart the VPN client to apply all changes."
echo ""
echo "For binary protocol analysis, run: ./analyze_binary_protocol.sh"
echo "To learn about SoftEtherVPN vs rVPNSE differences, see: SOFTETHER_COMPARISON.md"
