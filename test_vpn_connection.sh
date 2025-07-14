#!/bin/bash

echo "🔧 VPN Connection Test Script"
echo "=============================="

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "⚠️ This script needs to be run with sudo for routing inspection"
    echo "Usage: sudo $0"
    exit 1
fi

echo "📋 Current system state BEFORE VPN connection:"
echo ""

echo "🛣️ Current routing table:"
ip route show | head -10

echo ""
echo "🌐 Current public IP:"
timeout 5 curl -s ifconfig.me || echo "Failed to get public IP"

echo ""
echo "🔍 DNS configuration:"
head -3 /etc/resolv.conf

echo ""
echo "📡 Network interfaces:"
ip addr show | grep -E "(inet |vpnse|tun)" | head -10

echo ""
echo "=========================="
echo "🚀 Starting VPN client..."
echo "=========================="
echo ""

# Run the VPN client with debug output
cd /home/akash/Documents/GitHub/rVPNSE
timeout 30 ./target/release/rvpnse-client --server 62.24.65.211 --port 992 --user devstroop --password devstroop111222 &

VPN_PID=$!
sleep 10

echo ""
echo "=========================="
echo "📋 System state AFTER VPN connection attempt:"
echo "=========================="

echo ""
echo "🛣️ Updated routing table:"
ip route show | head -15

echo ""
echo "📡 Updated network interfaces:"
ip addr show | grep -E "(inet |vpnse|tun)" | head -15

echo ""
echo "🔍 VPN interface status:"
if ip link show vpnse0 >/dev/null 2>&1; then
    echo "✅ vpnse0 interface exists"
    ip addr show vpnse0
else
    echo "❌ vpnse0 interface not found"
fi

echo ""
echo "🌐 Public IP after VPN (if working):"
timeout 5 curl -s ifconfig.me || echo "Failed to get public IP"

# Clean up
echo ""
echo "🧹 Cleaning up VPN process..."
kill $VPN_PID 2>/dev/null || true
sleep 2

echo ""
echo "✅ Test completed!"
