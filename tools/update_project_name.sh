#!/bin/bash

# Script to update project name from RVPNSE/rvpnse to rVPNSE
# Usage: ./update_project_name.sh

echo "🔄 Updating project name from RVPNSE/rvpnse to rVPNSE..."

# Find all markdown files in docs directory
find docs/ -name "*.md" -type f | while read -r file; do
    echo "Processing: $file"
    
    # Create backup
    cp "$file" "$file.bak"
    
    # Update RVPNSE to rVPNSE (uppercase)
    sed -i.tmp 's/RVPNSE/rVPNSE/g' "$file"
    
    # Update standalone rvpnse to rVPNSE (but keep library/file names)
    # This preserves librvpnse, rvpnse.h, etc. but updates project references
    sed -i.tmp 's/\([^a-zA-Z0-9_-]\)rvpnse\([^a-zA-Z0-9_.-]\)/\1rVPNSE\2/g' "$file"
    sed -i.tmp 's/^rvpnse\([^a-zA-Z0-9_.-]\)/rVPNSE\1/g' "$file"
    sed -i.tmp 's/\([[:space:]]\)rvpnse$/\1rVPNSE/g' "$file"
    
    # Clean up temp files
    rm -f "$file.tmp"
    
    # Check if file was actually changed
    if ! diff -q "$file" "$file.bak" > /dev/null 2>&1; then
        echo "  ✅ Updated: $file"
        rm "$file.bak"
    else
        echo "  ➖ No changes: $file"
        rm "$file.bak"
    fi
done

echo ""
echo "✅ Project name update complete!"
echo ""
echo "📋 Summary of changes:"
echo "  • RVPNSE → rVPNSE (all uppercase instances)"
echo "  • rvpnse → rVPNSE (standalone project name references)"
echo "  • Preserved: librvpnse, rvpnse.h, rvpnse.so, etc. (library/file names)"
echo ""
echo "🔍 To verify changes, run:"
echo "  grep -r 'RVPNSE\\|rvpnse[^a-zA-Z0-9_.-]' docs/ | grep -v 'librvpnse\\|rvpnse\\.\\|rvpnse-'"
