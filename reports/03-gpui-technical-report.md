# GpUI — GPU Pixel UI Framework
## Technical Report

**Project Path:** `/home/zach/.openclaw/workspace/dev/gpui/`
**Type:** Rust + WebGPU rendering framework
**Language:** Rust (core renderer), WGSL (shaders)
**Status:** Core compiles and renders; not yet wired to window/event loop
**Last Updated:** 2026-04-01

---

## 1. Overview

GpUI is a GPU-native pixel UI renderer built in Rust with WebGPU. The core design principle: **CPU builds a compact element list, GPU does all the rendering work.**

Unlike traditional immediate-mode UIs that iterate widget trees on CPU each frame, GpUI:
1. CPU writes compact `UiElement[]` to a storage buffer
2. GPU compute shader culls, clips, and pixel-snaps elements
3. GPU render shader draws instanced textured quads
4. Screenshot is read back as PNG

This is the architecture of game engines applied to UI — the GPU is the bottleneck for pixels, so that's where computation belongs.

**Key goal:** Headless screenshot capability. No window, no compositor — just capture pixel output programmatically.

---

## 2. Architecture

### 2.1 Three-Layer Pipeline

```
CPU                          GPU Compute                   GPU Render
──────────────────────────    ──────────────────────────    ───────────────────────
UiElement[] ──────────────────→ Instance[] + counts         → Pixel output
(compact, ~80 bytes/elem)      (expanded quads)              (PNG screenshot)
                               culled, clipped, snapped
```

### 2.2 Coordinate System

- **Pixel space, top-left origin**
- 1 unit = 1 pixel
- Positions **snapped to integers** for crisp rendering
- No subpixel blur, no anti-aliasing on geometry

---

## 3. Shader Architecture (WGSL)

### 3.1 Shared Types (Group 0: Viewport Uniform)

```wgsl
struct Viewport {
    size: vec2<f32>,
    inv_size: vec2<f32>,
    scroll: vec2<f32>,
    _pad: vec2<f32>,
};

struct Instance {
    pos: vec2<f32>,
    size: vec2<f32>,
    uv_min: vec2<f32>,
    uv_max: vec2<f32>,
    color: vec4<f32>,
    clip: vec4<f32>,   // x, y, w, h — zero = no clip
    z: f32,
    flags: u32,
    _pad: vec2<f32>,
};
```

### 3.2 Compute Shader: expand()

**Input:** Compact `UiElement[]` storage buffer
**Output:** Expanded `Instance[]` + atomic count

The compute shader:
1. Reads each `UiElement`
2. Applies scroll offset (viewport pan)
3. Snaps position to pixel grid
4. Handles 9-slice sprite expansion
5. Intersects with active clip rect stack
6. Writes `Instance` records to output buffer
7. Increments atomic counter for render pass

**9-slice scaling:** Specified via UV coordinates in `UiElement`. Scale/offset corners preserve edge proportions while stretching center. Essential for button/slider backgrounds.

### 3.3 Render Shader: Vertex + Fragment

**Vertex shader:** Instanced quad rendering — each `Instance` becomes 2 triangles.
**Fragment shader:** Textured or solid color. UVs map to atlas.

---

## 4. Bind Group Layout

Three bind group slots:

| Group | Contents | Used By |
|-------|----------|---------|
| Group 0 | `Viewport` uniform buffer | Compute + Render |
| Group 1 | Storage buffers: elements, instances, indirect count | Compute only |
| Group 1 | `Instance[]` read-only | Render only |
| Group 2 | Atlas texture + sampler | Render only |

---

## 5. Crate Structure

```
gpui/
├── Cargo.toml
├── src/
├── crates/
│   ├── core/                    # WebGPU abstraction layer
│   │   ├── src/device.rs        # Device/queue initialization
│   │   ├── src/buffers.rs       # Storage, uniform, indirect buffers
│   │   ├── src/shaders.rs       # WGSL source (shared + compute + render)
│   │   └── src/pipelines.rs     # Compute + render pipeline state
│   │
│   └── gpui/                    # User-facing UI primitives
│       ├── src/lib.rs           # Public API (Ui, Atlas, MsdfAtlas)
│       ├── src/ui.rs            # Ui builder (rect, text, image, clip)
│       ├── src/atlas.rs          # Texture atlas management
│       ├── src/font.rs          # MSDF font atlas builder
│       └── src/text.rs          # Text layout and rendering
│
└── examples/
    └── demo.rs                  # Basic rendering demo
```

---

## 6. UiBuilder API

```rust
let mut ui = Ui::new(device, queue, width, height).await;

// Red rectangle
ui.rect(10.0, 10.0, 100.0, 50.0, [1.0, 0.0, 0.0, 1.0]);

// Image from atlas
ui.image(atlas_slot, x, y, w, h);

// Text with MSDF font
ui.text("Hello", font, 16.0, x, y, [1.0, 1.0, 1.0, 1.0]);

// Clipped region
ui.push_clip(x, y, w, h);
// ... nested elements ...
ui.pop_clip();

// Render to texture
ui.render();
let png_bytes = ui.screenshot();  // Vec<u8> PNG
```

---

## 7. MSDF Font Atlas

**MSDF = Multi-channel Signed Distance Field**

Instead of raster glyphs, MSDF stores per-pixel distance to the nearest glyph edge. This allows:
- Crisp text at any size
- GPU-side text rendering (no CPU font rasterization)
- Scalable vector-quality text from a single atlas

**Pipeline:**
1. `msdfgen` or similar → generate per-glyph SDF images
2. `ttf-parser` → read font metrics (advance, kerning)
3. Shelf-packing algorithm → pack glyphs into atlas texture
4. GPU fragment shader → evaluates SDF per pixel

**Atlas format:** RGBA8 — R = left edge distance, G = right edge, B = top edge, A = bottom edge. Corner pixels disambiguate (sharp vs rounded).

---

## 8. Atlas Shelf Packing

Glyphs are packed using a **shelf algorithm**:
1. Sort glyphs by height (tallest first)
2. Fill current shelf left-to-right
3. When shelf fills, start new shelf below
4. Track remaining shelf width for fast rejection

This is CPU-side but runs once at startup. Runtime is O(n log n) for sort, O(n) for packing.

---

## 9. Headless Screenshot

The key GpUI feature for RustyParts integration:

```rust
// Render without displaying
ui.render();

// Read back framebuffer as PNG
let png_bytes = ui.read_framebuffer();  // Vec<u8>

// Write to file or serve over HTTP
std::fs::write("output.png", &png_bytes).unwrap();
```

No X11, no Wayland, no display compositor. Just GPU → CPU readback → PNG bytes.

---

## 10. Current Status

### Working ✅
- WebGPU device initialization (wgpu-core)
- WGSL compute + render shaders
- Storage/uniform/indirect buffers
- Pipeline creation (compute + graphics)
- Headless GPU init (no window required)
- MSDF font atlas (builder + GPU upload)
- Shelf-packing atlas algorithm
- 9-slice sprite rendering
- Clip stack (push/pop)

### Not Yet Wired 🔧
- Window surface (needs `winit` or similar for surface management)
- Mouse/keyboard event handling
- Z-ordering (element ordering is implicit; explicit z-layering needed)
- Hot reload of shaders (dev UX)

---

## 11. Comparison to Alternatives

| Framework | Architecture | Text | Headless | Language |
|-----------|-------------|------|----------|----------|
| druid | Immediate-mode CPU | druid-skia | No | Rust |
| iced | Immediate-mode CPU | font-kit | No | Rust |
| egui | Immediate-mode CPU | emath/egui | Yes ( offscreen) | Rust |
| Slint | Retained-mode CPU | owned | No | Rust/C++ |
| **GpUI** | **GPU-driven retained** | **MSDF/GPU** | **Yes (native)** | **Rust/WGSL** |

GpUI is unique in combining GPU-driven rendering + MSDF text + fully headless operation in Rust.

---

## 12. Dependencies

```toml
# crates/core/Cargo.toml
wgpu = "0.19"
anyhow = "1.0"

# crates/gpui/Cargo.toml
# (core as dependency)
```

No `winit` yet — surface management is the missing piece for interactive use.
