#!/bin/bash
# Helper script to add new pages to the documentation

if [ $# -lt 2 ]; then
    echo "Usage: $0 <section> <page-name>"
    echo "Example: $0 guide wifi-configuration"
    echo ""
    echo "Available sections:"
    echo "  - getting-started"
    echo "  - guide"
    echo "  - advanced"
    echo "  - examples"
    echo "  - gui"
    echo "  - api"
    echo "  - development"
    echo "  - appendix"
    exit 1
fi

SECTION=$1
PAGE=$2
TITLE=$(echo "${PAGE//-/ }" | awk '{for(i=1;i<=NF;i++) $i=toupper(substr($i,1,1)) tolower(substr($i,2))}1')

# Create the file
cat > "src/${SECTION}/${PAGE}.md" << EOF
# ${TITLE}

[Content to be added]

## Overview

...

## Examples

...

## See Also

- [Related page](../guide/other.md)
EOF

echo "✓ Created src/${SECTION}/${PAGE}.md"
echo ""
echo "Next steps:"
echo "1. Add content to src/${SECTION}/${PAGE}.md"
echo "2. Add to src/SUMMARY.md in the appropriate section:"
echo "   - [${TITLE}](./${SECTION}/${PAGE}.md)"
echo "3. Build and preview: mdbook serve"
