#!/bin/bash
# Starfire Entrypoint Script

set -e

# Default environment. PORT is supported by common container hosts.
export STARFIRE_PORT="${STARFIRE_PORT:-${PORT:-8080}}"
export STARFIRE_DATA="${STARFIRE_DATA:-/data}"
export STARFIRE_LOG="${STARFIRE_LOG:-info}"

# Create persistent data directories.
mkdir -p "$STARFIRE_DATA/memory"
mkdir -p "$STARFIRE_DATA/logs"

# Set library path for libstar.so.
export LD_LIBRARY_PATH="/usr/local/lib:${LD_LIBRARY_PATH:-}"

# Containers are API deployments unless an explicit command is supplied.
if [ "$#" -eq 0 ]; then
  set -- api --host 0.0.0.0 --port "$STARFIRE_PORT"
fi

exec star --data-dir "$STARFIRE_DATA" "$@"
