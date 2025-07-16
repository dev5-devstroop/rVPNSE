#!/bin/bash
# fix_vpn_routing.sh - Script to fix VPN routing issues
# This script ensures all traffic properly routes through the VPN tunnel

# Must run as root
if [ "$EUID" -ne 0 ]; then
  echo "Please run as root (with sudo)"
  exit 1
fi

# Capture original information
ORIGINAL_GATEWAY=$(ip route show default | grep -oP 'via \K\S+' | head -n1)
ORIGINAL_INTERFACE=$(ip route show default | grep -oP 'dev \K\S+' | head -n1)
VPN_INTERFACE="vpnse0"
VPN_SERVER_IP="62.24.65.211" # Update with your VPN server IP from logs

echo "=== Original Network Configuration ==="
echo "Original Gateway: $ORIGINAL_GATEWAY"
echo "Original Interface: $ORIGINAL_INTERFACE"
echo ""

# Check if VPN interface exists
if ! ip link show "$VPN_INTERFACE" &>/dev/null; then
    echo "❌ ERROR: VPN interface $VPN_INTERFACE does not exist!"
    echo "The VPN tunnel must be established first."
    exit 1
fi

echo "=== VPN Interface Information ==="
ip addr show "$VPN_INTERFACE"
echo ""

# 1. Ensure we can still reach the VPN server through original gateway
echo "🔧 Adding route to VPN server via original gateway..."
# First check if we have both required values
if [ -z "$ORIGINAL_GATEWAY" ] || [ -z "$ORIGINAL_INTERFACE" ]; then
    echo "⚠️ Warning: Could not determine original gateway or interface"
    echo "Attempting to find correct values..."
    # Find a valid gateway and interface from route table
    ORIGINAL_GATEWAY=$(ip route show | grep -v 'vpnse' | grep -oP 'via \K[0-9.]+' | head -n1)
    ORIGINAL_INTERFACE=$(ip route show | grep -v 'vpnse' | grep -oP 'dev \K\S+' | head -n1)
    echo "Found alternative gateway: $ORIGINAL_GATEWAY"
    echo "Found alternative interface: $ORIGINAL_INTERFACE"
fi

# Delete any existing routes to the VPN server to avoid conflicts
ip route del "$VPN_SERVER_IP/32" 2>/dev/null
# Add the route
ip route add "$VPN_SERVER_IP/32" via "$ORIGINAL_GATEWAY" dev "$ORIGINAL_INTERFACE"

# 2. Delete existing default route
echo "🔧 Removing existing default route..."
ip route del default

# 3. Create a new default route through VPN tunnel
echo "🔧 Setting VPN as default gateway..."
# Better parsing of peer address from interface
VPN_GATEWAY=$(ip addr show "$VPN_INTERFACE" | grep -oP 'peer \K[0-9.]+' | head -n1)
if [ -z "$VPN_GATEWAY" ]; then
    # Try alternative method to find gateway
    VPN_GATEWAY=$(ip route show | grep "$VPN_INTERFACE" | grep -oP 'via \K[0-9.]+' | head -n1)
fi
# If still not found, use the first 3 octets of local IP with .1 as last octet
if [ -z "$VPN_GATEWAY" ]; then
    LOCAL_IP=$(ip addr show "$VPN_INTERFACE" | grep -oP 'inet \K[0-9.]+' | head -n1)
    if [ -n "$LOCAL_IP" ]; then
        PREFIX=$(echo "$LOCAL_IP" | cut -d. -f1-3)
        VPN_GATEWAY="${PREFIX}.1"
    else
        # Final fallback
        VPN_GATEWAY="10.251.223.1"
    fi
fi
echo "✓ Detected VPN gateway: $VPN_GATEWAY"
ip route add default via "$VPN_GATEWAY" dev "$VPN_INTERFACE"

# 4. Add split tunneling routes (covering all IP ranges)
echo "🔧 Adding split routing to ensure all traffic uses VPN..."
# First remove any existing split routes to avoid errors
ip route del 0.0.0.0/1 2>/dev/null
ip route del 128.0.0.0/1 2>/dev/null
# Add fresh routes
ip route add 0.0.0.0/1 dev "$VPN_INTERFACE"
ip route add 128.0.0.0/1 dev "$VPN_INTERFACE"

# 4b. Disable reverse path filtering to prevent traffic blocking
echo "🔧 Disabling reverse path filtering..."
sysctl -w net.ipv4.conf.all.rp_filter=0
sysctl -w net.ipv4.conf."$VPN_INTERFACE".rp_filter=0

# 5. Set up NAT for VPN traffic
echo "🔧 Setting up NAT for VPN traffic..."
sysctl -w net.ipv4.ip_forward=1
iptables -t nat -A POSTROUTING -o "$VPN_INTERFACE" -j MASQUERADE
iptables -A FORWARD -i "$VPN_INTERFACE" -j ACCEPT

# 5b. Properly configure DNS
echo "🔧 Configuring DNS to use Google DNS..."
# Backup original resolv.conf
cp /etc/resolv.conf /etc/resolv.conf.bak
# Create new resolv.conf with Google DNS
cat > /etc/resolv.conf << EOF
nameserver 8.8.8.8
nameserver 8.8.4.4
nameserver 1.1.1.1
EOF

# 6. Verify routes
echo -e "\n=== New Routing Configuration ==="
ip route show
echo ""

# 7. Test connectivity
echo "=== Testing Connectivity ==="
echo "Trying to reach google.com..."
ping -c 3 google.com

# 8. Check public IP
echo -e "\nChecking current public IP..."
curl -s https://api.ipify.org
echo -e "\n"

# 9. Additional check for server connectivity
echo "🔧 Verifying connectivity to VPN server..."
ping -c 1 -W 2 "$VPN_SERVER_IP" > /dev/null 2>&1
if [ $? -ne 0 ]; then
    echo "⚠️ Warning: Cannot reach VPN server at $VPN_SERVER_IP"
    echo "This may indicate a routing loop or server is down"
fi

# 10. Add a keepalive helper function - you can call this later if needed
cat > /tmp/vpn-keepalive.sh << 'EOF'
#!/bin/bash
VPN_INTERFACE="vpnse0"
VPN_SERVER_IP="62.24.65.211"
VPN_GATEWAY=$(ip addr show "$VPN_INTERFACE" 2>/dev/null | grep -oP 'peer \K[0-9.]+' | head -n1)

# Check if VPN interface exists
if ! ip link show "$VPN_INTERFACE" &>/dev/null; then
    echo "❌ VPN interface not found - connection down"
    exit 1
fi

# Check if we can reach the internet
ping -c 1 -W 2 8.8.8.8 > /dev/null 2>&1
if [ $? -ne 0 ]; then
    echo "🔄 Connectivity issue detected - fixing routes..."
    # Fix routes
    ip route del default 2>/dev/null
    ip route add default via "$VPN_GATEWAY" dev "$VPN_INTERFACE"
fi

echo "✅ VPN keepalive check completed"
EOF
chmod +x /tmp/vpn-keepalive.sh

echo -e "\n✅ VPN routing configuration completed!"
echo "If your traffic is still not routing through the VPN, run:"
echo "sudo sysctl -w net.ipv4.conf.all.rp_filter=0"
echo "sudo sysctl -w net.ipv4.conf.$VPN_INTERFACE.rp_filter=0"
echo -e "\nTo maintain connectivity, you can run the helper script periodically:"
echo "sudo /tmp/vpn-keepalive.sh"
echo "Consider adding it to a cron job for automatic maintenance."
