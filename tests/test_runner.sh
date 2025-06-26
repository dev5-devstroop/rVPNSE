#!/bin/bash

# Test runner for Rust VPNSE VPN Gate integration
# This script runs various tests to validate our SoftEther VPN implementation

set -e  # Exit on any error

echo "🦀 Rust VPNSE Test Suite"
echo "======================"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Please run this script from the rvpnse directory"
    exit 1
fi

# Build the project
print_status "Building project..."
if cargo build; then
    print_success "Project built successfully"
else
    print_error "Failed to build project"
    exit 1
fi

echo ""

# Run unit tests
print_status "Running unit tests..."
if cargo test --lib; then
    print_success "Unit tests passed"
else
    print_error "Unit tests failed"
    exit 1
fi

echo ""

# Test packet creation and parsing
print_status "Testing packet creation and parsing..."
if cargo run test-packets; then
    print_success "Packet tests passed"
else
    print_error "Packet tests failed"
    exit 1
fi

echo ""

# List available VPN Gate servers
print_status "Listing available VPN Gate servers..."
if cargo run list-servers; then
    print_success "Server listing completed"
else
    print_warning "Server listing failed (network issue?)"
fi

echo ""

# Run performance benchmarks
print_status "Running performance benchmarks..."
if cargo run benchmark; then
    print_success "Benchmarks completed"
else
    print_error "Benchmarks failed"
    exit 1
fi

echo ""

# Test connectivity to fastest servers (optional - requires network)
print_status "Testing connectivity to fastest servers..."
if cargo run test-fastest 2; then
    print_success "Connectivity tests completed"
else
    print_warning "Connectivity tests failed (network issues expected)"
fi

echo ""

# Protocol tests with real servers (optional - requires network)
print_status "Testing protocol implementation with real servers..."
if timeout 30 cargo run test-protocol; then
    print_success "Protocol tests completed"
else
    print_warning "Protocol tests failed or timed out (network issues expected)"
fi

echo ""
print_success "All available tests completed!"
echo ""
echo "📝 Summary:"
echo "  • Unit tests: PASSED"
echo "  • Packet creation/parsing: PASSED" 
echo "  • Performance benchmarks: PASSED"
echo "  • Server listing: COMPLETED"
echo "  • Network tests: ATTEMPTED (results may vary)"
echo ""
echo "🎉 Rust VPNSE test suite finished successfully!"
echo ""
echo "Next steps:"
echo "  1. Run 'cargo run test-connectivity' to test all server connections"
echo "  2. Run 'cargo run help' to see all available commands"
echo "  3. Examine logs for any warnings or errors"
