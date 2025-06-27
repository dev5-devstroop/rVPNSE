#!/bin/bash
#
# Shell wrapper for the README benchmark updater
# This script calls the Python implementation that parses Criterion.rs results
#

set -euo pipefail

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}📊 rVPNSE README Benchmark Updater${NC}"
echo "=================================================="

# Check if Python is available
if ! command -v python3 &> /dev/null; then
    echo -e "${RED}❌ Python 3 is required but not installed${NC}"
    exit 1
fi

# Check if the Python script exists
PYTHON_SCRIPT="${SCRIPT_DIR}/update-readme-benchmarks.py"
if [[ ! -f "${PYTHON_SCRIPT}" ]]; then
    echo -e "${RED}❌ Python script not found: ${PYTHON_SCRIPT}${NC}"
    exit 1
fi

# Check if README.md exists
README_FILE="${PROJECT_ROOT}/README.md"
if [[ ! -f "${README_FILE}" ]]; then
    echo -e "${YELLOW}⚠️  README.md not found at ${README_FILE}${NC}"
    echo "Creating a basic README.md with benchmark markers..."
    
    cat > "${README_FILE}" << 'EOF'
# rVPNSE

A high-performance VPN static library written in Rust.

<!-- BENCHMARK_RESULTS_START -->
<!-- This section is automatically updated by CI -->
<!-- BENCHMARK_RESULTS_END -->

## Features

- High-performance VPN client library
- C FFI for cross-language integration
- Cross-platform support (Linux, macOS, Windows)
- Memory efficient design
- Async-ready API

EOF
    echo -e "${GREEN}✅ Created basic README.md${NC}"
fi

# Change to project root directory
cd "${PROJECT_ROOT}"

echo -e "${BLUE}🔍 Checking for benchmark results...${NC}"

# Check if we have any benchmark results
CRITERION_DIR="${PROJECT_ROOT}/target/criterion"
if [[ -d "${CRITERION_DIR}" ]]; then
    RESULT_COUNT=$(find "${CRITERION_DIR}" -name "estimates.json" | wc -l)
    echo -e "${GREEN}📈 Found ${RESULT_COUNT} benchmark result(s)${NC}"
else
    echo -e "${YELLOW}⚠️  No criterion results found, will use sample data${NC}"
fi

echo -e "${BLUE}🐍 Running Python benchmark parser...${NC}"

# Run the Python script
if python3 "${PYTHON_SCRIPT}"; then
    echo -e "${GREEN}✅ README.md benchmark section updated successfully!${NC}"
    
    # Show a summary of what was updated
    if command -v git &> /dev/null && git rev-parse --git-dir > /dev/null 2>&1; then
        echo -e "${BLUE}📝 Git diff summary:${NC}"
        git diff --stat README.md || true
    fi
    
    exit 0
else
    echo -e "${RED}❌ Failed to update README.md benchmark section${NC}"
    exit 1
fi
