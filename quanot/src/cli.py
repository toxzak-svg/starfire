"""
QuaNot CLI - Command-Line Interface
=====================================
Interactive command-line interface for QuaNot AGI system.

Usage:
    python src/cli.py                    # Interactive mode
    python src/cli.py --demo              # Run demo
    python src/cli.py --help              # Show help
"""

import sys
import argparse
import numpy as np
from pathlib import Path

# Add src to path
sys.path.insert(0, str(Path(__file__).parent))

from agi_core import create_agi_system, AGISystem
from consciousness_enhanced import compute_all_consciousness_metrics
from creativity import NoveltyDetector, CreativeSynthesizer
from chaos import lorenz_attractor, rossler_attractor
from reservoir import ChaoticReservoir


class QuaNotCLI:
    """QuaNot Command-Line Interface."""
    
    def __init__(self, state_dim=64, reservoir_size=128, seed=42):
        """Initialize CLI with AGI system."""
        self.system = create_agi_system(
            state_dim=state_dim,
            reservoir_size=reservoir_size,
            random_seed=seed
        )
        self.running = True
    
    def print_banner(self):
        """Print welcome banner."""
        print("=" * 60)
        print("  QuaNot - AGI Architecture")
        print("  Quantum-Inspired + Chaos + Creativity + Consciousness")
        print("=" * 60)
        print()
    
    def print_help(self):
        """Print help message."""
        print("""
Commands:
  help               - Show this help message
  status             - Show system status
  process <input>    - Process input through AGI
  creative           - Run creative cycle
  consciousness      - Get consciousness metrics
  metrics            - Show all metrics
  run <n>            - Run n cycles
  reset              - Reset system
  quit               - Exit CLI
  
Examples:
  process random     - Process random input
  process sin        - Process sine wave
  run 10             - Run 10 cycles
  metrics            - Show consciousness metrics
        """)
    
    def do_status(self):
        """Show system status."""
        status = self.system.get_system_info()
        print("\n--- System Status ---")
        print(f"  State dimension: {status['state_dim']}")
        print(f"  Reservoir size: {status['reservoir_size']}")
        print(f"  Cycles: {status['status']['metrics']['total_cycles']}")
        print(f"  Avg creativity: {status['status']['metrics']['avg_creativity']:.4f}")
        print(f"  Avg consciousness: {status['status']['metrics']['avg_consciousness']:.4f}")
        print(f"  Avg novelty: {status['status']['metrics']['avg_novelty']:.4f}")
        print(f"  Reservoir regime: {status['status']['reservoir_regime']}")
        print()
    
    def do_process(self, input_type="random"):
        """Process input through system."""
        if input_type == "random":
            input_data = np.random.randn(self.system.core.state_dim)
        elif input_type == "sin":
            t = np.linspace(0, 4*np.pi, self.system.core.state_dim)
            input_data = np.sin(t)
        elif input_type == "zeros":
            input_data = np.zeros(self.system.core.state_dim)
        else:
            print(f"Unknown input type: {input_type}")
            return
        
        result = self.system.run({'default': input_data})
        
        print(f"\n--- Processing: {input_type} ---")
        print(f"  Creativity: {result['creative']['creativity']:.4f}")
        print(f"  Novelty: {result['novelty']:.4f}")
        print(f"  Consciousness: {result['consciousness']['consciousness_level']:.4f}")
        print(f"  Reservoir: {result['reservoir']['regime']}")
        print()
    
    def do_creative(self):
        """Run creative cycle."""
        state = np.random.randn(self.system.core.state_dim)
        result = self.system.core.creative_synthesizer.creative_step(state, mode='explore')
        
        print(f"\n--- Creative Output ---")
        print(f"  Novelty: {result['novelty']:.4f}")
        print(f"  Creativity: {result['evaluation']['overall_creativity']:.4f}")
        print(f"  Mode: {result['mode']}")
        print()
    
    def do_consciousness(self):
        """Get consciousness metrics."""
        # Get states from history
        if len(self.system.core.state_history) < 10:
            print("Not enough history for consciousness metrics. Run 'run 20' first.")
            return
        
        states = np.array(list(self.system.core.state_history))
        metrics = compute_all_consciousness_metrics(states)
        
        print("\n--- Consciousness Metrics ---")
        print(f"  Causal density: {metrics['causal_density']:.4f}")
        print(f"  Neural complexity: {metrics['neural_complexity']:.4f}")
        print(f"  Temporal granularity: {metrics['temporal_granularity']:.4f}")
        print(f"  Phenomenological binding: {metrics['phenomenological_binding']:.4f}")
        print(f"  Overall consciousness: {metrics['overall_consciousness']:.4f}")
        print()
    
    def do_metrics(self):
        """Show all metrics."""
        self.do_status()
        self.do_consciousness()
    
    def do_run(self, n=10):
        """Run n cycles."""
        print(f"\n--- Running {n} cycles ---")
        for i in range(n):
            input_data = np.random.randn(self.system.core.state_dim)
            self.system.run({'default': input_data})
            if (i + 1) % 5 == 0:
                print(f"  Completed {i + 1}/{n}")
        print(f"  Done!")
        self.do_status()
    
    def do_reset(self):
        """Reset system."""
        self.system.reset()
        print("\nSystem reset.")
        print()
    
    def run_interactive(self):
        """Run interactive CLI."""
        self.print_banner()
        self.print_help()
        
        while self.running:
            try:
                cmd = input("quanot> ").strip().split()
                if not cmd:
                    continue
                
                command = cmd[0].lower()
                args = cmd[1:] if len(cmd) > 1 else []
                
                if command in ['quit', 'exit', 'q']:
                    self.running = False
                    print("Goodbye!")
                elif command == 'help':
                    self.print_help()
                elif command == 'status':
                    self.do_status()
                elif command == 'process':
                    input_type = args[0] if args else "random"
                    self.do_process(input_type)
                elif command == 'creative':
                    self.do_creative()
                elif command == 'consciousness':
                    self.do_consciousness()
                elif command == 'metrics':
                    self.do_metrics()
                elif command == 'run':
                    n = int(args[0]) if args else 10
                    self.do_run(n)
                elif command == 'reset':
                    self.do_reset()
                else:
                    print(f"Unknown command: {command}")
                    print("Type 'help' for available commands.")
                    
            except KeyboardInterrupt:
                print("\nUse 'quit' to exit.")
            except Exception as e:
                print(f"Error: {e}")


def run_demo():
    """Run demonstration."""
    print("=" * 60)
    print("  QuaNot Demo")
    print("=" * 60)
    print()
    
    # Create system
    cli = QuaNotCLI(state_dim=32, seed=42)
    
    # Run some cycles
    print("[1] Running initial cycles...")
    cli.do_run(10)
    
    # Get consciousness metrics
    print("[2] Computing consciousness metrics...")
    # Add more history
    cli.do_run(20)
    cli.do_consciousness()
    
    # Creative output
    print("[3] Creative output...")
    cli.do_creative()
    
    print("\nDemo complete!")
    print("Run 'python src/cli.py' for interactive mode.")


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(description="QuaNot AGI CLI")
    parser.add_argument('--demo', action='store_true', help='Run demo')
    parser.add_argument('--state-dim', type=int, default=64, help='State dimension')
    parser.add_argument('--reservoir-size', type=int, default=128, help='Reservoir size')
    parser.add_argument('--seed', type=int, default=42, help='Random seed')
    
    args = parser.parse_args()
    
    if args.demo:
        run_demo()
    else:
        cli = QuaNotCLI(
            state_dim=args.state_dim,
            reservoir_size=args.reservoir_size,
            seed=args.seed
        )
        cli.run_interactive()


if __name__ == "__main__":
    main()