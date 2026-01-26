#!/bin/bash
set -e

# Extract metadata from Cargo.toml
VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
DESCRIPTION=$(grep '^description' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
AUTHOR=$(grep '^authors' Cargo.toml | head -1 | sed 's/.*\["\(.*\)"\].*/\1/')
REPOSITORY=$(grep '^repository' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
LICENSE=$(grep '^license' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
# Extract keywords array (TOML array format is valid JSON)
KEYWORDS=$(grep '^keywords' Cargo.toml | head -1 | sed 's/keywords = //')

echo "Building reframe-wasm v$VERSION"
echo "  Description: $DESCRIPTION"
echo "  Author: $AUTHOR"
echo "  Repository: $REPOSITORY"
echo "  License: $LICENSE"
echo ""

# Build WASM package for web target
echo "Building WASM package..."
wasm-pack build --target web --out-dir pkg

# Update package.json with correct metadata from Cargo.toml
cd pkg
cat > package.json << EOF
{
  "name": "@goplasmatic/reframe-wasm",
  "version": "$VERSION",
  "description": "$DESCRIPTION",
  "type": "module",
  "main": "reframe_wasm.js",
  "types": "reframe_wasm.d.ts",
  "files": [
    "reframe_wasm_bg.wasm",
    "reframe_wasm.js",
    "reframe_wasm.d.ts",
    "reframe_wasm_bg.wasm.d.ts"
  ],
  "repository": {
    "type": "git",
    "url": "$REPOSITORY"
  },
  "keywords": $KEYWORDS,
  "author": "$AUTHOR",
  "license": "$LICENSE",
  "bugs": {
    "url": "$REPOSITORY/issues"
  },
  "homepage": "$REPOSITORY#readme"
}
EOF

echo ""
echo "Package built successfully! (v$VERSION)"
echo ""
echo "To publish to npm:"
echo "  cd pkg && npm publish --access public"
echo ""
echo "To test locally:"
echo "  cd pkg && npm link"
