# car-small Rust Port — Initial Implementation Plan
# Location: projects/starfire/lib/car_small/

## Architecture to implement

```
car-small: 840K params
├── embed:        Embedding(65, 256)
├── layers ×4:    SSMLayer(256, d_state=512, d_ss=32)
│   ├── ssm_proj: Linear(256→512) → Linear(512→32)
│   ├── ssm_out:  Linear(32→256)
│   ├── cheap:    Linear(256→64) → ReLU → Linear(64→256)
│   └── norm:     RMSNorm(256)
├── car_state:    LSTM(256, hidden=512, batch_first=True)
├── car_predict:  Sequential(Linear(512→512), ReLU, Linear(512→2))
├── norm_f:       RMSNorm(512)
└── lm_head:      Linear(256, 65)

Output: (steering, throttle) — 2 floats
Input:  65-dim token (one-hot or embedded sensor vector)
```

## Rust Crate Dependencies

```toml
# projects/starfire/lib/car_small/Cargo.toml
[package]
name = "car_small"
version = "0.1.0"
edition = "2021"

[dependencies]
ndarray = "0.16"        # N-dimensional arrays
ndarray-rand = "0.15"  # Random initialization
rand = "0.8"           # Random numbers (from workspace deps)
rand_distr = "0.4"     # Distributions
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"

[dev-dependencies]
criterion = "0.5"
```

## Key Implementation Decisions

### 1. RMSNorm
```rust
// RMSNorm(x) = x / rms(x) * weight
// rms(x) = sqrt(mean(x^2))
fn rms_norm(x: &Array2<f32>, weight: &Array1<f32>) -> Array2<f32> {
    let scale = (x.mapv(|v| v * v).mean() + 1e-8).sqrt();
    let scale = scale.reciprocal();
    x * scale * weight
}
```

### 2. SSM Projection (Mamba-style)
```rust
// Projection with SSM-like gated dynamics
fn ssm_proj(x: &Array2<f32>) -> (Array2<f32>, Array2<f32>) {
    // x: (batch, 256)
    let ssm_hidden = self.proj1(x); // (batch, 512)
    let ss_state = self.proj2(&ssm_hidden); // (batch, 32) — SSM state
    (ssm_hidden, ss_state)
}
```

### 3. INT8 Quantization
Pure Rust, no external ML framework needed:
```rust
fn quantize_int8(weights: &Array2<f32>) -> (Array2<i8>, f32) {
    let scale = weights.fold(0.0f32, |m, &v| m.max(v.abs()));
    let scale = scale / 127.0;
    let q = weights.mapv(|v| ((v / scale).round() as i8).clamp(-128, 127));
    (q, scale)
}

fn dequantize_int8(q: &Array2<i8>, scale: f32) -> Array2<f32> {
    q.mapv(|v| v as f32 * scale)
}
```

### 4. Input Format
car-small was trained on DonkeyCar data — 65-dim sensor vector per timestep:
- steering: continuous [-1, 1]
- throttle: continuous [0, 1]
- gyro: (x, y, z)
- camera: not used in this small model version

For inference: one-hot encode sensor bucket OR use raw float vector after normalization.

## Testing Plan

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_forward_pass() {
        let model = CarSmall::random();
        let input = Array2::zeros((1, 65)); // one-hot at token 0
        let (action, _) = model.forward(&input);
        assert_eq!(action.len(), 2);
    }
    
    #[test]
    fn test_int8_roundtrip() {
        let weights = Array2::random((256, 512), Uniform(-1.0, 1.0));
        let (q, scale) = quantize_int8(&weights);
        let dq = dequantize_int8(&q, scale);
        let error = (&weights - &dq).mapv(|v| v * v).mean();
        assert!(error < 0.01); // less than 1% MSE
    }
}
```

## File Structure

```
lib/car_small/
├── Cargo.toml
├── src/
│   ├── lib.rs          # public API
│   ├── embed.rs        # embedding layer
│   ├── ssm_layer.rs    # SSM block
│   ├── lstm.rs         # LSTM car_state
│   ├── predict.rs      # prediction head
│   ├── model.rs        # full CarSmall model
│   └── quant.rs        # INT8/INT4 quantization
├── tests/
│   └── test_model.rs   # integration tests
└── README.md
```

## Connection to Quanot

car-small's SSM layers are **learned** state space models — they learn the dynamics directly from data. Quanot's ESN uses **fixed random** reservoir weights.

**Potential cross-pollination:**
- Replace Quanot's fixed ESN reservoir with car-small's learned SSM
- OR use car-small's SSM as the "creative oscillation" mechanism in Quanot's creativity module
- The CAR (Conditional Activation Reweighting) from phi-2 is conceptually similar: route activations through cheap path when error is low

This would give Quanot learned rather than random temporal dynamics.
