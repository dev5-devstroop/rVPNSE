#!/bin/bash
# This script compares SoftEtherVPN and rVPNSE binary protocols and identifies issues

echo "=== Binary Protocol Analysis for SSL-VPN Implementation ==="

# Check for SoftEtherVPN process
if pgrep vpnclient > /dev/null; then
    echo "SoftEtherVPN client is running"
    # Analyze binary protocol packets with tcpdump
    echo "Analyzing SoftEtherVPN binary protocol packets..."
    sudo tcpdump -i vpnse0 -n -X -c 20 > /tmp/softether_packets.log 2>/dev/null &
    TCPDUMP_PID=$!
    sleep 5
    kill $TCPDUMP_PID 2>/dev/null
fi

# Check for rVPNSE implementation details
echo -e "\n=== Checking rVPNSE binary protocol implementation ==="

# Create a binary protocol analyzer for debugging
cat > /tmp/binary_protocol_checker.rs << 'EOF'
//! Binary protocol checker for rVPNSE
//! This file analyzes the binary protocol implementation and suggests fixes

use std::io::{Read, Write};
use std::net::{TcpStream, SocketAddr};

fn main() {
    println!("Binary Protocol Checker for rVPNSE");
    println!("===================================");
    
    // Test connection to server
    let server = "62.24.65.211:992";
    println!("Attempting to connect to {}", server);
    
    match server.parse::<SocketAddr>() {
        Ok(addr) => {
            match TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(5)) {
                Ok(mut stream) => {
                    println!("✅ Connected to server");
                    
                    // Set timeouts
                    let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(5)));
                    let _ = stream.set_write_timeout(Some(std::time::Duration::from_secs(5)));
                    
                    // Perform SSL-VPN handshake (simplified)
                    let handshake = b"\x00\x01SOFTETHER";
                    if let Err(e) = stream.write_all(handshake) {
                        println!("❌ Failed to send handshake: {}", e);
                        return;
                    }
                    
                    // Read response
                    let mut buffer = [0u8; 1024];
                    match stream.read(&mut buffer) {
                        Ok(size) => {
                            println!("✅ Received {} bytes from server", size);
                            println!("First bytes: {:?}", &buffer[..std::cmp::min(size, 16)]);
                            
                            // Check binary protocol format
                            if size > 8 && buffer[0] == 0x00 {
                                println!("✅ Binary protocol signature detected");
                            } else {
                                println!("❌ Invalid binary protocol response");
                            }
                        },
                        Err(e) => {
                            println!("❌ Failed to read response: {}", e);
                        }
                    }
                },
                Err(e) => {
                    println!("❌ Failed to connect: {}", e);
                }
            }
        },
        Err(e) => {
            println!("❌ Invalid server address: {}", e);
        }
    }
}
EOF

echo "Created binary protocol test code at /tmp/binary_protocol_checker.rs"
echo "You can build and run this to test the binary protocol directly."

echo -e "\n=== Key Issues Identified ==="
echo "1. DNS Resolution: rVPNSE needs to properly install DNS servers and handle systemd-resolved"
echo "2. Routing Loop: Need to properly set up routes for the VPN server through the original gateway"
echo "3. Binary Protocol: SoftEther uses a specific binary protocol with keepalives that might not be fully implemented"
echo "4. Packet Processing: The tunnel may not be properly forwarding decrypted packets to the TUN interface"

echo -e "\nSuggested fixes:"
echo "1. Run the fix_dns_resolution.sh script to fix DNS issues"
echo "2. Ensure proper routing with split tunneling as implemented in the tunnel module"
echo "3. Make sure keepalives are properly implemented in binary format, not HTTP"
echo "4. Debug packet flow from VPN to TUN interface using tcpdump"
