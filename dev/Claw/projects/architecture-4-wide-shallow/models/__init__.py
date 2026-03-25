"""
Wide and Shallow Transformer Models Package

Core architecture: 8K-16K attention heads across 2-4 layers.
"""

from .wide_shallow import (
    WideShallowTransformer,
    WideShallowDecoderLayer,
    GroupedQueryAttention,
    HeadwiseMLP,
    create_wide_shallow_model,
)

__all__ = [
    "WideShallowTransformer",
    "WideShallowDecoderLayer", 
    "GroupedQueryAttention",
    "HeadwiseMLP",
    "create_wide_shallow_model",
]
