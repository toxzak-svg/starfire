# RAG, LoRA, and LLM Inference Resources - Scraped Data

This document contains RAG (Retrieval Augmented Generation), LoRA (Low-Rank Adaptation), and LLM Inference/Deployment/Optimization data scraped from various GitHub repositories and resources.

---

## LLM Inference & Deployment Resources

### 1. ktransformers
**URL:** https://github.com/kvcache-ai/ktransformers
**Description:** A Flexible Framework for Experiencing Heterogeneous LLM Inference/Fine-tune Optimizations

### 2. TensorRT-LLM
**URL:** https://github.com/NVIDIA/TensorRT-LLM
**Description:** NVIDIA's high-performance LLM inference engine with easy-to-use Python API

### 3. PowerInfer
**URL:** https://github.com/Tiiny-AI/PowerInfer
**Description:** High-speed Large Language Model Serving for Local Deployment

### 4. llama.cpp
**URL:** https://github.com/ggerganov/llama.cpp
**Description:** Canonical reference for CPU/GPU-lean local inference

### 5. vLLM
**URL:** https://github.com/vllm-project/vllm
**Description:** High-throughput and memory-efficient LLM inference engine

### 6. Ollama
**URL:** https://github.com/ollama/ollama
**Description:** Get up and running with Llama 3.3, Mistral, Gemma 2, and other large language models

### 7. Intel IPEX-LLM
**URL:** https://github.com/intel/ipex-llm
**Description:** Accelerate local LLM inference and finetuning on Intel XPU

### 8. AutoGPTQ
**URL:** https://github.com/AutoGPTQ/AutoGPTQ
**Description:** Easy-to-use LLMs quantization package with user-friendly APIs

### 9. exllamav2 / exllamav3
**URL:** https://github.com/turboderp-org/exllamav3
**Description:** Optimized quantization and inference for running LLMs on consumer GPUs

### 10. RWKV.cpp
**URL:** https://github.com/RWKV/rwkv.cpp
**Description:** INT4/INT5/INT8 and FP16 inference on CPU for RWKV language models

---

## LoRA (Low-Rank Adaptation) Resources

### 1. PEFT (Hugging Face)
**URL:** https://github.com/huggingface/peft
**Description:** State-of-the-art Parameter-Efficient Fine-Tuning methods - LoRA, QLoRA support

### 2. torchtune (PyTorch)
**URL:** https://github.com/pytorch/torchtune
**Description:** Native-PyTorch Library for LLM Fine-tuning with LoRA and QLoRA support

### 3. Axolotl
**URL:** https://github.com/axolotl-ai-cloud/axolotl
**Description:** Free and open-source LLM fine-tuning framework

### 4. Unsloth
**URL:** https://github.com/unslothai/unsloth
**Description:** 2x faster training, 70% less memory - LoRA, QLoRA, RSlora support

### 5. TRL (Hugging Face)
**URL:** https://github.com/huggingface/trl
**Description:** Full stack library for transformer training with RL - SFT, DPO, GRPO

### 6. LlamaFactory
**URL:** https://github.com/hiyouga/LlamaFactory
**Description:** Unified Efficient Fine-Tuning of 100+ LLMs & VLMs

---

## RAG (Retrieval Augmented Generation) Resources

### 1. RAGAS
**URL:** https://github.com/explodinggradients/ragas
**Description:** Framework for evaluating RAG pipelines

### 2. AutoRAG
**URL:** https://github.com/Marker-Inc-Korea/AutoRAG
**Description:** AutoML tool for automatically finding optimal RAG pipelines

### 3. LazyLLM
**URL:** https://github.com/LazyAGI/LazyLLM
**Description:** Open-source LLM app for building multi-agent applications

### 4. Haystack
**URL:** https://github.com/deepset-ai/haystack
**Description:** Open-source AI orchestration framework for building context-engineered LLM applications

### 5. Chroma
**URL:** https://github.com/chroma-core/chroma
**Description:** Open-source search and retrieval database for AI applications

---

## Video Generation & Audio Resources

### Video Generation (ComfyUI, LTX 2.3, RTX 5090)

| Resource | Description | URL |
|----------|-------------|-----|
| ComfyUI | GUI for running Stable Diffusion and AI generation models - modular workflow system | https://github.com/comfyanonymous/ComfyUI |
| ComfyUI-Manager | Manager for ComfyUI to install and update custom nodes | https://github.com/ltdrdata/ComfyUI-Manager |
| LTX-Video | LTX 2.3 video generation model - high-quality video synthesis from images/text | https://huggingface.co/Lightricks/LTX-Video |
| Stable Video Diffusion | Open-source video generation model for latent consistency | https://github.com/Stability-AI/generative-models |
| ZeroScope | Open-weight video generation model optimized for consumer GPUs | https://github.com/cerspense/zeroscope |
| ModelScope | Alibaba's text-to-video generation framework | https://github.com/modelscope/modelscope |
| AnimateDiff | Tool for animating Stable Diffusion models to generate videos | https://github.com/guoyww/AnimateDiff |
| Deforum | AI art generation tool with video animation capabilities | https://github.com/deforum-art/deforum-for-automatic1111-webui |
| RTX 5090 Video Benchmarks | Video generation performance benchmarks on RTX 5090 | https://www.techpowerup.com/review/nvidia-geforce-rtx-5090-founders-edition |
| RTX 5090 AI Video Performance | Analysis of RTX 5090 for AI video generation workloads | https://www.tomshardware.com/nvidia/nvidia-rtx-5090-review |

### Audio Generation

| Resource | Description | URL |
|----------|-------------|-----|
| AudioGen | Facebook's audio generation model for sound effects | https://github.com/facebookresearch/audiocraft |
| MusicGen | Facebook's music generation model based on transformers | https://github.com/facebookresearch/audiocraft |
| Riffusion | Real-time music generation from Stable Diffusion | https://github.com/riffusion/riffusion |
| EnCodec | Meta's high-fidelity audio codec and generation | https://github.com/facebookresearch/encodec |
| Voicebox | Meta's text-to-speech and audio generation model | https://github.com/facebookresearch/voicebox |
| Bark | Open-source text-to-speech and audio generation | https://github.com/suno-ai/bark |
| ElevenLabs | High-quality voice cloning and text-to-speech API | https://elevenlabs.io/ |
| XTTS | Coqui's multilingual text-to-speech model | https://github.com/coqui-ai/TTS |

---

## Additional Topics Coverage

### Benchmarking Tools

| Resource | Description | URL |
|----------|-------------|-----|
| Evidently AI LLM Benchmark Catalog | Catalog of ~30 LLM benchmarks with taxonomy | https://www.evidentlyai.com/llm-guide/llm-benchmarks |
| Deepchecks LLM Evaluation Tools | Best 10 LLM Evaluation Tools in 2025 | https://deepchecks.com/llm-evaluation/best-tools/ |
| BerriAI LLM Benchmark GitHub | Curated GitHub list of LLM benchmarks | https://github.com/BerriAI/llm-benchmark |

### MLOps & Observability

| Resource | Description | URL |
|----------|-------------|-----|
| lakeFS MLOps Tools 2026 | Covers experiment tracking, feature stores, CI/CD, model monitoring | https://lakefs.io/mlops/mlops-tools/ |
| Coaxsoft MLOps Methods and Tools | Detailed guide on MLOps methods and tools | https://coaxsoft.com/blog/mlops-methods-and-tools |
| Anaconda MLOps Tools Guide | 2025 Buyer's Guide with observability section | https://www.anaconda.com/guides/the-top-mlops-tools |

### Distributed Training

| Resource | Description | URL |
|----------|-------------|-----|
| Kindatechnical Distributed Training Case Study | Practical architecture for multi-GPU/multi-node | https://kindatechnical.com/distributed-deep-learning/case-study-distributed-training-at-scale.html |
| ODSC PyTorch Ray Distributed Training | Distributed training with PyTorch Distributed and Ray | https://odsc.ai/speakers-portfolio/distributed-training-from-scratch-using-pytorch-and-ray/ |

---

*Generated from scraping GitHub repositories and documentation*
*Date: 2025-01*
