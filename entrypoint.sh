#!/bin/bash
set -e

# /data is mounted as a persistent volume by Railway, owned by root.
# Create and chown the star data dir, then exec as nonroot.
mkdir -p /data/star-data
chown 1000:1000 /data/star-data

exec sudo -u nonroot /usr/local/bin/star --data-dir /data/star-data api --host 0.0.0.0 --port 8080
