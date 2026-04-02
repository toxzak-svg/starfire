#!/bin/bash
set -e

# /data is mounted as root-owned persistent volume by Railway.
# Create and chown the star data dir as root, then exec as nonroot.
mkdir -p /data/star
chown 1000:1000 /data/star

# Run star as nonroot (the default user set by Dockerfile USER directive)
exec /usr/local/bin/star --data-dir /data/star api --host 0.0.0.0 --port 8080
