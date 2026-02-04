#!/bin/bash
# Run all validation checks

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

pass() { echo -e "${GREEN}✓${NC} $1"; }
fail() { echo -e "${RED}✗${NC} $1"; exit 1; }

echo "Running validation checks..."
echo ""

# Format check
echo "Checking formatting..."
cargo fmt --check && pass "Format" || fail "Format (run 'cargo fmt' to fix)"

# Clippy
echo "Running clippy..."
cargo clippy -- -D warnings && pass "Clippy" || fail "Clippy"

# Tests
echo "Running tests..."
cargo test --quiet && pass "Tests" || fail "Tests"

# Audit (optional - skip if not installed)
if command -v cargo-audit &>/dev/null; then
    echo "Running security audit..."
    cargo audit --quiet && pass "Audit" || fail "Audit"
else
    echo -e "Skipping audit (install with 'cargo install cargo-audit')"
fi

echo ""
echo -e "${GREEN}All checks passed!${NC}"
