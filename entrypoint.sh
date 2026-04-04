#!/bin/bash
# Starfire Entrypoint Script

set -e

# Default environment
export STARFIRE_PORT="${STARFIRE_PORT:-8080}"
export STARFIRE_DATA="${STARFIRE_DATA:-/data}"
export STARFIRE_LOG="${STARFIRE_LOG:-info}"

# Create data directories
mkdir -p "$STARFIRE_DATA/memory"
mkdir -p "$STARFIRE_DATA/logs"

# Set library path for libstar.so
export LD_LIBRARY_PATH="/usr/local/lib:${LD_LIBRARY_PATH}"

# Run starfire
exec star "$@"
