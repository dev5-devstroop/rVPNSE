#!/bin/bash
# fix_vpn_connection.sh - Comprehensive script to fix VPN routing and DNS issues
# This script addresses both traffic routing and DNS resolution problems

set -e # Exit on error

# Verify we're running as root
if [ "$EUID" -ne 0 ]; then
  echo "🛑 This script must be run as root (use sudo)"
  exit 1
fi

# Get the VPN interface name (assuming it starts with tun or vpnse)
VPN_IF=$(ip link show | grep -E 'tun|vpnse' | head -n 1 | cut -d: -f2 | tr -d ' ')
if [ -z "$VPN_IF" ]; then
  echo "❌ No VPN interface found. Is the VPN connected?"
  exit 1
fi

echo "🔍 Found VPN interface: $VPN_IF"

# Get VPN IP address
VPN_IP=$(ip addr show dev "$VPN_IF" | grep -oP 'inet \K[\d.]+')
if [ -z "$VPN_IP" ]; then
  echo "❌ No IP address found on $VPN_IF"
  exit 1
fi

echo "🔍 VPN IP address: $VPN_IP"

# Get VPN gateway (remote IP)
VPN_GW=$(ip route show dev "$VPN_IF" | grep -oP 'via \K[\d.]+')
if [ -z "$VPN_GW" ]; then
  # If no 'via' found, try to determine gateway from interface route
  VPN_NET=$(ip route show dev "$VPN_IF" | grep -v 'default' | grep -oP '[\d.]+/[\d]+' | head -n 1)
  if [ -n "$VPN_NET" ]; then
    # Use the first host in the network as gateway
    VPN_GW=$(echo "$VPN_NET" | cut -d/ -f1 | sed -r 's/([0-9]+\.[0-9]+\.[0-9]+)\.[0-9]+/\1.1/')
  fi
fi

if [ -z "$VPN_GW" ]; then
  echo "⚠️ Could not determine VPN gateway. Using the first hop in the VPN network."
  # Try to get the network part of the VPN IP and use .1 as gateway
  VPN_GW=$(echo "$VPN_IP" | sed -r 's/([0-9]+\.[0-9]+\.[0-9]+)\.[0-9]+/\1.1/')
  echo "   🔍 Constructed VPN gateway: $VPN_GW from VPN IP: $VPN_IP"
  
  # Special handling for certain IP ranges
  # If this is a 10.21.*.* network (or similar unusual assignment)
  if [[ "$VPN_IP" =~ ^10\.21\. ]]; then
    echo "   🔍 Detected 10.21.*.* network, adjusting gateway detection"
    # Get the proper subnet from the interface config
    VPN_SUBNET=$(ip route show dev "$VPN_IF" | grep -v 'default' | head -n 1 | awk '{print $1}')
    echo "   🔍 VPN subnet: $VPN_SUBNET"
    # Extract proper gateway from interface config if possible
    POSSIBLE_GW=$(ip route show dev "$VPN_IF" | grep -oP 'via \K[\d.]+' | head -n 1)
    if [ -n "$POSSIBLE_GW" ]; then
      VPN_GW="$POSSIBLE_GW"
      echo "   ✅ Found explicit gateway: $VPN_GW"
    fi
  fi
fi

echo "🔍 VPN gateway: $VPN_GW"

# Get original default gateway
ORIG_GW=$(ip route show default | grep -v "$VPN_IF" | head -n 1 | grep -oP 'via \K[\d.]+')
ORIG_IF=$(ip route show default | grep -v "$VPN_IF" | head -n 1 | grep -oP 'dev \K\w+')

if [ -z "$ORIG_GW" ] || [ -z "$ORIG_IF" ]; then
  echo "⚠️ Could not determine original gateway. This might cause issues when disconnecting."
else
  echo "🔍 Original default gateway: $ORIG_GW via $ORIG_IF"
fi

# Store original gateway for restoration
echo "$ORIG_GW $ORIG_IF" > /tmp/vpn_orig_gateway

echo "
🛠️  Starting VPN connection fixes...
"

echo "Step 1: Fix routing configuration"
echo "--------------------------------"

# Check for multiple default routes
DEFAULT_ROUTES=$(ip route show default | wc -l)
if [ "$DEFAULT_ROUTES" -gt 1 ]; then
  echo "⚠️ Found $DEFAULT_ROUTES default routes, fixing..."
  
  # Remove all default routes
  echo "   Removing all default routes..."
  ip route show default | while read -r route; do
    ip route del default $(echo "$route" | grep -oP '(via [\d.]+ )?(dev \w+)( metric \d+)?')
  done
else
  echo "✅ Default route configuration looks good"
fi

# Add VPN default route with lower metric
echo "   Setting VPN as default route with priority..."
ip route add default via "$VPN_GW" dev "$VPN_IF" metric 50

# Make sure server's actual IP address gets routed through the original gateway
# This prevents a routing loop when VPN server traffic goes through the VPN itself
echo "   Adding direct route to VPN server via original gateway..."
if [ -n "$ORIG_GW" ] && [ -n "$ORIG_IF" ]; then
  # Try to get the VPN server's actual public IP (this might not work if already connected)
  # Use a heuristic approach to find potential server IPs in active connections
  SERVER_IP=$(netstat -tn | grep ESTABLISHED | grep -v "$VPN_IP" | head -n 1 | awk '{print $5}' | cut -d: -f1)
  
  if [ -n "$SERVER_IP" ]; then
    echo "   Adding direct route to VPN server $SERVER_IP via original gateway $ORIG_GW..."
    ip route add "$SERVER_IP" via "$ORIG_GW" dev "$ORIG_IF"
  else
    echo "   ⚠️ Could not determine VPN server IP, skipping server route"
  fi
fi

# Verify routing
echo "   Checking routing configuration..."
ip route show default
echo ""

echo "Step 2: Fix DNS resolution"
echo "-------------------------"

# Check if systemd-resolved is used
if systemctl is-active systemd-resolved >/dev/null 2>&1; then
  echo "   Detected systemd-resolved, configuring..."
  
  # Create systemd-resolved config for the VPN interface
  mkdir -p /etc/systemd/resolved.conf.d/
  cat > /etc/systemd/resolved.conf.d/vpn-dns.conf << EOF
[Resolve]
DNS=8.8.8.8 8.8.4.4 1.1.1.1
DNSStubListener=yes
EOF

  # Restart systemd-resolved
  systemctl restart systemd-resolved
  echo "   ✅ systemd-resolved configured"
  
  # Set interface DNS settings using resolvectl
  resolvectl dns "$VPN_IF" 8.8.8.8 8.8.4.4
  resolvectl domain "$VPN_IF" "~."
  echo "   ✅ Interface DNS settings configured"
else
  # Direct resolv.conf configuration
  echo "   Using direct resolv.conf configuration..."
  
  # Backup original resolv.conf if it hasn't been backed up yet
  if [ ! -f /etc/resolv.conf.vpn_backup ]; then
    cp -f /etc/resolv.conf /etc/resolv.conf.vpn_backup
  fi
  
  # Create new resolv.conf
  cat > /etc/resolv.conf << EOF
# rVPNSE DNS configuration
options timeout:1
nameserver 8.8.8.8
nameserver 8.8.4.4
nameserver 1.1.1.1
EOF

  chmod 644 /etc/resolv.conf
  echo "   ✅ resolv.conf updated"
fi

echo "Step 3: Enable IP forwarding"
echo "--------------------------"
echo 1 > /proc/sys/net/ipv4/ip_forward
echo "   ✅ IP forwarding enabled"

echo "Step 4: Set up proper masquerading for VPN traffic"
echo "-----------------------------------------------"
# Get original outgoing interface
if [ -n "$ORIG_IF" ]; then
  # Check if the rule already exists
  if ! iptables -t nat -C POSTROUTING -o "$ORIG_IF" -j MASQUERADE 2>/dev/null; then
    iptables -t nat -A POSTROUTING -o "$ORIG_IF" -j MASQUERADE
    echo "   ✅ Added masquerading rule for $ORIG_IF"
  else
    echo "   ✅ Masquerading rule already exists"
  fi
fi

echo "
🔍 Testing connectivity:
"
echo "Testing DNS resolution..."
host google.com
echo ""

echo "Testing connectivity through the VPN..."
curl -s https://ipinfo.io/ip
echo " <- Your public IP should be the VPN exit node"
echo ""

echo "
✅ VPN connection fixes completed!
"

echo "To verify your traffic is flowing through the VPN, run:"
echo "curl -s https://ipinfo.io/ip"
echo ""

echo "To restore original settings when disconnecting, run:"
echo "sudo ./restore_original_network.sh"

# Create restoration script
cat > /tmp/restore_original_network.sh << EOF
#!/bin/bash
# restore_original_network.sh - Restore original network settings after VPN disconnect

set -e # Exit on error

# Verify we're running as root
if [ "\$EUID" -ne 0 ]; then
  echo "🛑 This script must be run as root (use sudo)"
  exit 1
fi

echo "🔄 Restoring original network settings..."

# Read original gateway info
if [ -f /tmp/vpn_orig_gateway ]; then
  read -r ORIG_GW ORIG_IF < /tmp/vpn_orig_gateway
  
  # Remove all default routes first
  echo "Removing all default routes..."
  ip route show default | while read -r route; do
    ip route del default \$(echo "\$route" | grep -oP '(via [\d.]+ )?(dev \w+)( metric \d+)?')
  done
  
  # Add back original default route
  if [ -n "\$ORIG_GW" ] && [ -n "\$ORIG_IF" ]; then
    echo "Restoring original default route via \$ORIG_GW dev \$ORIG_IF"
    ip route add default via "\$ORIG_GW" dev "\$ORIG_IF"
  fi
  
  # Clean up
  rm -f /tmp/vpn_orig_gateway
else
  echo "⚠️ Original gateway information not found"
fi

# Restore DNS settings
if [ -f /etc/resolv.conf.vpn_backup ]; then
  echo "Restoring original resolv.conf..."
  mv -f /etc/resolv.conf.vpn_backup /etc/resolv.conf
else
  echo "⚠️ No resolv.conf backup found"
fi

# Remove systemd-resolved VPN configuration if it exists
if [ -f /etc/systemd/resolved.conf.d/vpn-dns.conf ]; then
  rm -f /etc/systemd/resolved.conf.d/vpn-dns.conf
  systemctl restart systemd-resolved
  echo "✅ Removed VPN DNS configuration from systemd-resolved"
fi

# Remove any masquerading rules we added
iptables -t nat -D POSTROUTING -o "\$ORIG_IF" -j MASQUERADE 2>/dev/null || true

echo "✅ Original network settings restored!"
EOF

chmod +x /tmp/restore_original_network.sh
cp /tmp/restore_original_network.sh ./restore_original_network.sh
chmod +x ./restore_original_network.sh

echo "Restoration script created: restore_original_network.sh"
