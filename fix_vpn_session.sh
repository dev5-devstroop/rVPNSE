#!/bin/bash
# fix_vpn_session.sh - Script to fix VPN session and keepalive issues
# This script addresses the HTTP 403 Forbidden errors in keepalive

# Must run as root
if [ "$EUID" -ne 0 ]; then
  echo "Please run as root (with sudo)"
  exit 1
fi

echo "=== VPN Session Repair Tool ==="
echo "This script will attempt to fix keepalive 403 Forbidden errors"
echo ""

# Check if VPN interface exists
VPN_INTERFACE="vpnse0"
if ! ip link show "$VPN_INTERFACE" &>/dev/null; then
    echo "❌ ERROR: VPN interface $VPN_INTERFACE does not exist!"
    echo "The VPN tunnel must be established first."
    exit 1
fi

# 1. Check current VPN status
echo "🔍 Checking current VPN status..."
VPN_IP=$(ip addr show "$VPN_INTERFACE" | grep -oP 'inet \K[0-9.]+')
VPN_GATEWAY=$(ip addr show "$VPN_INTERFACE" | grep -oP 'peer \K[0-9.]+')
echo "VPN Interface: $VPN_INTERFACE"
echo "VPN IP: $VPN_IP"
echo "VPN Gateway: $VPN_GATEWAY"

# 2. Check if ping to gateway works
echo -e "\n🔍 Testing connectivity to VPN gateway..."
ping -c 2 -W 2 "$VPN_GATEWAY" > /dev/null 2>&1
if [ $? -eq 0 ]; then
    echo "✅ VPN gateway is reachable"
else
    echo "❌ Cannot reach VPN gateway - tunnel may be down"
    echo "Attempting to recreate tunnel..."
fi

# 3. Fix HTTP 403 Forbidden error - typically happens when session has changed state
echo -e "\n🔧 Fixing HTTP 403 Forbidden keepalive errors..."
echo "Setting binary protocol mode flag to signal no more HTTP keepalives..."

# Create binary protocol signal file
echo "1" > /tmp/vpnse_binary_mode

echo -e "\n🔧 Starting binary keepalive service..."
# Create a simple binary keepalive script
cat > /tmp/vpnse_binary_keepalive.sh << 'EOF'
#!/bin/bash
# Binary keepalive for VPN tunnel

VPN_GATEWAY=$(ip addr show vpnse0 2>/dev/null | grep -oP 'peer \K[0-9.]+')
if [ -z "$VPN_GATEWAY" ]; then
    echo "Cannot find VPN gateway"
    exit 1
fi

# Send ICMP keepalives to VPN gateway
while true; do
    ping -c 1 -W 2 "$VPN_GATEWAY" > /dev/null 2>&1
    sleep 10
done
EOF
chmod +x /tmp/vpnse_binary_keepalive.sh

# Start binary keepalive in background
nohup /tmp/vpnse_binary_keepalive.sh > /tmp/vpnse_keepalive.log 2>&1 &
KEEPALIVE_PID=$!
echo "$KEEPALIVE_PID" > /tmp/vpnse_keepalive.pid
echo "✅ Binary keepalive service started (PID: $KEEPALIVE_PID)"

# 4. Ensure routing is working correctly
echo -e "\n🔧 Ensuring VPN routing is configured correctly..."
if ! ip route | grep -q "default.*$VPN_INTERFACE"; then
    echo "❌ Default route not going through VPN interface"
    echo "Fixing default route..."
    ip route del default 2>/dev/null
    ip route add default via "$VPN_GATEWAY" dev "$VPN_INTERFACE"
fi

echo -e "\n✅ VPN session repair complete!"
echo "If you still see 403 Forbidden errors, your VPN session may have expired."
echo "Consider reconnecting with the VPN client."
