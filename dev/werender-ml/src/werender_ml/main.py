"""WeRender-ML - Zero-Config Distributed ML Training.

Main entry point for the CLI application.
"""

import argparse
import sys
import os
from pathlib import Path

# Check for PyTorch
try:
    import torch
    import torch.nn as nn
    HAS_TORCH = True
except ImportError:
    HAS_TORCH = False


def cmd_coordinator(args: argparse.Namespace) -> int:
    """Start the ML training coordinator."""
    if not HAS_TORCH:
        print("PyTorch is required. Install with: pip install torch")
        return 1
    
    print("WeRender-ML Coordinator")
    print("=" * 50)
    print(f"Model: {args.model}")
    print(f"Data: {args.data}")
    print(f"Epochs: {args.epochs}")
    print(f"Batch size: {args.batch_size}")
    print(f"Dashboard: http://localhost:{args.port}")
    print()
    
    # Import coordinator
    from werender_ml.coordinator import MLCoordinator
    
    coordinator = MLCoordinator(
        model_name=args.model,
        data_path=args.data,
        epochs=args.epochs,
        batch_size=args.batch_size,
        learning_rate=args.lr,
        port=args.port,
        devices=args.devices,
        num_workers=args.workers
    )
    
    print(" Coordinator initialized")
    print(" Waiting for workers to connect...")
    print("   Run 'werender-ml worker' on other machines")
    print()
    
    coordinator.run()


def cmd_worker(args: argparse.Namespace) -> int:
    """Start a worker node."""
    if not HAS_TORCH:
        print(" PyTorch is required. Install with: pip install torch")
        return 1
    
    print(f" WeRender-ML Worker")
    print(f"=" * 50)
    
    # Import worker
    from werender_ml.worker import MLWorker
    
    worker = MLWorker(
        coordinator_url=args.coordinator,
        api_key=args.api_key,
        gpu_id=args.gpu,
        cpu_only=args.cpu
    )
    
    print(f" Looking for coordinator: {args.coordinator}")
    worker.connect()
    print(" Connected!")
    print("⏳ Waiting for training tasks...")
    
    worker.run()


def cmd_status(args: argparse.Namespace) -> int:
    """Check training status."""
    print(f" WeRender-ML Status")
    print(f"=" * 50)
    
    # TODO: Connect to coordinator API
    print("⚠️  Status dashboard coming soon")
    print("   Run 'werender-ml coordinator' first")
    
    return 0


def main():
    parser = argparse.ArgumentParser(
        description="WeRender-ML: Zero-Config Distributed ML Training"
    )
    subparsers = parser.add_subparsers(dest="command", help="Commands")
    
    # Coordinator command
    coord_parser = subparsers.add_parser(
        "coordinator",
        help="Start ML training coordinator"
    )
    coord_parser.add_argument(
        "--model", "-m",
        default="resnet18",
        help="Model name (default: resnet18)"
    )
    coord_parser.add_argument(
        "--data", "-d",
        required=True,
        help="Path to training data"
    )
    coord_parser.add_argument(
        "--epochs", "-e",
        type=int,
        default=10,
        help="Number of epochs (default: 10)"
    )
    coord_parser.add_argument(
        "--batch-size", "-b",
        type=int,
        default=32,
        help="Batch size (default: 32)"
    )
    coord_parser.add_argument(
        "--lr",
        type=float,
        default=0.001,
        help="Learning rate (default: 0.001)"
    )
    coord_parser.add_argument(
        "--port", "-p",
        type=int,
        default=8421,
        help="Dashboard port (default: 8421)"
    )
    coord_parser.add_argument(
        "--devices",
        default="auto",
        help="Device selection (default: auto)"
    )
    coord_parser.add_argument(
        "--workers", "-w",
        type=int,
        default=1,
        help="Number of workers (default: 1)"
    )
    
    # Worker command
    worker_parser = subparsers.add_parser(
        "worker",
        help="Start ML worker"
    )
    worker_parser.add_argument(
        "--coordinator", "-c",
        default="http://localhost:8421",
        help="Coordinator URL (default: http://localhost:8421)"
    )
    worker_parser.add_argument(
        "--api-key",
        help="API key for authentication"
    )
    worker_parser.add_argument(
        "--gpu", "-g",
        type=int,
        default=None,
        help="GPU ID to use (default: auto-detect)"
    )
    worker_parser.add_argument(
        "--cpu",
        action="store_true",
        help="CPU-only mode (no GPU)"
    )
    
    # Status command
    status_parser = subparsers.add_parser(
        "status",
        help="Check training status"
    )
    
    args = parser.parse_args()
    
    if args.command == "coordinator":
        return cmd_coordinator(args)
    elif args.command == "worker":
        return cmd_worker(args)
    elif args.command == "status":
        return cmd_status(args)
    else:
        parser.print_help()
        return 1


if __name__ == "__main__":
    sys.exit(main())
