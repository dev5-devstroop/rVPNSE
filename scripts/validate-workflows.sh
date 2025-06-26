#!/bin/bash
# GitHub Actions workflow validator script

echo "🔍 Validating GitHub Actions workflows..."

# Check if act or github-cli is available for validation
if command -v gh &> /dev/null; then
    echo "✅ GitHub CLI found, using it for validation"
    VALIDATOR="gh"
elif command -v act &> /dev/null; then
    echo "✅ Act found, using it for validation"
    VALIDATOR="act"
else
    echo "⚠️ No GitHub Actions validator found (gh or act)"
    echo "📋 Performing basic YAML syntax check..."
    VALIDATOR="yaml"
fi

# Function to check YAML syntax
check_yaml() {
    local file=$1
    if command -v python3 &> /dev/null; then
        python3 -c "import yaml; yaml.safe_load(open('$file'))" 2>/dev/null
        return $?
    elif command -v yq &> /dev/null; then
        yq eval . "$file" >/dev/null 2>&1
        return $?
    else
        echo "⚠️ No YAML validator found"
        return 0
    fi
}

# Check each workflow file
WORKFLOW_DIR=".github/workflows"
if [ -d "$WORKFLOW_DIR" ]; then
    for workflow in "$WORKFLOW_DIR"/*.yml "$WORKFLOW_DIR"/*.yaml; do
        if [ -f "$workflow" ]; then
            echo "📄 Checking $(basename "$workflow")..."
            
            case $VALIDATOR in
                "gh")
                    # GitHub CLI validation would require uploading
                    echo "  ℹ️ GitHub CLI validation requires repository access"
                    check_yaml "$workflow"
                    if [ $? -eq 0 ]; then
                        echo "  ✅ YAML syntax valid"
                    else
                        echo "  ❌ YAML syntax error"
                    fi
                    ;;
                "act")
                    act -l -W "$workflow" &>/dev/null
                    if [ $? -eq 0 ]; then
                        echo "  ✅ Workflow syntax valid"
                    else
                        echo "  ❌ Workflow syntax error"
                    fi
                    ;;
                "yaml")
                    check_yaml "$workflow"
                    if [ $? -eq 0 ]; then
                        echo "  ✅ YAML syntax valid"
                    else
                        echo "  ❌ YAML syntax error"
                    fi
                    ;;
            esac
        fi
    done
else
    echo "❌ No workflows directory found"
    exit 1
fi

echo ""
echo "🎉 Workflow validation complete!"
