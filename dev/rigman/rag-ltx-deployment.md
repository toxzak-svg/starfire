# LTX 2.3 Image-to-Video Deployment on RTX 5090 - RAG Knowledge Base

## Overview

This document provides a comprehensive knowledge base for deploying LTX 2.3 (image-to-video generation) on a single NVIDIA RTX 5090 GPU.

---

## 1. Hardware Requirements

### Primary Hardware
| Component | Specification | Notes |
|-----------|---------------|-------|
| GPU | NVIDIA RTX 5090 | 32GB GDDR7, Blackwell architecture |
| System RAM | 32GB+ DDR5 | Buffer memory for video processing |
| Storage | 2TB+ NVMe SSD | Models, temp frames, output videos |
| PSU | 1000W+ | RTX 5090 can draw up to 600W |
| Cooling | High-performance AIO or case airflow | Significant heat output |

### Why RTX 5090 for LTX 2.3?
- **32GB VRAM** - Fits LTX 2.3 comfortably in FP16 without quantization
- **GDDR7** - Faster memory bandwidth for frame generation
- **Blackwell Tensor Cores** - Optimized for transformer inference
- **No quantization needed** - Can run at full FP16 precision

---

## 2. Software Stack

### Core Framework
| Tool | Purpose | URL |
|------|---------|-----|
| ComfyUI | Primary GUI for LTX-Video generation | https://github.com/comfyanonymous/ComfyUI |
| ComfyUI-Manager | Custom node management | https://github.com/ltdrdata/ComfyUI-Manager |

### Model & Dependencies
| Resource | Description | URL |
|----------|-------------|-----|
| LTX-Video | LTX 2.3 video generation model | https://huggingface.co/Lightricks/LTX-Video |
| PyTorch 2.x | Deep learning framework | https://pytorch.org/ |
| CUDA 12.x | GPU compute toolkit | https://developer.nvidia.com/cuda-downloads |
| FFmpeg | Video encoding/output | https://ffmpeg.org/ |

### Optimization Libraries
| Library | Purpose |
|---------|---------|
| Flash Attention | Memory-efficient attention mechanism |
| xFormers | Additional attention optimizations |
| PyTorch compile | Graph optimization for inference |

---

## 3. Deployment Steps

### Step 1: Environment Setup
```bash
# Install CUDA 12.x + cuDNN
# Install PyTorch with CUDA support
pip install torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cu121
```

### Step 2: ComfyUI Installation
```bash
# Clone ComfyUI
git clone https://github.com/comfyanonymous/ComfyUI

# Install dependencies
pip install -r requirements.txt

# Run ComfyUI
python main.py
```

### Step 3: Install Custom Nodes
1. Open ComfyUI
2. Open ComfyUI-Manager
3. Search and install "LTX" or "Video" related nodes
4. Restart ComfyUI

### Step 4: Download Model
- Get LTX-Video weights from HuggingFace
- Place in `ComfyUI/models/` directory
- Verify model loads correctly

### Step 5: Configure Output
- Install FFmpeg for video encoding
- Set up output directory structure

---

## 4. I2V Workflow

### Typical Image-to-Video Pipeline
1. **Load Image** - Upload conditioning frame to ComfyUI
2. **Configure Parameters**:
   - Number of frames
   - FPS (typically 24-30)
   - Guidance scale
   - Seed for reproducibility
3. **Run Inference** - Generate video frames
4. **Compile Video** - Use FFmpeg to combine frames into video

### ComfyUI Workflow Nodes
- Load Image node
- LTX Video Generate node
- Video Combine node
- Save Video node

---

## 5. Optimization & Tuning

### VRAM Optimization
| Technique | When to Use |
|-----------|-------------|
| FP16 (default) | Recommended for RTX 5090 - no quantization needed |
| INT8 Quantization | If memory issues arise |
| CPU Offload | If VRAM still insufficient |

### Performance Tuning
| Parameter | Recommendation |
|-----------|----------------|
| Batch Size | 1 (single image to video) |
| Attention | Flash Attention enabled |
| Compilation | torch.compile() for 10-20% speedup |
| Prefill | Optimize for prefill over decoding |

### Troubleshooting
| Issue | Solution |
|-------|----------|
| OOM Errors | Reduce frame count, check batch size |
| Slow Generation | Enable torch.compile(), use SSD for temp storage |
| Black Output | Check FFmpeg installation, verify model weights |

---

## 6. Related Resources

### Video Generation Models
| Model | Description |
|-------|-------------|
| Stable Video Diffusion | Latent consistency video generation |
| ZeroScope | Consumer GPU optimized |
| ModelScope | Alibaba T2V framework |
| AnimateDiff | SD-based video animation |

### Inference Engines (for other models)
| Engine | Best For |
|--------|----------|
| llama.cpp | CPU+GPU lean inference |
| vLLM | High-throughput serving |
| TensorRT-LLM | NVIDIA-optimized inference |
| Ollama | Easy local deployment |

### Quantization Tools
| Tool | Use Case |
|------|----------|
| AutoGPTQ | GPTQ quantization |
| exllamav2/v3 | Optimized quantized inference |
| GGUF | llama.cpp format |

---

## 7. Reference Links

### Official Resources
- [LTX-Video HuggingFace](https://huggingface.co/Lightricks/LTX-Video)
- [ComfyUI Documentation](https://docs.comfy.org/)
- [NVIDIA TensorRT-LLM](https://github.com/NVIDIA/TensorRT-LLM)

### Community Resources
- [r/LocalLLaMA](https://www.reddit.com/r/LocalLLaMA/) - Optimization tips
- [ComfyUI Discord](https://discord.gg/comfyui)
- [HuggingFace Spaces](https://huggingface.co/spaces) - Demo versions

### Hardware Reviews
- [RTX 5090 TechPowerUp Review](https://www.techpowerup.com/review/nvidia-geforce-rtx-5090-founders-edition)
- [RTX 5090 Tom's Hardware](https://www.tomshardware.com/nvidia/nvidia-rtx-5090-review)

---

## 8. Quick Start Checklist

```
Hardware Setup:
- [ ] RTX 5090 installed
- [ ] 32GB+ system RAM
- [ ] 2TB+ NVMe SSD
- [ ] 1000W+ PSU

Software Setup:
- [ ] CUDA 12.x installed
- [ ] PyTorch 2.x with CUDA
- [ ] ComfyUI installed
- [ ] ComfyUI-Manager installed
- [ ] LTX custom nodes installed
- [ ] FFmpeg installed
- [ ] LTX-Video model downloaded

Testing:
- [ ] Verify GPU detection (nvidia-smi)
- [ ] Test ComfyUI loads
- [ ] Test LTX model loads
- [ ] Generate test video
- [ ] Verify output plays correctly
```

---

## 9. Model Specifications (Reference)

### LTX-Video 2.3
- **Architecture**: Transformer-based video diffusion
- **Input**: Image (conditioning frame) + text prompts
- **Output**: Video frames
- **Context**: Supports various frame lengths
- **Precision**: FP16 recommended on 32GB VRAM

### VRAM Estimates
| Config | VRAM Usage |
|--------|------------|
| FP16 (24 frames) | ~20-28GB |
| FP16 (50 frames) | ~28-32GB |
| INT8 (24 frames) | ~14-18GB |

---

*Last Updated: March 2026*
*For RTX 5090 Single GPU Deployment*
