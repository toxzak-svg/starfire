#!/bin/bash
# Starfire Entrypoint Script

set -euo pipefail

# Default environment. PORT is supported by common container hosts.
export STARFIRE_PORT="${STARFIRE_PORT:-${PORT:-8080}}"
export STARFIRE_DATA="${STARFIRE_DATA:-/data}"
export STARFIRE_LOG="${STARFIRE_LOG:-info}"

# Create persistent data directories.
mkdir -p "$STARFIRE_DATA/memory"
mkdir -p "$STARFIRE_DATA/logs"
mkdir -p "$STARFIRE_DATA/models"

# Render persistent disks replace the image's /data tree at runtime. Keep the
# canonical assets outside that mount and seed them only when the persistent
# copy is absent or empty. User-edited identity files and newer checkpoints are
# therefore preserved across deploys.
seed_asset() {
    local source="$1"
    local target="$2"
    local label="$3"

    if [ ! -s "$target" ]; then
        local temporary="${target}.tmp.$$"
        cp "$source" "$temporary"
        chmod 0600 "$temporary"
        mv "$temporary" "$target"
        echo "Seeded ${label}: ${target}"
    fi
}

seed_asset "/opt/starfire/assets/IDENTITY.md" \
    "$STARFIRE_DATA/IDENTITY.md" \
    "full Star identity"
seed_asset "/opt/starfire/assets/models/ckpt_e28_b500.pt" \
    "$STARFIRE_DATA/models/ckpt_e28_b500.pt" \
    "trained CharRNN reranker checkpoint"

# Set library path for libstar.so.
export LD_LIBRARY_PATH="/usr/local/lib:${LD_LIBRARY_PATH:-}"

# Containers are API deployments unless an explicit command is supplied.
if [ "$#" -eq 0 ]; then
  set -- api --host 0.0.0.0 --port "$STARFIRE_PORT"
fi

exec star --data-dir "$STARFIRE_DATA" "$@"
