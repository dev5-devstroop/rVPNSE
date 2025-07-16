#!/bin/bash
# Fix VPN Routing and DNS for rVPNSE
# This script incorporates learnings from SoftEtherVPN implementation

echo "=== VPN Network Fix Script ==="
echo "Checking for VPN interface..."

# Get VPN interface
VPN_INTERFACE=$(ip addr | grep -o 'vpnse[0-9]*' | head -1)
if [ -z "$VPN_INTERFACE" ]; then
  echo "❌ No VPN interface found. Make sure the VPN is connected first."
  exit 1
fi

echo "✅ Found VPN interface: $VPN_INTERFACE"

# Get interface details
VPN_IP=$(ip addr show $VPN_INTERFACE | grep -oP 'inet \K\S+' | cut -d/ -f1)
VPN_GATEWAY=$(ip route | grep $VPN_INTERFACE | grep -oP 'via \K\S+' | head -1)
if [ -z "$VPN_GATEWAY" ]; then
  # If no gateway is found in the routes, try to get it from the peer address
  VPN_GATEWAY=$(ip addr show $VPN_INTERFACE | grep -oP 'peer \K\S+' | cut -d/ -f1)
fi

# Get original gateway
ORIGINAL_GATEWAY=$(ip route show default | grep -v $VPN_INTERFACE | grep -oP 'via \K\S+' | head -1)
if [ -z "$ORIGINAL_GATEWAY" ]; then
  # Fallback: Try to get gateway from all routes
  ORIGINAL_GATEWAY=$(ip route | grep -v $VPN_INTERFACE | grep -oP 'default via \K\S+' | head -1)
fi

# Get original interface
ORIGINAL_INTERFACE=$(ip route show default | grep -v $VPN_INTERFACE | grep -oP 'dev \K\S+' | head -1)
if [ -z "$ORIGINAL_INTERFACE" ]; then
  # Fallback: Try to get interface from all routes
  ORIGINAL_INTERFACE=$(ip route | grep -v $VPN_INTERFACE | grep -oP 'default via .+ dev \K\S+' | head -1)
fi

# Get VPN server IP (this should be passed as an argument or set as an environment variable)
VPN_SERVER=${VPN_SERVER_IP:-"62.24.65.211"}

echo "=== Original Network Configuration ==="
echo "Original Gateway: $ORIGINAL_GATEWAY"
echo "Original Interface: $ORIGINAL_INTERFACE"
echo ""
echo "=== VPN Interface Information ==="
ip addr show $VPN_INTERFACE

# Fix routing for VPN server (to prevent routing loop)
echo "🔧 Adding route to VPN server via original gateway..."
sudo ip route del $VPN_SERVER 2>/dev/null
sudo ip route add $VPN_SERVER via $ORIGINAL_GATEWAY dev $ORIGINAL_INTERFACE

# Fix default routing
echo "🔧 Removing existing default route..."
sudo ip route del default 2>/dev/null

echo "🔧 Setting VPN as default gateway..."
if [ -n "$VPN_GATEWAY" ]; then
  echo "✓ Detected VPN gateway: $VPN_GATEWAY"
  sudo ip route add default via $VPN_GATEWAY dev $VPN_INTERFACE
else
  echo "⚠️ No VPN gateway detected, using VPN interface directly for default route"
  sudo ip route add default dev $VPN_INTERFACE
fi

# Add split tunneling routes for comprehensive coverage
echo "🔧 Adding split routing to ensure all traffic uses VPN..."
sudo ip route add 0.0.0.0/1 dev $VPN_INTERFACE
sudo ip route add 128.0.0.0/1 dev $VPN_INTERFACE

# Disable reverse path filtering (important for VPN traffic)
echo "🔧 Disabling reverse path filtering..."
sudo sysctl -w net.ipv4.conf.all.rp_filter=0
sudo sysctl -w net.ipv4.conf.$VPN_INTERFACE.rp_filter=0

# Setup NAT and IP forwarding for VPN traffic
echo "🔧 Setting up NAT for VPN traffic..."
sudo sysctl -w net.ipv4.ip_forward=1
sudo iptables -t nat -F
sudo iptables -t nat -A POSTROUTING -o $VPN_INTERFACE -j MASQUERADE
sudo iptables -A FORWARD -i $VPN_INTERFACE -j ACCEPT

# Fix DNS configuration
echo "🔧 Configuring DNS to use Google DNS..."
cat > /tmp/resolv.conf.vpn << EOF
# VPN DNS Configuration
nameserver 8.8.8.8
nameserver 1.1.1.1
nameserver 8.8.4.4
options timeout:1
EOF

sudo cp /etc/resolv.conf /etc/resolv.conf.backup
sudo mv /tmp/resolv.conf.vpn /etc/resolv.conf
sudo chmod 644 /etc/resolv.conf

# Create DNS keepalive helper script
echo "🔧 Creating DNS keepalive helper script..."
cat > /tmp/vpn-keepalive.sh << 'EOF'
#!/bin/bash
# VPN Keepalive script
# This helps maintain VPN connectivity and DNS resolution

VPN_INTERFACE=$(ip addr | grep -o 'vpnse[0-9]*' | head -1)
if [ -z "$VPN_INTERFACE" ]; then
  echo "No VPN interface found. Exiting."
  exit 1
fi

# Test DNS resolution
if ! host google.com >/dev/null 2>&1; then
  echo "$(date): DNS resolution failed. Fixing..."
  
  # Restart DNS resolver
  if systemctl is-active systemd-resolved >/dev/null 2>&1; then
    sudo systemctl restart systemd-resolved
  else
    # Force DNS configuration
    echo "# VPN DNS Configuration
nameserver 8.8.8.8
nameserver 1.1.1.1
nameserver 8.8.4.4
options timeout:1" | sudo tee /etc/resolv.conf >/dev/null
  fi
  
  echo "DNS configuration restored."
fi

# Test VPN connectivity
ping -c 1 -W 2 8.8.8.8 >/dev/null 2>&1
if [ $? -ne 0 ]; then
  echo "$(date): VPN connectivity test failed."
  
  # Re-add routes
  echo "Restoring VPN routes..."
  sudo ip route add 0.0.0.0/1 dev $VPN_INTERFACE 2>/dev/null
  sudo ip route add 128.0.0.0/1 dev $VPN_INTERFACE 2>/dev/null
  
  # Reset reverse path filtering
  sudo sysctl -w net.ipv4.conf.all.rp_filter=0
  sudo sysctl -w net.ipv4.conf.$VPN_INTERFACE.rp_filter=0
  
  echo "VPN routes restored."
fi
EOF

sudo mv /tmp/vpn-keepalive.sh /tmp/vpn-keepalive.sh
sudo chmod +x /tmp/vpn-keepalive.sh

# Show the new routing configuration
echo ""
echo "=== New Routing Configuration ==="
ip route

# Test connectivity
echo ""
echo "=== Testing Connectivity ==="
echo "Trying to reach google.com..."
host google.com

echo "Checking current public IP..."
curl -s https://api.ipify.org || echo "Could not reach ipify.org"
echo ""

# Verify VPN server connectivity
echo "🔧 Verifying connectivity to VPN server..."
ping -c 1 -W 2 $VPN_SERVER >/dev/null 2>&1
if [ $? -eq 0 ]; then
  echo "✅ VPN server at $VPN_SERVER is reachable"
else
  echo "⚠️ Warning: Cannot reach VPN server at $VPN_SERVER"
  echo "This may indicate a routing loop or server is down"
fi

echo ""
echo "✅ VPN routing configuration completed!"
echo "If your traffic is still not routing through the VPN, run:"
echo "sudo sysctl -w net.ipv4.conf.all.rp_filter=0"
echo "sudo sysctl -w net.ipv4.conf.$VPN_INTERFACE.rp_filter=0"
echo ""
echo "To maintain connectivity, you can run the helper script periodically:"
echo "sudo /tmp/vpn-keepalive.sh"
echo "Consider adding it to a cron job for automatic maintenance."
