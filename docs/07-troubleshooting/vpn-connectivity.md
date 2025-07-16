# Troubleshooting VPN Connectivity

This document provides guidance on resolving common connectivity issues when using rVPNSE.

## Common Issues

### 1. Authentication Succeeds but No Internet Through VPN

This is a common issue where the VPN tunnel is created but traffic doesn't actually route through it. 
The most likely causes are:

1. **Routing conflicts**: Multiple default routes causing traffic to go through your regular connection
2. **DNS resolution failure**: Cannot resolve domain names through the VPN
3. **VPN server route configuration**: When VPN server traffic goes through the VPN itself (routing loop)

### 2. DNS Resolution Failures

If you can ping IP addresses but can't resolve domain names:

1. The DNS servers provided by the VPN are not being used
2. systemd-resolved is not properly configured for the VPN interface
3. Your system is still using your original DNS servers

### 3. Partial Connectivity

If some websites/services work but others don't:

1. MTU issues causing large packets to be dropped
2. Selective routing or split tunneling issues
3. Missing routes for certain networks

## Diagnostic Tools

We provide several scripts to help diagnose and fix VPN connectivity issues:

### verify_vpn_connectivity.sh

This script performs comprehensive checks on your VPN connection:

```bash
sudo ./verify_vpn_connectivity.sh
```

It checks:
- VPN interface presence and configuration
- Routing table configuration
- DNS resolution
- Potential traffic leaks
- Public IP address (to verify traffic is actually going through the VPN)

### fix_vpn_connection.sh

This script applies fixes for common VPN connectivity issues:

```bash
sudo ./fix_vpn_connection.sh
```

It addresses:
- Routing conflicts by ensuring proper default route configuration
- DNS resolution issues by configuring systemd-resolved or resolv.conf
- VPN server route issues by adding direct routes to prevent loops
- IP forwarding and masquerading configuration

### install_vpn_hooks.sh

For a more permanent solution, this script installs system hooks that automatically configure routing and DNS when the VPN connects or disconnects:

```bash
sudo ./install_vpn_hooks.sh
```

This creates:
- Automatic routing configuration when a VPN interface appears
- Proper DNS setup for the VPN connection
- Automatic cleanup when the VPN disconnects

## Manual Troubleshooting Steps

If the scripts don't resolve your issue, try these manual troubleshooting steps:

### 1. Check VPN Interface

```bash
ip addr show
```

Look for a tun/tap interface with an assigned IP address.

### 2. Check Routing Table

```bash
ip route show
ip route show default
```

Verify that:
- There is only ONE default route
- The default route points to your VPN interface
- There is a specific route to the VPN server through your original interface

### 3. Check DNS Configuration

```bash
cat /etc/resolv.conf
```

Verify that:
- The nameservers are correctly set (should point to VPN DNS servers)
- There are no conflicting DNS configurations

### 4. Test Connectivity

```bash
ping 8.8.8.8   # Test IP connectivity
host google.com # Test DNS resolution
curl https://ipinfo.io/ip # Check your public IP
```

### 5. Manual Fix for Multiple Default Routes

```bash
# Remove all default routes
sudo ip route show default | while read -r route; do 
  sudo ip route del default $(echo "$route" | grep -oP '(via [\d.]+ )?(dev \w+)( metric \d+)?'); 
done

# Add VPN default route with lower metric
sudo ip route add default via YOUR_VPN_GATEWAY dev YOUR_VPN_INTERFACE metric 50
```

## Advanced Issues

If you're still experiencing issues after trying the above solutions:

1. **Packet Capture**: Use `tcpdump` to capture and analyze VPN traffic
2. **MTU Issues**: Try reducing MTU on the VPN interface (`sudo ip link set dev INTERFACE mtu 1400`)
3. **Firewall Rules**: Check if your firewall is blocking VPN traffic
4. **VPN Server Logs**: If you have access, check the VPN server logs

## Getting Help

If you're still experiencing issues:
1. Open an issue in the repository with the output of `verify_vpn_connectivity.sh`
2. Include your system information and VPN client configuration (redact sensitive info)
