#!/bin/bash
# Starfire Entrypoint Script

set -euo pipefail

# Default environment. PORT is supported by common container hosts.
export STARFIRE_PORT="${STARFIRE_PORT:-${PORT:-8080}}"
export STARFIRE_DATA="${STARFIRE_DATA:-/data}"
export STARFIRE_LOG="${STARFIRE_LOG:-info}"
# ΩV1-F2 is compiled into the binary but remains inert until this explicit
# switch is enabled after the external build/deploy gate succeeds.
export STARFIRE_OMEGA_V1F2_SHADOW="${STARFIRE_OMEGA_V1F2_SHADOW:-0}"

# The CLI currently resolves an explicit data directory to its nested life/
# directory unless SPEC.md is present. Keep /data as the canonical asset store,
# then expose those assets at the effective runtime path without duplicating them.
runtime_data="$STARFIRE_DATA/life"

mkdir -p "$STARFIRE_DATA/memory"
mkdir -p "$STARFIRE_DATA/logs"
mkdir -p "$STARFIRE_DATA/models"
mkdir -p "$runtime_data/memory"
mkdir -p "$runtime_data/logs"
mkdir -p "$runtime_data/models"

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

f2_model_target="$STARFIRE_DATA/models/omega_v1f1r1_model.json"
seed_asset "/opt/starfire/assets/models/omega_v1f1r1_model.json" \
    "$f2_model_target" \
    "bounded ΩV1-F1R1 shadow model"

# Runtime::new receives /data/life from the current CLI path resolver. Link the
# canonical persistent assets into that effective directory so identity and the
# trained reranker are loaded from the same files on every boot.
ln -sfn "$STARFIRE_DATA/IDENTITY.md" "$runtime_data/IDENTITY.md"
ln -sfn "$reranker_target" "$runtime_data/models/ckpt_e28_b500.pt"
ln -sfn "$f2_model_target" "$runtime_data/models/omega_v1f1r1_model.json"

# Set library path for libstar.so.
export LD_LIBRARY_PATH="/usr/local/lib:${LD_LIBRARY_PATH:-}"

# Containers are API deployments unless an explicit command is supplied.
if [ "$#" -eq 0 ]; then
  set -- api --host 0.0.0.0 --port "$STARFIRE_PORT"
fi

exec star --data-dir "$STARFIRE_DATA" "$@"