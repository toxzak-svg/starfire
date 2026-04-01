"""Global Workspace Language Model (GWL).

A PyTorch implementation of Global Workspace Theory for language modeling.

Modules:
- config: GWLConfig and TrainingConfig classes
- model: GlobalWorkspaceLM model implementation  
- trainer: GWLTrainer and TextDataset for training
"""

from .config import GWLConfig, TrainingConfig
from .model import GlobalWorkspaceLM
from .trainer import GWLTrainer, TextDataset

__all__ = [
    "GWLConfig",
    "TrainingConfig", 
    "GlobalWorkspaceLM",
    "GWLTrainer",
    "TextDataset",
]

__version__ = "0.1.0"