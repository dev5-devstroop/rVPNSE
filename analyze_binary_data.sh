#!/bin/bash

echo "🔬 VPN Binary Session Data Analysis"
echo "===================================="

echo "📡 Testing connection to VPN server to analyze binary session data..."
echo ""

cd /home/akash/Documents/GitHub/rVPNSE

# Enable Rust logging to see the binary data analysis
export RUST_LOG=info,rvpnse::protocol::pack=debug

echo "🚀 Running VPN client with enhanced logging..."
echo "   This will show binary session data parsing in detail"
echo ""

timeout 15 ./target/release/rvpnse-client --server 62.24.65.211 --port 992 --user devstroop --password devstroop111222 2>&1 | grep -E "(🔍|📦|🎯|⚠️|📊|📍|binary session|IP|gateway|netmask|Found|potential)"

echo ""
echo "✅ Analysis completed!"
echo ""
echo "💡 Key things to look for:"
echo "   - 'Found potential IP' messages showing discovered IPs"
echo "   - 'Selected IP configuration' showing chosen settings" 
echo "   - Binary data hex dump for manual analysis"
echo "   - Any parsing errors or fallback usage"
