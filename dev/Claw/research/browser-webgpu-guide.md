# Running SmolLM3 in Browser with WebGPU

## The Reality

**Fine-tuning in browser = Very Limited**
- WebGPU can run inference
- But training/ fine-tuning needs more memory than browser provides
- You'd need WebGPU compute shaders (experimental)

**Inference in browser = Works!**
- Using Transformers.js
- Can run small models (135M-360M)
- Uses your GPU via WebGPU

---

## What's Available

### 1. Browser Inference (Works Now!)
**File:** `browser-smollm.html`

- Opens in any modern browser (Chrome best)
- Uses WebGPU if available, falls back to WASM
- Runs SmolLM 135M (smallest)
- Works on: Mac, Windows, Linux with GPU

**To use:**
1. Open `browser-smollm.html` in Chrome
2. Wait for model to load
3. Chat!

### 2. Kaggle Fine-tuning (For Training)
**File:** `kaggle-finetune-smollm3.ipynb`

- Upload to Kaggle
- Use free GPU (T4, P100)
- Fine-tune on your data
- Download and convert to GGUF

---

## WebGPU Support

| Browser | WebGPU | Notes |
|---------|--------|-------|
| Chrome 113+ | Yes | Best support |
| Edge 113+ | Yes | Same as Chrome |
| Firefox | Behind flag | Enable webgpu.enabled |
| Safari 17.4+ | Partial | Mac only |

**Check:** Go to `chrome://gpu` in Chrome

---

## Limitations

- **Fine-tuning:** Can't do in browser (not enough memory)
- **Model size:** Browser can only run 135M-360M params
- **Quantization:** Must use Q4 or smaller

---

## Recommendation

1. **Use Kaggle** to fine-tune (free GPU)
2. **Download** the fine-tuned model
3. **Convert** to GGUF using llama.cpp
4. **Run locally** with Ollama or in browser for inference
