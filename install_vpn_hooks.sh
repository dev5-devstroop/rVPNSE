#!/bin/bash
# install_vpn_hooks.sh - Install systemd hooks for proper VPN connect/disconnect handling

set -e # Exit on error

# Verify we're running as root
if [ "$EUID" -ne 0 ]; then
  echo "🛑 This script must be run as root (use sudo)"
  exit 1
fi

echo "🔧 Installing rVPNSE VPN hooks..."

# Create script directory if it doesn't exist
mkdir -p /usr/local/sbin/rvpnse

# Create the connection handler script
cat > /usr/local/sbin/rvpnse/vpn-up.sh << 'EOF'
#!/bin/bash
# vpn-up.sh - Executed when VPN interface comes up

# Log the event
logger -t "rVPNSE" "VPN interface detected, setting up routing and DNS"

# Store original gateway for later restoration
ORIG_GW=$(ip route show default | head -n 1 | grep -oP 'via \K[\d.]+')
ORIG_IF=$(ip route show default | head -n 1 | grep -oP 'dev \K\w+')
echo "$ORIG_GW $ORIG_IF" > /var/run/rvpnse_orig_gateway

# Find VPN interface (it might take a moment to appear)
for i in {1..10}; do
  VPN_IF=$(ip link show | grep -E 'tun|vpnse' | head -n 1 | cut -d: -f2 | tr -d ' ')
  if [ -n "$VPN_IF" ]; then
    break
  fi
  sleep 0.5
done

if [ -z "$VPN_IF" ]; then
  logger -t "rVPNSE" "No VPN interface found after waiting"
  exit 1
fi

# Get VPN IP address (may take a moment to be assigned)
for i in {1..10}; do
  VPN_IP=$(ip addr show dev "$VPN_IF" | grep -oP 'inet \K[\d.]+')
  if [ -n "$VPN_IP" ]; then
    break
  fi
  sleep 0.5
done

# Get VPN gateway
VPN_GW=$(echo "$VPN_IP" | sed -r 's/([0-9]+\.[0-9]+\.[0-9]+)\.[0-9]+/\1.1/')

logger -t "rVPNSE" "VPN interface: $VPN_IF, IP: $VPN_IP, Gateway: $VPN_GW"

# Fix routing
# Remove all default routes
ip route show default | while read -r route; do
  ip route del default $(echo "$route" | grep -oP '(via [\d.]+ )?(dev \w+)( metric \d+)?')
done

# Add VPN default route with lower metric
ip route add default via "$VPN_GW" dev "$VPN_IF" metric 50

# Configure DNS
if systemctl is-active systemd-resolved >/dev/null 2>&1; then
  # Using systemd-resolved
  mkdir -p /etc/systemd/resolved.conf.d/
  cat > /etc/systemd/resolved.conf.d/vpn-dns.conf << EOD
[Resolve]
DNS=8.8.8.8 8.8.4.4 1.1.1.1
DNSStubListener=yes
EOD

  systemctl restart systemd-resolved
  
  # Set interface DNS settings using resolvectl
  resolvectl dns "$VPN_IF" 8.8.8.8 8.8.4.4
  resolvectl domain "$VPN_IF" "~."
else
  # Direct resolv.conf configuration
  if [ ! -f /etc/resolv.conf.vpn_backup ]; then
    cp -f /etc/resolv.conf /etc/resolv.conf.vpn_backup
  fi
  
  # Create new resolv.conf
  cat > /etc/resolv.conf << EOD
# rVPNSE DNS configuration
options timeout:1
nameserver 8.8.8.8
nameserver 8.8.4.4
nameserver 1.1.1.1
EOD

  chmod 644 /etc/resolv.conf
fi

# Enable IP forwarding
echo 1 > /proc/sys/net/ipv4/ip_forward

# Set up masquerading
if [ -n "$ORIG_IF" ]; then
  iptables -t nat -A POSTROUTING -o "$ORIG_IF" -j MASQUERADE 2>/dev/null || true
fi

logger -t "rVPNSE" "VPN setup completed successfully"
EOF

# Create the disconnection handler script
cat > /usr/local/sbin/rvpnse/vpn-down.sh << 'EOF'
#!/bin/bash
# vpn-down.sh - Executed when VPN interface goes down

# Log the event
logger -t "rVPNSE" "VPN interface down, restoring original network settings"

# Read original gateway info
if [ -f /var/run/rvpnse_orig_gateway ]; then
  read -r ORIG_GW ORIG_IF < /var/run/rvpnse_orig_gateway
  
  # Remove all default routes first
  ip route show default | while read -r route; do
    ip route del default $(echo "$route" | grep -oP '(via [\d.]+ )?(dev \w+)( metric \d+)?')
  done
  
  # Add back original default route
  if [ -n "$ORIG_GW" ] && [ -n "$ORIG_IF" ]; then
    logger -t "rVPNSE" "Restoring original default route via $ORIG_GW dev $ORIG_IF"
    ip route add default via "$ORIG_GW" dev "$ORIG_IF"
  fi
  
  # Clean up
  rm -f /var/run/rvpnse_orig_gateway
fi

# Restore DNS settings
if [ -f /etc/resolv.conf.vpn_backup ]; then
  mv -f /etc/resolv.conf.vpn_backup /etc/resolv.conf
fi

# Remove systemd-resolved VPN configuration if it exists
if [ -f /etc/systemd/resolved.conf.d/vpn-dns.conf ]; then
  rm -f /etc/systemd/resolved.conf.d/vpn-dns.conf
  systemctl restart systemd-resolved
fi

# Remove masquerading rules if they exist
if [ -n "$ORIG_IF" ]; then
  iptables -t nat -D POSTROUTING -o "$ORIG_IF" -j MASQUERADE 2>/dev/null || true
fi

logger -t "rVPNSE" "Original network settings restored"
EOF

# Make scripts executable
chmod +x /usr/local/sbin/rvpnse/vpn-up.sh
chmod +x /usr/local/sbin/rvpnse/vpn-down.sh

# Create udev rules to detect VPN interface changes
cat > /etc/udev/rules.d/99-rvpnse-vpn.rules << 'EOF'
# rVPNSE VPN Interface Rules
ACTION=="add", SUBSYSTEM=="net", ENV{INTERFACE}=="tun*", RUN+="/usr/local/sbin/rvpnse/vpn-up.sh"
ACTION=="remove", SUBSYSTEM=="net", ENV{INTERFACE}=="tun*", RUN+="/usr/local/sbin/rvpnse/vpn-down.sh"
ACTION=="add", SUBSYSTEM=="net", ENV{INTERFACE}=="vpnse*", RUN+="/usr/local/sbin/rvpnse/vpn-up.sh"
ACTION=="remove", SUBSYSTEM=="net", ENV{INTERFACE}=="vpnse*", RUN+="/usr/local/sbin/rvpnse/vpn-down.sh"
EOF

# Reload udev rules
udevadm control --reload-rules

echo "✅ rVPNSE VPN hooks installed successfully!"
echo ""
echo "🔹 VPN interfaces will now be automatically configured when connected"
echo "🔹 Network settings will be restored when the VPN disconnects"
echo ""
echo "To remove these hooks, run:"
echo "sudo rm -rf /usr/local/sbin/rvpnse /etc/udev/rules.d/99-rvpnse-vpn.rules"
