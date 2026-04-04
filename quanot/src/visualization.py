"""
Visualization Tools for QuaNot
==============================
Tools for visualizing consciousness metrics, attractors, and creative outputs.

Requires: matplotlib, numpy

Usage:
    from visualization import visualize_consciousness, visualize_attractor
    visualize_consciousness(metrics)
    visualize_attractor('lorenz')
"""

import numpy as np
from typing import Optional, List, Dict, Tuple
import warnings

# Try to import matplotlib, fallback to text if not available
try:
    import matplotlib.pyplot as plt
    import matplotlib
    matplotlib.use('Agg')  # Non-interactive backend
    HAS_MATPLOTLIB = True
except ImportError:
    HAS_MATPLOTLIB = False
    plt = None


# ============================================================================
# TEXT-BASED VISUALIZATION (Always Available)
# ============================================================================

def print_banner(text: str, width: int = 60):
    """Print a text banner."""
    print("=" * width)
    print(f"  {text}")
    print("=" * width)


def visualize_metrics_text(metrics: Dict):
    """Visualize metrics as text."""
    print("\n--- Metrics ---")
    for key, value in metrics.items():
        if isinstance(value, float):
            print(f"  {key}: {value:.4f}")
        elif isinstance(value, list):
            print(f"  {key}: [{len(value)} values]")
        else:
            print(f"  {key}: {value}")


def visualize_bar(value: float, width: int = 20, label: str = "") -> str:
    """Create ASCII bar chart."""
    filled = int(value * width)
    bar = "[" + "#" * filled + "-" * (width - filled) + "]"
    return f"{label} {bar} {value:.2f}"


def print_consciousness_dashboard(metrics: Dict):
    """Print consciousness dashboard in text."""
    print("\n" + "=" * 60)
    print("  CONSCIOUSNESS DASHBOARD")
    print("=" * 60)
    
    print("\nCore Metrics:")
    if 'causal_density' in metrics:
        print(f"  {visualize_bar(metrics.get('causal_density', 0), label='Causal Density')}")
    if 'neural_complexity' in metrics:
        print(f"  {visualize_bar(metrics.get('neural_complexity', 0), label='Neural Complexity')}")
    if 'temporal_granularity' in metrics:
        print(f"  {visualize_bar(metrics.get('temporal_granularity', 0), label='Temporal Granularity')}")
    if 'phenomenological_binding' in metrics:
        print(f"  {visualize_bar(metrics.get('phenomenological_binding', 0), label='Binding')}")
    
    if 'overall_consciousness' in metrics:
        print(f"\nOverall Consciousness: {metrics['overall_consciousness']:.4f}")
        print(visualize_bar(metrics['overall_consciousness'], width=40, label=""))
    
    print()


# ============================================================================
# MATPLOTLIB VISUALIZATION
# ============================================================================

if HAS_MATPLOTLIB:
    
    def visualize_consciousness(
        metrics: Dict,
        save_path: Optional[str] = None
    ):
        """
        Visualize consciousness metrics.
        
        Parameters
        ----------
        metrics : dict
            Consciousness metrics dictionary
        save_path : str
            Path to save figure (optional)
        """
        fig, axes = plt.subplots(2, 2, figsize=(12, 10))
        fig.suptitle('Consciousness Metrics', fontsize=14, fontweight='bold')
        
        # 1. Core metrics bar chart
        ax1 = axes[0, 0]
        metric_names = ['causal_density', 'neural_complexity', 
                       'temporal_granularity', 'phenomenological_binding']
        values = [metrics.get(m, 0) for m in metric_names]
        names = ['Causal\nDensity', 'Neural\nComplexity', 'Temporal\nGranularity', 'Binding']
        
        bars = ax1.bar(names, values, color=['#3498db', '#2ecc71', '#e74c3c', '#9b59b6'])
        ax1.set_ylim(0, 1)
        ax1.set_ylabel('Score')
        ax1.set_title('Core Consciousness Metrics')
        ax1.axhline(y=0.5, color='gray', linestyle='--', alpha=0.5)
        
        # 2. Heat map if available
        ax2 = axes[0, 1]
        if 'consciousness_heatmap' in metrics:
            heatmap = np.array(metrics['consciousness_heatmap'])
            if len(heatmap) > 0:
                ax2.plot(heatmap, color='#3498db', alpha=0.7)
                ax2.fill_between(range(len(heatmap)), heatmap, alpha=0.3)
                ax2.set_xlabel('Time')
                ax2.set_ylabel('Consciousness')
                ax2.set_title('Consciousness Over Time')
        
        # 3. Radar chart
        ax3 = axes[1, 0]
        angles = np.linspace(0, 2*np.pi, len(metric_names), endpoint=False).tolist()
        angles += angles[:1]
        values_plot = values + [values[0]]
        
        ax3 = fig.add_subplot(2, 2, 3, projection='polar')
        ax3.plot(angles, values_plot, 'o-', linewidth=2, color='#3498db')
        ax3.fill(angles, values_plot, alpha=0.25, color='#3498db')
        ax3.set_xticks(angles[:-1])
        ax3.set_xticklabels(names)
        ax3.set_ylim(0, 1)
        ax3.set_title('Consciousness Profile')
        
        # 4. Overall score gauge
        ax4 = axes[1, 1]
        overall = metrics.get('overall_consciousness', 0)
        
        # Simple gauge using bar
        colors = ['#e74c3c', '#f39c12', '#2ecc71']  # red, yellow, green
        thresholds = [0.3, 0.6, 1.0]
        
        for i, (color, threshold) in enumerate(zip(colors, thresholds)):
            if overall >= threshold:
                bar_color = color
        
        ax4.barh(['Consciousness'], [overall], color=bar_color)
        ax4.set_xlim(0, 1)
        ax4.set_title(f'Overall Score: {overall:.3f}')
        ax4.set_xlabel('Score')
        
        plt.tight_layout()
        
        if save_path:
            plt.savefig(save_path, dpi=150, bbox_inches='tight')
            print(f"Saved to: {save_path}")
        
        plt.close()


    def visualize_attractor(
        attractor_type: str = 'lorenz',
        n_steps: int = 1000,
        save_path: Optional[str] = None
    ):
        """
        Visualize strange attractors.
        
        Parameters
        ----------
        attractor_type : str
            Type of attractor ('lorenz', 'rossler', 'henon')
        n_steps : int
            Number of steps
        save_path : str
            Path to save figure
        """
        from chaos import lorenz_attractor, rossler_attractor, henon_map
        
        fig = plt.figure(figsize=(12, 4))
        
        if attractor_type == 'lorenz':
            traj = lorenz_attractor(n_steps)
            ax1 = fig.add_subplot(131, projection='3d')
            ax1.plot(traj[:, 0], traj[:, 1], traj[:, 2], 'b-', alpha=0.5)
            ax1.set_title('Lorenz Attractor')
            ax1.set_xlabel('X')
            ax1.set_ylabel('Y')
            ax1.set_zlabel('Z')
            
            ax2 = fig.add_subplot(132)
            ax2.plot(traj[:200, 0], traj[:200, 1], 'b-')
            ax2.set_title('XY Projection')
            ax2.set_xlabel('X')
            ax2.set_ylabel('Y')
            
            ax3 = fig.add_subplot(133)
            ax3.plot(traj[:, 2], 'b-', alpha=0.5)
            ax3.set_title('Z over time')
            ax3.set_xlabel('Time')
            ax3.set_ylabel('Z')
            
        elif attractor_type == 'rossler':
            traj = rossler_attractor(n_steps)
            ax1 = fig.add_subplot(131, projection='3d')
            ax1.plot(traj[:, 0], traj[:, 1], traj[:, 2], 'r-', alpha=0.5)
            ax1.set_title('Rossler Attractor')
            
        elif attractor_type == 'henon':
            traj = henon_map(n_steps)
            ax1 = fig.add_subplot(111)
            ax1.plot(traj[:, 0], traj[:, 1], 'g.', alpha=0.3, markersize=1)
            ax1.set_title('Henon Map')
            ax1.set_xlabel('X')
            ax1.set_ylabel('Y')
        
        plt.tight_layout()
        
        if save_path:
            plt.savefig(save_path, dpi=150, bbox_inches='tight')
            print(f"Saved to: {save_path}")
        
        plt.close()


    def visualize_creative_output(
        novelty_history: List[float],
        creativity_scores: List[float],
        save_path: Optional[str] = None
    ):
        """
        Visualize creative output over time.
        
        Parameters
        ----------
        novelty_history : list
            History of novelty scores
        creativity_scores : list
            History of creativity scores
        save_path : str
            Path to save figure
        """
        fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(10, 8))
        
        # Novelty
        ax1.plot(novelty_history, 'b-', alpha=0.7)
        ax1.fill_between(range(len(novelty_history)), novelty_history, alpha=0.3)
        ax1.set_xlabel('Cycle')
        ax1.set_ylabel('Novelty')
        ax1.set_title('Novelty Over Time')
        ax1.set_ylim(0, 1)
        
        # Creativity
        ax2.plot(creativity_scores, 'g-', alpha=0.7)
        ax2.fill_between(range(len(creativity_scores)), creativity_scores, alpha=0.3, color='green')
        ax2.set_xlabel('Cycle')
        ax2.set_ylabel('Creativity')
        ax2.set_title('Creativity Over Time')
        ax2.set_ylim(0, 1)
        
        plt.tight_layout()
        
        if save_path:
            plt.savefig(save_path, dpi=150, bbox_inches='tight')
            print(f"Saved to: {save_path}")
        
        plt.close()


    def visualize_state_space(
        states: np.ndarray,
        labels: Optional[np.ndarray] = None,
        save_path: Optional[str] = None
    ):
        """
        Visualize state space using PCA-like projection.
        
        Parameters
        ----------
        states : np.ndarray
            State trajectory
        labels : np.ndarray
            Optional labels for states
        save_path : str
            Path to save figure
        """
        # Simple projection: just take first 2 dimensions or use PCA-like
        if states.shape[1] >= 2:
            x = states[:, 0]
            y = states[:, 1]
        else:
            x = np.arange(len(states))
            y = states[:, 0]
        
        fig, ax = plt.subplots(figsize=(10, 8))
        
        if labels is not None:
            scatter = ax.scatter(x, y, c=labels, cmap='viridis', alpha=0.6)
            plt.colorbar(scatter, ax=ax, label='State Label')
        else:
            ax.scatter(x, y, alpha=0.6, c=range(len(x)), cmap='viridis')
        
        ax.set_xlabel('Dimension 1')
        ax.set_ylabel('Dimension 2')
        ax.set_title('State Space Visualization')
        
        # Add trajectory line
        ax.plot(x, y, 'k-', alpha=0.2, linewidth=0.5)
        
        plt.tight_layout()
        
        if save_path:
            plt.savefig(save_path, dpi=150, bbox_inches='tight')
            print(f"Saved to: {save_path}")
        
        plt.close()


else:
    # Fallback functions when matplotlib not available
    def visualize_consciousness(metrics, save_path=None):
        """Text fallback for visualize_consciousness."""
        print("Matplotlib not available. Using text visualization:")
        print_consciousness_dashboard(metrics)
    
    def visualize_attractor(attractor_type, n_steps, save_path=None):
        """Text fallback for visualize_attractor."""
        from chaos import lorenz_attractor, rossler_attractor, henon_map
        
        print(f"\n--- {attractor_type.title()} Attractor ---")
        
        if attractor_type == 'lorenz':
            traj = lorenz_attractor(n_steps)
            print(f"  X range: [{traj[:, 0].min():.2f}, {traj[:, 0].max():.2f}]")
            print(f"  Y range: [{traj[:, 1].min():.2f}, {traj[:, 1].max():.2f}]")
            print(f"  Z range: [{traj[:, 2].min():.2f}, {traj[:, 2].max():.2f}]")
        elif attractor_type == 'rossler':
            traj = rossler_attractor(n_steps)
            print(f"  Shape: {traj.shape}")
        elif attractor_type == 'henon':
            traj = henon_map(n_steps)
            print(f"  Shape: {traj.shape}")
    
    def visualize_creative_output(novelty_history, creativity_scores, save_path=None):
        """Text fallback for visualize_creative_output."""
        print("\n--- Creative Output ---")
        print(f"  Novelty: min={min(novelty_history):.3f}, max={max(novelty_history):.3f}, avg={np.mean(novelty_history):.3f}")
        print(f"  Creativity: min={min(creativity_scores):.3f}, max={max(creativity_scores):.3f}, avg={np.mean(creativity_scores):.3f}")
    
    def visualize_state_space(states, labels=None, save_path=None):
        """Text fallback for visualize_state_space."""
        print("\n--- State Space ---")
        print(f"  Shape: {states.shape}")
        print(f"  Dim 1: min={states[:, 0].min():.3f}, max={states[:, 0].max():.3f}")
        if states.shape[1] > 1:
            print(f"  Dim 2: min={states[:, 1].min():.3f}, max={states[:, 1].max():.3f}")


# ============================================================================
# EXPORT FUNCTIONS
# ============================================================================

def export_metrics_json(metrics: Dict, filepath: str):
    """Export metrics to JSON."""
    import json
    
    # Convert numpy types to Python types
    serializable = {}
    for key, value in metrics.items():
        if isinstance(value, np.ndarray):
            serializable[key] = value.tolist()
        elif isinstance(value, (np.float32, np.float64)):
            serializable[key] = float(value)
        elif isinstance(value, (np.int32, np.int64)):
            serializable[key] = int(value)
        elif isinstance(value, list):
            serializable[key] = [float(v) if isinstance(v, (np.floating, np.integer)) else v for v in value]
        else:
            serializable[key] = value
    
    with open(filepath, 'w') as f:
        json.dump(serializable, f, indent=2)
    
    print(f"Exported to: {filepath}")


def export_metrics_csv(metrics: Dict, filepath: str):
    """Export metrics to CSV."""
    import csv
    
    with open(filepath, 'w', newline='') as f:
        writer = csv.writer(f)
        writer.writerow(['Metric', 'Value'])
        
        for key, value in metrics.items():
            if not isinstance(value, (list, np.ndarray)):
                writer.writerow([key, value])
    
    print(f"Exported to: {filepath}")


# ============================================================================
# DEMO
# ============================================================================

if __name__ == "__main__":
    print("=" * 60)
    print("  QuaNot Visualization Tools")
    print("=" * 60)
    
    # Demo metrics
    demo_metrics = {
        'causal_density': 0.65,
        'neural_complexity': 0.72,
        'temporal_granularity': 0.58,
        'phenomenological_binding': 0.81,
        'overall_consciousness': 0.69,
        'consciousness_heatmap': [0.5, 0.6, 0.7, 0.65, 0.55, 0.6, 0.7, 0.75, 0.8, 0.7]
    }
    
    print("\n[1] Text Dashboard:")
    print_consciousness_dashboard(demo_metrics)
    
    print("\n[2] Attractor (text):")
    visualize_attractor('lorenz', n_steps=500)
    
    print("\n[3] Creative Output (text):")
    visualize_creative_output(
        [0.5, 0.6, 0.7, 0.8, 0.6, 0.5, 0.7, 0.9],
        [0.4, 0.5, 0.6, 0.7, 0.5, 0.4, 0.6, 0.8]
    )
    
    print("\n[4] State Space (text):")
    states = np.random.randn(100, 3)
    visualize_state_space(states)
    
    print("\nDone!")