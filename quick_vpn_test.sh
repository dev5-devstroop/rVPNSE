#!/bin/bash

echo "🚀 Quick VPN Traffic Test"
echo "========================"

if [ "$EUID" -ne 0 ]; then
    echo "⚠️ This test needs to be run with sudo"
    echo "Usage: sudo $0"
    exit 1
fi

echo "📋 Current state before VPN:"
echo "Current public IP:"
timeout 3 curl -s ifconfig.me || echo "Failed to get IP"

echo ""
echo "Current default route:"
ip route show default

echo ""
echo "🚀 Testing VPN connection with server-assigned IPs..."

cd /home/akash/Documents/GitHub/rVPNSE

# Run VPN client with improved parsing
timeout 20 ./target/release/rvpnse-client --server 62.24.65.211 --port 992 --user devstroop --password devstroop111222 &

VPN_PID=$!
echo "VPN PID: $VPN_PID"

# Wait for connection to establish
sleep 8

echo ""
echo "📋 VPN connection state:"

# Check if VPN interface exists
if ip link show vpnse0 >/dev/null 2>&1; then
    echo "✅ vpnse0 interface exists"
    echo "Interface details:"
    ip addr show vpnse0
    echo ""
    echo "Routing table:"
    ip route show | head -10
    echo ""
    echo "Testing if traffic goes through VPN:"
    timeout 5 curl -s ifconfig.me || echo "Failed to get IP after VPN"
else
    echo "❌ vpnse0 interface not found"
    echo "Current interfaces:"
    ip link show | grep -E "(vpnse|tun)"
fi

echo ""
echo "🧹 Cleaning up..."
kill $VPN_PID 2>/dev/null || true
sleep 2
kill -9 $VPN_PID 2>/dev/null || true

echo "✅ Test completed!"
