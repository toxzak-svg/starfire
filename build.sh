#!/bin/bash
# =============================================================================
# Starfire Build Script
# =============================================================================
# Simple build script for Starfire + QuaNot.
#
# Usage:
#   ./build.sh all       - Build everything
#   ./build.sh starfire - Build Starfire
#   ./build.sh quanot   - Setup QuaNot
#   ./build.sh clean    - Clean artifacts
# =============================================================================

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
QUANOT_DIR="$PROJECT_DIR/quanot"

log() { echo -e "${BLUE}[*]${NC} $1"; }
ok() { echo -e "${GREEN}[OK]${NC} $1"; }
err() { echo -e "${RED}[ERROR]${NC} $1"; }

# =============================================================================
# Build Starfire (Rust)
# =============================================================================
build_starfire() {
    log "Building Starfire..."
    cd "$PROJECT_DIR"
    cargo build --release
    ok "Starfire ready: target/release/star.exe"
}

# =============================================================================
# Setup QuaNot (Python)
# =============================================================================
setup_quanot() {
    log "Setting up QuaNot..."
    cd "$QUANOT_DIR"
    
    # Check if venv exists
    if [ -d ".venv" ]; then
        ok "Virtual environment exists"
    else
        log "Creating venv..."
        python -m venv .venv
    fi
    
    # Install dependencies
    log "Installing dependencies..."
    .venv/Scripts/pip install -r ../requirements.txt
    
    # Test
    log "Testing QuaNot..."
    .venv/Scripts/python src/main.py > /dev/null 2>&1
    
    ok "QuaNot ready"
}

# =============================================================================
# Build All
# =============================================================================
build_all() {
    log "Building Starfire + QuaNot..."
    
    build_starfire
    setup_quanot
    
    ok "Build complete!"
    echo ""
    echo "Run: ./target/release/star.exe"
}

# =============================================================================
# Clean
# =============================================================================
clean() {
    log "Cleaning..."
    cargo clean
    find "$QUANOT_DIR" -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null || true
    ok "Clean complete"
}

# =============================================================================
# Main
# =============================================================================
case "${1:-all}" in
    all) build_all ;;
    starfire) build_starfire ;;
    quanot) setup_quanot ;;
    clean) clean ;;
    *) 
        echo "Usage: $0 <all|starfire|quanot|clean>"
        ;;
esac
