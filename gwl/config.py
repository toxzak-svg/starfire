"""Global Workspace Theory (GWL) Configuration Module."""

from dataclasses import dataclass, field
from typing import Optional


@dataclass
class GWLConfig:
    """Configuration for Global Workspace Language Model.
    
    This config defines the architecture of a GWL model based on Global
    Workspace Theory - using a set of specialized "processor" modules that
    compete for attention, with a shared "workspace" that broadcasts 
    information to all processors.
    """
    
    # Model dimensions
    vocab_size: int = 50257
    embed_dim: int = 256
    workspace_dim: int = 512
    
    # Processor configuration
    num_processors: int = 32
    processor_config__hidden_dim: int = 128
    
    # Number of competition/attention steps per forward pass
    num_steps: int = 3
    
    # Architectural options
    use_residual_workspace: bool = True
    use_layer_norm: bool = True
    use_dropout: bool = True
    dropout_prob: float = 0.1
    
    # Processor type: 'mlp', 'lstm', or 'transformer'
    processor_type: str = 'mlp'
    
    # Competition mechanism: 'softmax', 'sigmoid', 'top_k'
    competition_type: str = 'softmax'
    
    # Whether to use gating in workspace
    use_workspace_gating: bool = True
    
    # Max tokens for generation
    max_gen_tokens: int = 512
    
    def __post_init__(self):
        """Validate configuration."""
        assert self.vocab_size > 0, "vocab_size must be positive"
        assert self.embed_dim > 0, "embed_dim must be positive"
        assert self.workspace_dim > 0, "workspace_dim must be positive"
        assert self.num_processors > 0, "num_processors must be positive"
        assert self.processor_config__hidden_dim > 0, "processor hidden_dim must be positive"
        assert self.num_steps > 0, "num_steps must be positive"
        assert self.processor_type in ['mlp', 'lstm', 'transformer'], \
            f"Invalid processor_type: {self.processor_type}"
        assert self.competition_type in ['softmax', 'sigmoid', 'top_k'], \
            f"Invalid competition_type: {self.competition_type}"


@dataclass
class TrainingConfig:
    """Configuration for GWL training."""
    
    # Batch and optimization
    batch_size: int = 32
    learning_rate: float = 1e-4
    weight_decay: float = 0.01
    grad_clip: float = 1.0
    
    # Learning rate schedule
    warmup_steps: int = 500
    
    # Training duration
    max_steps: int = 10000
    
    # Evaluation and logging
    eval_interval: int = 500
    log_interval: int = 50
    save_interval: int = 1000
    
    # Checkpointing
    checkpoint_dir: str = "checkpoints"
    
    # Mixed precision
    use_amp: bool = True
    
    # Gradient accumulation
    gradient_accumulation_steps: int = 1
    
    # Early stopping
    early_stopping_patience: int = 5
    
    def __post_init__(self):
        """Validate training config."""
        assert self.batch_size > 0, "batch_size must be positive"
        assert self.learning_rate > 0, "learning_rate must be positive"
        assert self.max_steps > 0, "max_steps must be positive"