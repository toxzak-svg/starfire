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

reranker_target="$STARFIRE_DATA/models/ckpt_e28_b500.pt"

# A ZIP header here means an earlier deploy persisted the unrelated PyTorch
# checkpoint under the native CharRNN filename. Remove only that known-bad
# format so the compatible bundled checkpoint can be seeded in its place.
if [ -s "$reranker_target" ]; then
    reranker_magic="$(head -c 4 "$reranker_target" | od -An -tx1 | tr -d '[:space:]')"
    if [ "$reranker_magic" = "504b0304" ]; then
        echo "Replacing incompatible PyTorch ZIP reranker checkpoint: $reranker_target"
        rm -f "$reranker_target"
    fi
fi

seed_asset "/opt/starfire/assets/models/ckpt_e28_b500.pt" \
    "$reranker_target" \
    "native CharRNN reranker checkpoint"

# Set library path for libstar.so.
export LD_LIBRARY_PATH="/usr/local/lib:${LD_LIBRARY_PATH:-}"

# Containers are API deployments unless an explicit command is supplied.
if [ "$#" -eq 0 ]; then
  set -- api --host 0.0.0.0 --port "$STARFIRE_PORT"
fi

exec star --data-dir "$STARFIRE_DATA" "$@"
