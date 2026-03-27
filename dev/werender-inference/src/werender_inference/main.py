"""WeRender-Inference - Distributed LLM Inference.

Main entry point for CLI.
"""

import argparse
import sys
import os
import subprocess
from pathlib import Path
import json
import requests
from typing import Optional


# Default llama.cpp paths
DEFAULT_LLAMA_CLI = "llama-cli.exe"
DEFAULT_LLAMA_SERVER = "llama-server.exe"

# Default port
DEFAULT_PORT = 8080


def find_llama_bin() -> Optional[str]:
    """Find llama.cpp binary."""
    # Check common locations
    paths = [
        "llama-cli.exe",
        "llama-server.exe",
        "C:\\llama.cpp\\build\\bin\\Release\\llama-cli.exe",
        os.path.expanduser("~\\llama.cpp\\build\\bin\\Release\\llama-cli.exe"),
    ]
    
    for path in paths:
        if Path(path).exists():
            return path
    
    # Try PATH
    result = subprocess.run(["where", "llama-cli.exe"], capture_output=True, text=True)
    if result.returncode == 0:
        return result.stdout.strip().split('\n')[0]
    
    return None


def cmd_start(args: argparse.Namespace) -> int:
    """Start inference server."""
    print("WeRender-Inference Server")
    print("=" * 50)
    
    # Find llama binary
    llama_bin = args.llama or find_llama_bin()
    if not llama_bin:
        print("ERROR: llama-cli.exe not found!")
        print("Please either:")
        print("  1. Add llama.cpp build to PATH")
        print("  2. Specify with --llama PATH")
        print("")
        print("Get llama.cpp from: https://github.com/ggerganov/llama.cpp/releases")
        return 1
    
    print(f"Using llama.cpp: {llama_bin}")
    
    # Find model
    model_path = Path(args.model)
    if not model_path.exists():
        print(f"ERROR: Model not found: {args.model}")
        return 1
    
    print(f"Model: {args.model}")
    print(f"Port: {args.port}")
    print(f"Context: {args.context}")
    print()
    
    # Build command
    cmd = [
        llama_bin,
        "-m", str(model_path),
        "--port", str(args.port),
        "-c", str(args.context),
        "--host", "0.0.0.0",
    ]
    
    if args.gpu_layers > 0:
        cmd.extend(["-ngl", str(args.gpu_layers)])
    
    if args.threads:
        cmd.extend(["-t", str(args.threads)])
    
    print(f"Starting: {' '.join(cmd)}")
    print()
    
    # Run
    try:
        subprocess.run(cmd)
    except KeyboardInterrupt:
        print("\nShutting down...")
    
    return 0


def cmd_worker(args: args = None) -> int:
    """Start worker that connects to coordinator."""
    print("WeRender-Inference Worker")
    print("=" * 50)
    
    # Find llama binary
    llama_bin = find_llama_bin()
    if not llama_bin:
        print("ERROR: llama-cli.exe not found!")
        return 1
    
    coordinator = args.coordinator if args else "http://localhost:8080"
    print(f"Coordinator: {coordinator}")
    
    # Worker connects to coordinator's RPC
    # In full implementation, would use mDNS to discover
    print("Waiting for coordinator...")
    
    return 0


def cmd_chat(args: argparse.Namespace) -> int:
    """Chat with the inference server."""
    base_url = f"http://localhost:{args.port}/v1"
    
    # Simple chat
    messages = [{"role": "user", "content": args.prompt}]
    
    payload = {
        "model": args.model or "default",
        "messages": messages,
        "stream": False
    }
    
    try:
        response = requests.post(
            f"{base_url}/chat/completions",
            json=payload,
            timeout=120
        )
        
        if response.status_code == 200:
            result = response.json()
            print(result["choices"][0]["message"]["content"])
        else:
            print(f"Error: {response.status_code}")
            print(response.text)
            return 1
            
    except Exception as e:
        print(f"Error: {e}")
        return 1
    
    return 0


def cmd_models(args: argparse.Namespace) -> int:
    """List available models."""
    # Check common model directories
    model_dirs = [
        "C:\\llama.cpp\\models",
        os.path.expanduser("~\\llama.cpp\\models"),
        ".\\models",
        args.models_dir or "."
    ]
    
    models = []
    for dir_path in model_dirs:
        p = Path(dir_path)
        if p.exists():
            models.extend(p.glob("*.gguf"))
    
    if models:
        print("Available models:")
        for m in models:
            size_mb = m.stat().st_size / (1024 * 1024)
            print(f"  {m.name} ({size_mb:.1f} MB)")
    else:
        print("No models found. Put .gguf files in:")
        for d in model_dirs:
            print(f"  {d}")
    
    return 0


def main():
    parser = argparse.ArgumentParser(
        description="WeRender-Inference: Distributed LLM Inference"
    )
    subparsers = parser.add_subparsers(dest="command", help="Commands")
    
    # Start server
    start_parser = subparsers.add_parser("start", help="Start inference server")
    start_parser.add_argument(
        "--model", "-m",
        required=True,
        help="Path to model (.gguf file)"
    )
    start_parser.add_argument(
        "--port", "-p",
        type=int,
        default=DEFAULT_PORT,
        help=f"Port (default: {DEFAULT_PORT})"
    )
    start_parser.add_argument(
        "--context", "-c",
        type=int,
        default=4096,
        help="Context size (default: 4096)"
    )
    start_parser.add_argument(
        "--gpu-layers", "-ngl",
        type=int,
        default=99,
        help="GPU layers (default: 99)"
    )
    start_parser.add_argument(
        "--threads", "-t",
        type=int,
        default=8,
        help="CPU threads (default: 8)"
    )
    start_parser.add_argument(
        "--llama",
        help="Path to llama-cli.exe"
    )
    
    # Worker
    worker_parser = subparsers.add_parser("worker", help="Start worker")
    worker_parser.add_argument(
        "--coordinator", "-c",
        default="http://localhost:8080",
        help="Coordinator URL"
    )
    worker_parser.add_argument(
        "--gpu",
        type=int,
        help="GPU ID"
    )
    
    # Chat
    chat_parser = subparsers.add_parser("chat", help="Chat with model")
    chat_parser.add_argument(
        "--prompt", "-p",
        required=True,
        help="Your message"
    )
    chat_parser.add_argument(
        "--port", "-p",
        type=int,
        default=DEFAULT_PORT,
        help="Server port"
    )
    chat_parser.add_argument(
        "--model", "-m",
        help="Model name"
    )
    
    # Models
    models_parser = subparsers.add_parser("models", help="List available models")
    models_parser.add_argument(
        "--dir", "-d",
        dest="models_dir",
        help="Models directory"
    )
    
    args = parser.parse_args()
    
    if args.command == "start":
        return cmd_start(args)
    elif args.command == "worker":
        return cmd_worker(args)
    elif args.command == "chat":
        return cmd_chat(args)
    elif args.command == "models":
        return cmd_models(args)
    else:
        parser.print_help()
        return 1


if __name__ == "__main__":
    sys.exit(main())
