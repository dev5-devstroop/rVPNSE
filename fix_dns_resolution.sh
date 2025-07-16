#!/bin/bash
# Fix DNS resolution issues in rVPNSE
# Based on comparison with SoftEtherVPN implementation

echo "=== Fixing DNS Resolution for rVPNSE ==="
echo "Checking current DNS settings..."

# Save original DNS settings
cp /etc/resolv.conf /etc/resolv.conf.rvpnse_backup

# Get the VPN interface details
VPN_INTERFACE=$(ip addr | grep vpnse | head -n 1 | cut -d: -f2 | tr -d ' ')
echo "VPN interface detected: $VPN_INTERFACE"

# Get the VPN DNS servers (SoftEtherVPN uses a specific approach for DNS)
echo "Setting up proper DNS resolution through VPN..."

# Create a new resolv.conf with direct file write (avoiding systemd-resolved issues)
cat > /tmp/resolv.conf.vpn << EOF
# DNS Configuration for rVPNSE VPN
nameserver 8.8.8.8
nameserver 1.1.1.1
nameserver 8.8.4.4
options timeout:1
EOF

# Apply the DNS configuration
echo "Applying VPN DNS configuration..."
sudo mv /tmp/resolv.conf.vpn /etc/resolv.conf

# Fix common systemd-resolved issues that might interfere with DNS
if command -v systemctl > /dev/null && systemctl is-active systemd-resolved > /dev/null; then
    echo "systemd-resolved is active, configuring it for VPN compatibility..."
    
    # Create a systemd-resolved config for the VPN interface
    sudo mkdir -p /etc/systemd/resolved.conf.d/
    cat > /tmp/vpn-dns.conf << EOF
[Resolve]
DNS=8.8.8.8 1.1.1.1
DNSStubListener=no
EOF
    sudo mv /tmp/vpn-dns.conf /etc/systemd/resolved.conf.d/
    
    # Restart systemd-resolved
    sudo systemctl restart systemd-resolved
fi

# Test DNS resolution
echo "Testing DNS resolution..."
echo "Trying to resolve google.com..."
host google.com

# Check routing
echo -e "\n=== Current Routing Table ==="
ip route

echo -e "\n=== Current DNS Configuration ==="
cat /etc/resolv.conf

echo -e "\nDNS Fix Complete!"
echo "If DNS issues persist, check /etc/nsswitch.conf and add 'dns' to the hosts line"
