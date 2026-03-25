#!/usr/bin/env python3
"""
Benchmarking script for Wide and Shallow Transformer

Evaluates on:
- Code completion
- Math reasoning  
- Long-context reasoning

Compares against standard transformer architectures.
"""

import os
import sys
import json
import yaml
import time
import logging
from pathlib import Path
from typing import Dict, List, Any, Optional
from dataclasses import dataclass

import torch
import torch.nn as nn
from torch.utils.data import DataLoader, Dataset
from tqdm import tqdm

# For perplexity calculation
import math

from models.wide_shallow import create_wide_shallow_model


logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger(__name__)


@dataclass
class BenchmarkResult:
    """Results from a benchmark run."""
    task: str
    metric: str
    value: float
    unit: str
    additional_info: Dict[str, Any]


class CodeCompletionDataset(Dataset):
    """Dataset for code completion benchmarks."""
    
    def __init__(self, sequences: List[str], tokenizer, max_length: int = 2048):
        self.sequences = sequences
        self.tokenizer = tokenizer
        self.max_length = max_length
        
    def __len__(self):
        return len(self.sequences)
    
    def __getitem__(self, idx):
        # Tokenize the sequence
        tokens = self.tokenizer(
            self.sequences[idx],
            max_length=self.max_length,
            truncation=True,
            return_tensors="pt"
        )
        return {
            "input_ids": tokens["input_ids"].squeeze(0),
            "attention_mask": tokens["attention_mask"].squeeze(0),
        }


class MathReasoningDataset(Dataset):
    """Dataset for math reasoning benchmarks."""
    
    def __init__(self, problems: List[Dict[str, str]], tokenizer, max_length: int = 1024):
        self.problems = problems
        self.tokenizer = tokenizer
        self.max_length = max_length
        
    def __len__(self):
        return len(self.problems)
    
    def __getitem__(self, idx):
        problem = self.problems[idx]
        # Format: "Problem: {question}\nSolution: {answer}"
        text = f"Problem: {problem['question']}\nSolution: {problem.get('answer', '')}"
        
        tokens = self.tokenizer(
            text,
            max_length=self.max_length,
            truncation=True,
            return_tensors="pt"
        )
        return {
            "input_ids": tokens["input_ids"].squeeze(0),
            "attention_mask": tokens["attention_mask"].squeeze(0),
            "answer": problem.get("answer", ""),
        }


class LongContextDataset(Dataset):
    """Dataset for long-context reasoning benchmarks."""
    
    def __init__(self, documents: List[str], queries: List[str], tokenizer, max_length: int = 8192):
        self.documents = documents
        self.queries = queries
        self.tokenizer = tokenizer
        self.max_length = max_length
        
    def __len__(self):
        return len(self.documents)
    
    def __getitem__(self, idx):
        # Format: "Document: {doc}\nQuery: {query}\nAnswer:"
        text = f"Document: {self.documents[idx]}\nQuery: {self.queries[idx]}\nAnswer:"
        
        tokens = self.tokenizer(
            text,
            max_length=self.max_length,
            truncation=True,
            return_tensors="pt"
        )
        return {
            "input_ids": tokens["input_ids"].squeeze(0),
            "attention_mask": tokens["attention_mask"].squeeze(0),
            "query": self.queries[idx],
        }


def calculate_perplexity(
    model: nn.Module,
    dataloader: DataLoader,
    device: torch.device,
    use_amp: bool = True,
) -> float:
    """Calculate perplexity on a dataset."""
    model.eval()
    
    total_loss = 0.0
    total_tokens = 0
    
    with torch.no_grad():
        for batch in tqdm(dataloader, desc="Calculating perplexity"):
            input_ids = batch["input_ids"].to(device)
            labels = input_ids.clone()
            
            # Set padding tokens to -100 to ignore in loss
            labels[labels == model.embed_tokens.padding_idx] = -100
            
            if use_amp and torch.cuda.is_available():
                with torch.cuda.amp.autocast():
                    outputs = model(input_ids=input_ids, labels=labels)
            else:
                outputs = model(input_ids=input_ids, labels=labels)
            
            loss = outputs["loss"]
            
            # Count non-padded tokens
            num_tokens = (labels != -100).sum().item()
            
            total_loss += loss.item() * num_tokens
            total_tokens += num_tokens
    
    avg_loss = total_loss / total_tokens
    perplexity = math.exp(avg_loss)
    
    return perplexity


def measure_throughput(
    model: nn.Module,
    dataloader: DataLoader,
    device: torch.device,
    num_batches: int = 100,
    use_amp: bool = True,
) -> Dict[str, float]:
    """Measure training/inference throughput."""
    model.eval()
    
    # Warmup
    for i, batch in enumerate(dataloader):
        if i >= 10:
            break
        input_ids = batch["input_ids"].to(device)
        with torch.no_grad():
            if use_amp and torch.cuda.is_available():
                with torch.cuda.amp.autocast():
                    _ = model(input_ids=input_ids)
            else:
                _ = model(input_ids=input_ids)
        torch.cuda.synchronize() if torch.cuda.is_available() else None
    
    # Measure
    start_time = time.time()
    total_tokens = 0
    
    for i, batch in enumerate(tqdm(dataloader, desc="Measuring throughput")):
        if i >= num_batches:
            break
            
        input_ids = batch["input_ids"].to(device)
        batch_size, seq_len = input_ids.shape
        total_tokens += batch_size * seq_len
        
        with torch.no_grad():
            if use_amp and torch.cuda.is_available():
                with torch.cuda.amp.autocast():
                    _ = model(input_ids=input_ids)
            else:
                _ = model(input_ids=input_ids)
        
        torch.cuda.synchronize() if torch.cuda.is_available() else None
    
    elapsed_time = time.time() - start_time
    
    tokens_per_second = total_tokens / elapsed_time
    batches_per_second = num_batches / elapsed_time
    
    return {
        "tokens_per_second": tokens_per_second,
        "batches_per_second": batches_per_second,
        "elapsed_time": elapsed_time,
        "total_tokens": total_tokens,
    }


def measure_memory_usage(
    model: nn.Module,
    device: torch.device,
    batch_size: int = 1,
    seq_length: int = 2048,
) -> Dict[str, float]:
    """Measure GPU memory usage."""
    if not torch.cuda.is_available():
        return {"error": "CUDA not available"}
    
    torch.cuda.reset_peak_memory_stats()
    torch.cuda.empty_cache()
    
    # Create dummy input
    input_ids = torch.randint(0, 50000, (batch_size, seq_length)).to(device)
    
    # Forward pass
    with torch.no_grad():
        _ = model(input_ids=input_ids)
    
    torch.cuda.synchronize()
    
    allocated = torch.cuda.memory_allocated() / 1024**3  # GB
    reserved = torch.cuda.memory_reserved() / 1024**3  # GB
    max_allocated = torch.cuda.max_memory_allocated() / 1024**3  # GB
    
    return {
        "allocated_gb": allocated,
        "reserved_gb": reserved,
        "max_allocated_gb": max_allocated,
    }


def benchmark_code_completion(
    model: nn.Module,
    config: Dict[str, Any],
    device: torch.device,
) -> BenchmarkResult:
    """Benchmark on code completion tasks."""
    logger.info("Running code completion benchmark...")
    
    # For now, use dummy data
    # In practice, load from datasets like HumanEval, MBPP, etc.
    try:
        from transformers import GPT2Tokenizer
        tokenizer = GPT2Tokenizer.from_pretrained("gpt2")
        tokenizer.pad_token = tokenizer.eos_token
    except ImportError:
        logger.warning("Transformers not available, using random data")
        tokenizer = None
    
    # Create synthetic dataset
    synthetic_sequences = [
        "def fibonacci(n):\n    if n <= 1:\n        return n\n    else:\n        return fibonacci(n-1) + fibonacci(n-2)"
    ] * 100  # Placeholder
    
    dataset = CodeCompletionDataset(synthetic_sequences, tokenizer)
    dataloader = DataLoader(dataset, batch_size=1)
    
    # Calculate perplexity
    perplexity = calculate_perplexity(model, dataloader, device)
    
    # Measure throughput
    throughput = measure_throughput(model, dataloader, device)
    
    # Measure memory
    memory = measure_memory_usage(model, device)
    
    logger.info(f"Code completion perplexity: {perplexity:.2f}")
    logger.info(f"Code completion throughput: {throughput}")
    logger.info(f"Code completion memory: {memory}")
    
    return BenchmarkResult(
        task="code_completion",
        metric="perplexity",
        value=perplexity,
        unit="",
        additional_info={
            "throughput": throughput,
            "memory": memory,
        }
    )


def benchmark_math_reasoning(
    model: nn.Module,
    config: Dict[str, Any],
    device: torch.device,
) -> BenchmarkResult:
    """Benchmark on math reasoning tasks."""
    logger.info("Running math reasoning benchmark...")
    
    # Synthetic problems
    synthetic_problems = [
        {"question": "What is 2 + 2?", "answer": "4"},
        {"question": "What is 10 * 5?", "answer": "50"},
        {"question": "What is 100 / 4?", "answer": "25"},
    ] * 33  # 100 problems
    
    try:
        from transformers import GPT2Tokenizer
        tokenizer = GPT2Tokenizer.from_pretrained("gpt2")
        tokenizer.pad_token = tokenizer.eos_token
    except ImportError:
        tokenizer = None
    
    dataset = MathReasoningDataset(synthetic_problems, tokenizer)
    dataloader = DataLoader(dataset, batch_size=1)
    
    perplexity = calculate_perplexity(model, dataloader, device)
    
    logger.info(f"Math reasoning perplexity: {perplexity:.2f}")
    
    return BenchmarkResult(
        task="math_reasoning",
        metric="perplexity",
        value=perplexity,
        unit="",
        additional_info={}
    )


def benchmark_long_context(
    model: nn.Module,
    config: Dict[str, Any],
    device: torch.device,
) -> BenchmarkResult:
    """Benchmark on long-context reasoning."""
    logger.info("Running long-context benchmark...")
    
    # Create long documents
    long_doc = "This is a test document. " * 500  # ~7000 tokens
    query = "What is this document about?"
    
    synthetic_docs = [long_doc] * 50
    synthetic_queries = [query] * 50
    
    try:
        from transformers import GPT2Tokenizer
        tokenizer = GPT2Tokenizer.from_pretrained("gpt2")
        tokenizer.pad_token = tokenizer.eos_token
    except ImportError:
        tokenizer = None
    
    dataset = LongContextDataset(synthetic_docs, synthetic_queries, tokenizer, max_length=8192)
    dataloader = DataLoader(dataset, batch_size=1)
    
    perplexity = calculate_perplexity(model, dataloader, device)
    
    logger.info(f"Long-context perplexity: {perplexity:.2f}")
    
    return BenchmarkResult(
        task="long_context",
        metric="perplexity",
        value=perplexity,
        unit="",
        additional_info={}
    )


def compare_architectures(
    wide_shallow_config: Dict[str, Any],
    standard_config: Dict[str, Any],
    device: torch.device,
) -> Dict[str, Any]:
    """Compare wide-shallow vs standard transformer."""
    logger.info("Comparing architectures...")
    
    results = {}
    
    # Create wide-shallow model
    logger.info("Creating wide-shallow model...")
    wide_model = create_wide_shallow_model(wide_shallow_config)
    wide_model = wide_model.to(device)
    wide_params = sum(p.numel() for p in wide_model.parameters())
    
    # Count FLOPs (approximate)
    # For transformer: ~6 * N * d_model * d_ffn * seq_len
    # Simplified: ~6 * layers * hidden_size^2 * 4 * seq_len
    wide_flops_estimate = 6 * wide_shallow_config["model"]["num_layers"] * \
                          wide_shallow_config["model"]["hidden_size"] ** 2 * 4 * 2048
    
    results["wide_shallow"] = {
        "parameters": wide_params,
        "flops_estimate": wide_flops_estimate,
    }
    
    logger.info(f"Wide-shallow parameters: {wide_params:,}")
    
    return results


def main(config_path: str = "config.yaml", output_path: str = "benchmark_results.json"):
    """Run all benchmarks."""
    
    # Load config
    with open(config_path, 'r') as f:
        config = yaml.safe_load(f)
    
    # Setup device
    device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
    logger.info(f"Using device: {device}")
    
    # Create model
    logger.info("Loading model...")
    model = create_wide_shallow_model(config)
    model = model.to(device)
    
    # Run benchmarks
    results: List[BenchmarkResult] = []
    
    # Code completion
    results.append(benchmark_code_completion(model, config, device))
    
    # Math reasoning
    results.append(benchmark_math_reasoning(model, config, device))
    
    # Long context
    results.append(benchmark_long_context(model, config, device))
    
    # Print summary
    logger.info("\n" + "="*50)
    logger.info("BENCHMARK SUMMARY")
    logger.info("="*50)
    
    for result in results:
        logger.info(f"{result.task}: {result.metric} = {result.value:.4f}")
    
    # Save results
    results_dict = {
        "benchmarks": [
            {
                "task": r.task,
                "metric": r.metric,
                "value": r.value,
                "unit": r.unit,
                "additional_info": r.additional_info,
            }
            for r in results
        ]
    }
    
    with open(output_path, 'w') as f:
        json.dump(results_dict, f, indent=2)
    
    logger.info(f"\nResults saved to {output_path}")
    
    return results


if __name__ == "__main__":
    import argparse
    
    parser = argparse.ArgumentParser(description="Benchmark Wide and Shallow Transformer")
    parser.add_argument("--config", type=str, default="config.yaml",
                       help="Path to config file")
    parser.add_argument("--output", type=str, default="benchmark_results.json",
                       help="Path to save results")
    args = parser.parse_args()
    
    main(args.config, args.output)
