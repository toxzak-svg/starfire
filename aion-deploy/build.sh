#!/usr/bin/env bash
# build.sh — Build the Docker image locally and push to a registry
# 
# Usage:
#   DOCKER_REGISTRY=yourhub ./build.sh
#
# Then in Railway: Deploy from Docker Image → yourhub/aion-star:latest

set -e

REGISTRY="${DOCKER_REGISTRY:-yourhub}"
IMAGE="aion-star"
TAG="latest"
FULL="${REGISTRY}/${IMAGE}:${TAG}"

echo "=== Building Docker image: ${FULL} ==="
docker build -t "${FULL}" .

echo "=== Pushing to registry ==="
docker push "${FULL}"

echo ""
echo "Done! In Railway, set your Docker image to:"
echo "  ${FULL}"
