//! Character-level RNN Model
//!
//! Simple LSTM-based character language model.
//! Pure Rust implementation — no external ML frameworks.

use std::f32::consts::E;
use std::io::{Read, Write};
use rand::Rng;

/// Model configuration
#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub vocab_size: usize,
    pub embedding_dim: usize,
    pub hidden_size: usize,
    pub num_layers: usize,
    pub dropout: f32,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            vocab_size: 227, // From vocabulary
            embedding_dim: 64,
            hidden_size: 256,
            num_layers: 2,
            dropout: 0.1,
        }
    }
}

/// Maximum accepted config values when loading a `CharRNN` checkpoint.
///
/// These are generous: real charRNNs used by this project sit comfortably
/// below these ceilings (vocab ~227, embed 64, hidden 256, layers 2), and
/// a typical small SLM stays well under them too. The bounds exist to
/// **reject garbage files** — e.g. a Python pickle that was misnamed with
/// a `.pt` extension and would otherwise be read as if its first 4 bytes
/// were `vocab_size`. Without this guard, the loader's `Self::new(config)`
/// would try to allocate `vocab_size * embedding_dim` floats (~petabytes
/// for a pickle header) before any actual data was read.
const MAX_VOCAB_SIZE: usize = 1_000_000;       // 1M tokens — covers BPE/word-level
const MAX_EMBEDDING_DIM: usize = 4096;         // 4K — well past GPT-3 style
const MAX_HIDDEN_SIZE: usize = 16_384;         // 16K — covers any plausible LSTM
const MAX_NUM_LAYERS: usize = 16;              // 16 stacked LSTMs is already extreme

/// Maximum single-vector length when reading weights (in floats).
///
/// One weight tensor in a charRNN is at most a few hundred thousand
/// floats. 100M floats = 400MB is a generous absolute ceiling. Anything
/// above this is unambiguously corrupt (or a malicious file claiming a
/// 4GB+ tensor in an 11MB checkpoint).
const MAX_VEC_LEN: usize = 100_000_000;

/// Offsets and sizes for the 8 weight/bias tensors in an LSTM cell's flat
/// weight buffer. Layout (contiguous):
///   [i_w | i_b | f_w | f_b | c_w | c_b | o_w | o_b]
/// All four weight tensors share the same shape (hidden_size × total_in), and
/// all four bias tensors share shape (hidden_size,), so we only need offsets
/// to slice into the flat buffer. The on-disk save format mirrors this
/// order so the wire format is preserved across the storage refactor.
#[derive(Clone, Copy, Debug)]
struct LSTMLayout {
    i_w_off: usize,
    i_b_off: usize,
    f_w_off: usize,
    f_b_off: usize,
    c_w_off: usize,
    c_b_off: usize,
    o_w_off: usize,
    o_b_off: usize,
    /// Total length of the flat buffer (= o_b_off + bias_len).
    total_len: usize,
}

impl LSTMLayout {
    fn compute(hidden_size: usize, input_size: usize) -> Self {
        let total_in = input_size + hidden_size;
        let w_len = hidden_size * total_in;
        let b_len = hidden_size;
        let i_w_off = 0;
        let i_b_off = i_w_off + w_len;
        let f_w_off = i_b_off + b_len;
        let f_b_off = f_w_off + w_len;
        let c_w_off = f_b_off + b_len;
        let c_b_off = c_w_off + w_len;
        let o_w_off = c_b_off + b_len;
        let o_b_off = o_w_off + w_len;
        Self {
            i_w_off,
            i_b_off,
            f_w_off,
            f_b_off,
            c_w_off,
            c_b_off,
            o_w_off,
            o_b_off,
            total_len: o_b_off + b_len,
        }
    }
}

/// A single LSTM cell.
///
/// Weights are stored as a single contiguous `Vec<f32>` matching
/// [`LSTMLayout`]. This trades 8 small per-tensor allocations for one
/// contiguous buffer, which lets:
///   - [`LSTMCell::forward`] slice without copying (just borrow a region)
///   - [`crate::language_model::train::train`] apply gradients in a single
///     fused loop per cell (auto-vectorization friendly)
///   - the on-disk save format stay wire-compatible (slice + write per region)
#[derive(Clone)]
struct LSTMCell {
    weights: Vec<f32>,
    layout: LSTMLayout,
}

impl LSTMCell {
    fn new(input_size: usize, hidden_size: usize) -> Self {
        let mut rng = rand::thread_rng();
        let total = input_size + hidden_size;
        let scale = f32::sqrt(2.0 / total as f32);
        let layout = LSTMLayout::compute(hidden_size, input_size);

        let mut weights = Vec::with_capacity(layout.total_len);
        let w_len = hidden_size * total;
        let b_len = hidden_size;

        // i_w (random)
        weights.extend((0..w_len).map(|_| rng.gen_range(-scale..scale)));
        // i_b (zeros)
        weights.extend(std::iter::repeat(0.0_f32).take(b_len));
        // f_w (random)
        weights.extend((0..w_len).map(|_| rng.gen_range(-scale..scale)));
        // f_b (ones — forget bias = 1)
        weights.extend(std::iter::repeat(1.0_f32).take(b_len));
        // c_w (random)
        weights.extend((0..w_len).map(|_| rng.gen_range(-scale..scale)));
        // c_b (zeros)
        weights.extend(std::iter::repeat(0.0_f32).take(b_len));
        // o_w (random)
        weights.extend((0..w_len).map(|_| rng.gen_range(-scale..scale)));
        // o_b (zeros)
        weights.extend(std::iter::repeat(0.0_f32).take(b_len));

        debug_assert_eq!(weights.len(), layout.total_len);

        LSTMCell { weights, layout }
    }

    // ----- Immutable slice accessors (used by forward / save) -----
    fn i_weight(&self) -> &[f32] {
        let len = self.i_weight_len();
        &self.weights[self.layout.i_w_off..self.layout.i_w_off + len]
    }
    fn i_bias(&self) -> &[f32] {
        let len = self.i_bias_len();
        &self.weights[self.layout.i_b_off..self.layout.i_b_off + len]
    }
    fn f_weight(&self) -> &[f32] {
        let len = self.f_weight_len();
        &self.weights[self.layout.f_w_off..self.layout.f_w_off + len]
    }
    fn f_bias(&self) -> &[f32] {
        let len = self.f_bias_len();
        &self.weights[self.layout.f_b_off..self.layout.f_b_off + len]
    }
    fn c_weight(&self) -> &[f32] {
        let len = self.c_weight_len();
        &self.weights[self.layout.c_w_off..self.layout.c_w_off + len]
    }
    fn c_bias(&self) -> &[f32] {
        let len = self.c_bias_len();
        &self.weights[self.layout.c_b_off..self.layout.c_b_off + len]
    }
    fn o_weight(&self) -> &[f32] {
        let len = self.o_weight_len();
        &self.weights[self.layout.o_w_off..self.layout.o_w_off + len]
    }
    fn o_bias(&self) -> &[f32] {
        let len = self.o_bias_len();
        &self.weights[self.layout.o_b_off..self.layout.o_b_off + len]
    }

    // ----- Mutable slice accessors (used by backward / save) -----
    fn i_weight_mut(&mut self) -> &mut [f32] {
        let len = self.i_weight_len();
        &mut self.weights[self.layout.i_w_off..self.layout.i_w_off + len]
    }
    fn i_bias_mut(&mut self) -> &mut [f32] {
        let len = self.i_bias_len();
        &mut self.weights[self.layout.i_b_off..self.layout.i_b_off + len]
    }
    fn f_weight_mut(&mut self) -> &mut [f32] {
        let len = self.f_weight_len();
        &mut self.weights[self.layout.f_w_off..self.layout.f_w_off + len]
    }
    fn f_bias_mut(&mut self) -> &mut [f32] {
        let len = self.f_bias_len();
        &mut self.weights[self.layout.f_b_off..self.layout.f_b_off + len]
    }
    fn c_weight_mut(&mut self) -> &mut [f32] {
        let len = self.c_weight_len();
        &mut self.weights[self.layout.c_w_off..self.layout.c_w_off + len]
    }
    fn c_bias_mut(&mut self) -> &mut [f32] {
        let len = self.c_bias_len();
        &mut self.weights[self.layout.c_b_off..self.layout.c_b_off + len]
    }
    fn o_weight_mut(&mut self) -> &mut [f32] {
        let len = self.o_weight_len();
        &mut self.weights[self.layout.o_w_off..self.layout.o_w_off + len]
    }
    fn o_bias_mut(&mut self) -> &mut [f32] {
        let len = self.o_bias_len();
        &mut self.weights[self.layout.o_b_off..self.layout.o_b_off + len]
    }

    // ----- Length helpers (so tests / num_params can keep counting tensors) -----
    fn i_weight_len(&self) -> usize {
        self.layout.i_b_off - self.layout.i_w_off
    }
    fn i_bias_len(&self) -> usize {
        self.layout.f_w_off - self.layout.i_b_off
    }
    fn f_weight_len(&self) -> usize {
        self.layout.f_b_off - self.layout.f_w_off
    }
    fn f_bias_len(&self) -> usize {
        self.layout.c_w_off - self.layout.f_b_off
    }
    fn c_weight_len(&self) -> usize {
        self.layout.c_b_off - self.layout.c_w_off
    }
    fn c_bias_len(&self) -> usize {
        self.layout.o_w_off - self.layout.c_b_off
    }
    fn o_weight_len(&self) -> usize {
        self.layout.o_b_off - self.layout.o_w_off
    }
    fn o_bias_len(&self) -> usize {
        self.layout.total_len - self.layout.o_b_off
    }

    fn forward(
        &self,
        x: &[f32],
        h: &[f32],
        c: &[f32],
    ) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
        let hidden_size = h.len();
        let input_size = x.len();
        let total = input_size + hidden_size;

        // Stack [x; h] on the stack (tiny, fits in L1). Avoids a heap alloc
        // for the per-call `combined` buffer in the original implementation.
        let mut combined = Vec::with_capacity(total);
        combined.extend_from_slice(x);
        combined.extend_from_slice(h);

        let i_w = self.i_weight();
        let f_w = self.f_weight();
        let c_w = self.c_weight();
        let o_w = self.o_weight();
        let i_b = self.i_bias();
        let f_b = self.f_bias();
        let c_b = self.c_bias();
        let o_b = self.o_bias();

        let i_pre = mat_mul(i_w, &combined, hidden_size, total);
        let f_pre = mat_mul(f_w, &combined, hidden_size, total);
        let c_pre = mat_mul(c_w, &combined, hidden_size, total);
        let o_pre = mat_mul(o_w, &combined, hidden_size, total);

        let mut input_gate = Vec::with_capacity(hidden_size);
        let mut f = Vec::with_capacity(hidden_size);
        let mut c_tilde = Vec::with_capacity(hidden_size);
        let mut o = Vec::with_capacity(hidden_size);

        for i in 0..hidden_size {
            input_gate.push(sigmoid(i_pre[i] + i_b[i]));
            f.push(sigmoid(f_pre[i] + f_b[i]));
            c_tilde.push(f32::tanh(c_pre[i] + c_b[i]));
            o.push(sigmoid(o_pre[i] + o_b[i]));
        }

        let mut c_new = Vec::with_capacity(hidden_size);
        let mut h_new = Vec::with_capacity(hidden_size);

        for idx in 0..hidden_size {
            let c_val = f[idx] * c[idx] + input_gate[idx] * c_tilde[idx];
            c_new.push(c_val);
            h_new.push(o[idx] * f32::tanh(c_val));
        }

        (h_new, c_new, o)
    }
}

/// Character-level RNN model
#[derive(Clone)]
pub struct CharRNN {
    config: ModelConfig,
    /// Embedding layer: (vocab_size, embedding_dim)
    embedding: Vec<f32>,
    /// LSTM layers
    lstm: Vec<LSTMCell>,
    /// Output projection: (hidden_size, vocab_size)
    output_weight: Vec<f32>,
    output_bias: Vec<f32>,
    /// Hidden state cache
    hidden: Vec<Vec<f32>>,
    cell: Vec<Vec<f32>>,
}

/// Stored activations for an entire sequence, in a flat structure-of-arrays
/// layout. Instead of `Vec<Vec<LayerActivations>>` (which allocates ~9 small
/// `Vec<f32>`s per timestep per layer — ~36,000 allocations for a 128-token
/// sequence with 2 layers — and fragments the heap), we pre-allocate **7
/// contiguous buffers** sized to `seq_len * num_layers * hidden_size` (or
/// the per-layer `total = input_size + hidden_size` for `combined`) and one
/// buffer of `seq_len * vocab_size` for the output logits.
///
/// Indexing is by `(timestep, layer_idx)` via `slot_idx`; backward pass
/// accesses the same buffers through the corresponding `_at` accessors.
pub struct SequenceActivations {
    /// [timestep * num_layers + layer_idx][..layer_total]
    combined: Vec<f32>,
    /// [timestep * num_layers + layer_idx][..hidden_size] — post-activation
    /// gate values (the only ones used by backward; pre-activations are
    /// deliberately dropped as dead storage).
    i: Vec<f32>,
    f: Vec<f32>,
    c_tilde: Vec<f32>,
    o: Vec<f32>,
    /// Cell and hidden states after the LSTM update
    c: Vec<f32>,
    h: Vec<f32>,
    /// [timestep][..vocab_size]
    output_logits: Vec<f32>,

    // Layout metadata (constant for the lifetime of this SequenceActivations).
    num_layers: usize,
    hidden_size: usize,
    vocab_size: usize,
    /// Per-layer `input_size + hidden_size` (the stride of `combined` at that
    /// layer). Stored once so `_at` accessors don't recompute.
    layer_totals: Vec<usize>,
}

impl SequenceActivations {
    fn new(seq_len: usize, config: &ModelConfig) -> Self {
        let hidden_size = config.hidden_size;
        let vocab_size = config.vocab_size;
        let num_layers = config.num_layers;
        let layer_totals: Vec<usize> = (0..num_layers)
            .map(|l| {
                let input_size = if l == 0 {
                    config.embedding_dim
                } else {
                    config.hidden_size
                };
                input_size + hidden_size
            })
            .collect();

        let slots = seq_len * num_layers;
        let max_layer_total = *layer_totals.iter().max().unwrap_or(&1);
        SequenceActivations {
            // `combined` is allocated using the max layer total so a single
            // stride works for all layers at all timesteps. The extra space
            // for layer-0 slots (when layer 1 is wider) is wasted but small.
            combined: vec![0.0; slots * max_layer_total],
            i: vec![0.0; slots * hidden_size],
            f: vec![0.0; slots * hidden_size],
            c_tilde: vec![0.0; slots * hidden_size],
            o: vec![0.0; slots * hidden_size],
            c: vec![0.0; slots * hidden_size],
            h: vec![0.0; slots * hidden_size],
            output_logits: vec![0.0; seq_len * vocab_size],
            num_layers,
            hidden_size,
            vocab_size,
            layer_totals,
        }
    }

    #[inline]
    fn slot_idx(&self, t: usize, layer_idx: usize) -> usize {
        t * self.num_layers + layer_idx
    }

    #[inline]
    fn max_layer_total(&self) -> usize {
        // Safe: `layer_totals` is non-empty for any real model.
        *self
            .layer_totals
            .iter()
            .max()
            .expect("layer_totals non-empty")
    }

    // ----- `combined` accessors (variable stride per layer) -----
    fn combined_at(&self, t: usize, layer_idx: usize) -> &[f32] {
        let s = self.slot_idx(t, layer_idx);
        let total = self.layer_totals[layer_idx];
        let stride = self.max_layer_total();
        &self.combined[s * stride..s * stride + total]
    }
    fn combined_at_mut(&mut self, t: usize, layer_idx: usize) -> &mut [f32] {
        let s = self.slot_idx(t, layer_idx);
        let total = self.layer_totals[layer_idx];
        let stride = self.max_layer_total();
        &mut self.combined[s * stride..s * stride + total]
    }

    // ----- `i, f, c_tilde, o, c, h` accessors (uniform `hidden_size` stride) -----
    fn i_at(&self, t: usize, layer_idx: usize) -> &[f32] {
        let s = self.slot_idx(t, layer_idx);
        let stride = self.hidden_size;
        &self.i[s * stride..(s + 1) * stride]
    }
    fn i_at_mut(&mut self, t: usize, layer_idx: usize) -> &mut [f32] {
        let s = self.slot_idx(t, layer_idx);
        let stride = self.hidden_size;
        &mut self.i[s * stride..(s + 1) * stride]
    }
    fn f_at(&self, t: usize, layer_idx: usize) -> &[f32] {
        let stride = self.hidden_size;
        let s = self.slot_idx(t, layer_idx);
        &self.f[s * stride..(s + 1) * stride]
    }
    fn f_at_mut(&mut self, t: usize, layer_idx: usize) -> &mut [f32] {
        let stride = self.hidden_size;
        let s = self.slot_idx(t, layer_idx);
        &mut self.f[s * stride..(s + 1) * stride]
    }
    fn c_tilde_at(&self, t: usize, layer_idx: usize) -> &[f32] {
        let stride = self.hidden_size;
        let s = self.slot_idx(t, layer_idx);
        &self.c_tilde[s * stride..(s + 1) * stride]
    }
    fn c_tilde_at_mut(&mut self, t: usize, layer_idx: usize) -> &mut [f32] {
        let stride = self.hidden_size;
        let s = self.slot_idx(t, layer_idx);
        &mut self.c_tilde[s * stride..(s + 1) * stride]
    }
    fn o_at(&self, t: usize, layer_idx: usize) -> &[f32] {
        let stride = self.hidden_size;
        let s = self.slot_idx(t, layer_idx);
        &self.o[s * stride..(s + 1) * stride]
    }
    fn o_at_mut(&mut self, t: usize, layer_idx: usize) -> &mut [f32] {
        let stride = self.hidden_size;
        let s = self.slot_idx(t, layer_idx);
        &mut self.o[s * stride..(s + 1) * stride]
    }
    fn c_at(&self, t: usize, layer_idx: usize) -> &[f32] {
        let stride = self.hidden_size;
        let s = self.slot_idx(t, layer_idx);
        &self.c[s * stride..(s + 1) * stride]
    }
    fn c_at_mut(&mut self, t: usize, layer_idx: usize) -> &mut [f32] {
        let stride = self.hidden_size;
        let s = self.slot_idx(t, layer_idx);
        &mut self.c[s * stride..(s + 1) * stride]
    }
    fn h_at(&self, t: usize, layer_idx: usize) -> &[f32] {
        let stride = self.hidden_size;
        let s = self.slot_idx(t, layer_idx);
        &self.h[s * stride..(s + 1) * stride]
    }
    fn h_at_mut(&mut self, t: usize, layer_idx: usize) -> &mut [f32] {
        let stride = self.hidden_size;
        let s = self.slot_idx(t, layer_idx);
        &mut self.h[s * stride..(s + 1) * stride]
    }

    // ----- `output_logits` accessors -----
    pub fn output_logits_at(&self, t: usize) -> &[f32] {
        &self.output_logits[t * self.vocab_size..(t + 1) * self.vocab_size]
    }
    fn output_logits_at_mut(&mut self, t: usize) -> &mut [f32] {
        &mut self.output_logits[t * self.vocab_size..(t + 1) * self.vocab_size]
    }
}

/// Gradient storage for one layer's parameters.
///
/// Mirrors the flat layout of [`LSTMCell`]: a single contiguous `Vec<f32>`
/// in the order [i_w | i_b | f_w | f_b | c_w | c_b | o_w | o_b], indexed
/// via [`LSTMLayout`]. This lets [`CharRNN::apply_gradients`] consume the
/// full buffer in one fused loop.
#[derive(Clone)]
struct LayerGradients {
    weights: Vec<f32>,
    layout: LSTMLayout,
}

impl LayerGradients {
    fn new(hidden_size: usize, input_size: usize) -> Self {
        let layout = LSTMLayout::compute(hidden_size, input_size);
        LayerGradients {
            weights: vec![0.0; layout.total_len],
            layout,
        }
    }

    /// Whole flat buffer — used by `apply_gradients` to read all per-layer
    /// gradients in a single fused loop.
    fn weights(&self) -> &[f32] {
        &self.weights
    }
    fn weights_mut(&mut self) -> &mut [f32] {
        &mut self.weights
    }

}

/// Full gradient set for the model
pub struct ModelGradients {
    embedding: Vec<f32>,
    layers: Vec<LayerGradients>,
    output_weight: Vec<f32>,
    output_bias: Vec<f32>,
}

impl CharRNN {
    pub fn new(config: ModelConfig) -> Self {
        let mut rng = rand::thread_rng();
        
        // Initialize embedding
        let embedding = (0..config.vocab_size * config.embedding_dim)
            .map(|_| rng.gen_range(-0.1..0.1))
            .collect();
        
        // Initialize LSTM layers
        let mut lstm = Vec::new();
        for layer in 0..config.num_layers {
            let input_size = if layer == 0 {
                config.embedding_dim
            } else {
                config.hidden_size
            };
            lstm.push(LSTMCell::new(input_size, config.hidden_size));
        }
        
        // Initialize output layer
        let scale = f32::sqrt(2.0 / config.hidden_size as f32);
        let output_weight = (0..config.hidden_size * config.vocab_size)
            .map(|_| rng.gen_range(-scale..scale))
            .collect();
        let output_bias = vec![0.0; config.vocab_size];
        
        // Hidden state cache (for inference)
        let hidden = vec![vec![0.0; config.hidden_size]; config.num_layers];
        let cell = vec![vec![0.0; config.hidden_size]; config.num_layers];
        
        CharRNN {
            config,
            embedding,
            lstm,
            output_weight,
            output_bias,
            hidden,
            cell,
        }
    }
    
    /// Reset hidden state to zeros
    pub fn reset_hidden(&mut self) {
        for h in &mut self.hidden {
            h.fill(0.0);
        }
        for c in &mut self.cell {
            c.fill(0.0);
        }
    }

    /// Vocabulary size for this model.
    ///
    /// Used by [`crate::language_model::generate::generate`] to filter out
    /// any prompt characters whose index would land outside the embedding
    /// table. Without this guard, loading a checkpoint trained against a
    /// different vocabulary than the runtime `Vocabulary` (e.g. the 11MB
    /// `ckpt_e28_b500.pt` vs the 3.7MB `data/star_model.bin`) would crash
    /// with a slice-bounds panic on the first out-of-range char.
    pub fn vocab_size(&self) -> usize {
        self.config.vocab_size
    }

    /// Forward pass for a single character
    pub fn step(&mut self, char_idx: usize) -> Vec<f32> {
        // Get embedding for this character
        let emb_start = char_idx * self.config.embedding_dim;
        let emb_end = emb_start + self.config.embedding_dim;
        let embedding = &self.embedding[emb_start..emb_end];
        
        // Forward through LSTM layers
        let mut input = embedding.to_vec();
        for (layer_idx, lstm_cell) in self.lstm.iter().enumerate() {
            let (h_new, c_new, _) = lstm_cell.forward(&input, &self.hidden[layer_idx], &self.cell[layer_idx]);
            self.hidden[layer_idx] = h_new;
            self.cell[layer_idx] = c_new;
            input = self.hidden[layer_idx].clone();
        }
        
        // Output projection
        let hidden_last = &self.hidden[self.config.num_layers - 1];
        let mut logits = Vec::with_capacity(self.config.vocab_size);
        for i in 0..self.config.vocab_size {
            let mut sum = self.output_bias[i];
            for j in 0..self.config.hidden_size {
                sum += self.output_weight[j * self.config.vocab_size + i] * hidden_last[j];
            }
            logits.push(sum);
        }
        
        logits
    }
    
    /// Get softmax probabilities for next character
    pub fn predict_next(&mut self, char_idx: usize) -> Vec<f32> {
        let logits = self.step(char_idx);
        softmax(&logits)
    }

    /// Forward pass through entire sequence, storing intermediates for BPTT.
    ///
    /// Pre-allocates one [`SequenceActivations`] (7 contiguous buffers for
    /// per-layer per-timestep activations + 1 for output logits) up front
    /// and writes into it in place. The original implementation allocated
    /// ~9 small `Vec<f32>`s per timestep per layer; for a 128-token
    /// sequence with 2 LSTM layers that's ~2,300 heap allocations per
    /// sequence (≈ 73,000 per batch across the 32 sequences). The flat
    /// SoA layout reduces that to 7 allocations per sequence.
    pub fn forward_sequence(&mut self, sequence: &[usize]) -> SequenceActivations {
        let seq_len = sequence.len();
        let hidden_size = self.config.hidden_size;
        let embedding_dim = self.config.embedding_dim;
        let vocab_size = self.config.vocab_size;
        let num_layers = self.config.num_layers;

        let mut acts = SequenceActivations::new(seq_len, &self.config);

        // Scratch buffer for the per-layer input. Resized to the widest
        // layer (hidden_size); for layer 0 we only use the first
        // `embedding_dim` slots.
        let mut input_buf: Vec<f32> = vec![0.0; hidden_size];

        // Stack-allocated scratch for `combined` (max size = input_size +
        // hidden_size for the widest layer). Layer 0 = 64+256 = 320, layer
        // 1 = 256+256 = 512; we size for the worst case to keep one
        // scratch slot across both layers.
        let mut combined_stack = [0.0f32; 512];
        // Stack-allocated scratch for the 4 gate values (size = hidden_size).
        // We need local copies because each gate's slot is held only during
        // its own computation; the cell-update step then needs all four
        // values simultaneously, which would otherwise re-borrow `acts`.
        let mut i_stack = [0.0f32; 256];
        let mut f_stack = [0.0f32; 256];
        let mut c_tilde_stack = [0.0f32; 256];
        let mut o_stack = [0.0f32; 256];

        for t in 0..seq_len {
            let char_idx = sequence[t];
            // Layer-0 input = embedding[char_idx]
            input_buf[..embedding_dim].copy_from_slice(
                &self.embedding[char_idx * embedding_dim..(char_idx + 1) * embedding_dim],
            );

            for layer_idx in 0..num_layers {
                let input_size = if layer_idx == 0 { embedding_dim } else { hidden_size };
                let total = input_size + hidden_size;

                // Cache immutable cell weight/bias slices once per layer
                // (avoids re-slicing inside the inner gate loops and keeps
                // disjoint borrows visible to the borrow checker).
                let cell = &self.lstm[layer_idx];
                let i_w = cell.i_weight();
                let i_b = cell.i_bias();
                let f_w = cell.f_weight();
                let f_b = cell.f_bias();
                let c_w = cell.c_weight();
                let c_b = cell.c_bias();
                let o_w = cell.o_weight();
                let o_b = cell.o_bias();

                // combined = [input; h_prev]. Fill the local stack scratch once,
                // then mirror it into the activation slot for backward.
                // (Computing once into the stack avoids a redundant
                // slot-then-stack copy.)
                combined_stack[..input_size].copy_from_slice(&input_buf[..input_size]);
                combined_stack[input_size..total].copy_from_slice(&self.hidden[layer_idx]);
                acts.combined_at_mut(t, layer_idx)
                    .copy_from_slice(&combined_stack[..total]);
                let combined = &combined_stack[..total];

                // Gate computations — fused matmul + bias + activation
                // written directly into the post-activation storage slot.
                // Each gate's slot is borrowed only during its own
                // computation; the post-activation value is also copied
                // into the local stack so the subsequent cell-update step
                // (which mutates other slots) doesn't need to re-borrow
                // the gate slots.
                {
                    let i_slot = acts.i_at_mut(t, layer_idx);
                    for i in 0..hidden_size {
                        let mut sum = 0.0f32;
                        let row = &i_w[i * total..(i + 1) * total];
                        for j in 0..total {
                            sum += row[j] * combined[j];
                        }
                        let v = sigmoid(sum + i_b[i]);
                        i_slot[i] = v;
                        i_stack[i] = v;
                    }
                }
                {
                    let f_slot = acts.f_at_mut(t, layer_idx);
                    for i in 0..hidden_size {
                        let mut sum = 0.0f32;
                        let row = &f_w[i * total..(i + 1) * total];
                        for j in 0..total {
                            sum += row[j] * combined[j];
                        }
                        let v = sigmoid(sum + f_b[i]);
                        f_slot[i] = v;
                        f_stack[i] = v;
                    }
                }
                {
                    let c_slot = acts.c_tilde_at_mut(t, layer_idx);
                    for i in 0..hidden_size {
                        let mut sum = 0.0f32;
                        let row = &c_w[i * total..(i + 1) * total];
                        for j in 0..total {
                            sum += row[j] * combined[j];
                        }
                        let v = f32::tanh(sum + c_b[i]);
                        c_slot[i] = v;
                        c_tilde_stack[i] = v;
                    }
                }
                {
                    let o_slot = acts.o_at_mut(t, layer_idx);
                    for i in 0..hidden_size {
                        let mut sum = 0.0f32;
                        let row = &o_w[i * total..(i + 1) * total];
                        for j in 0..total {
                            sum += row[j] * combined[j];
                        }
                        let v = sigmoid(sum + o_b[i]);
                        o_slot[i] = v;
                        o_stack[i] = v;
                    }
                }

                // Cell + hidden update using the local stack gate values.
                // This step mutates `c` and `h` slots. Split into two
                // scopes because the borrow checker treats
                // `acts.c_at_mut(...)` and `acts.h_at_mut(...)` as
                // overlapping borrows on `acts`. We bridge them with a
                // stack-local copy of `c_new` so the second scope can
                // compute h without re-reading the freshly-written slot.
                let mut c_new_stack = [0.0f32; 256];
                {
                    let c_slot = acts.c_at_mut(t, layer_idx);
                    let c_prev = &self.cell[layer_idx];
                    for idx in 0..hidden_size {
                        let c_val = f_stack[idx] * c_prev[idx] + i_stack[idx] * c_tilde_stack[idx];
                        c_slot[idx] = c_val;
                        c_new_stack[idx] = c_val;
                    }
                }
                {
                    let h_slot = acts.h_at_mut(t, layer_idx);
                    for idx in 0..hidden_size {
                        h_slot[idx] = o_stack[idx] * f32::tanh(c_new_stack[idx]);
                    }
                }

                let h_slot = acts.h_at(t, layer_idx);
                self.hidden[layer_idx].copy_from_slice(h_slot);
                self.cell[layer_idx].copy_from_slice(acts.c_at(t, layer_idx));

                // Prepare input for next layer
                if layer_idx < num_layers - 1 {
                    input_buf[..hidden_size].copy_from_slice(h_slot);
                }
            }

            // Output projection
            let hidden_last = &self.hidden[num_layers - 1];
            let logits = acts.output_logits_at_mut(t);
            for i in 0..vocab_size {
                let mut sum = self.output_bias[i];
                for j in 0..hidden_size {
                    sum += self.output_weight[j * vocab_size + i] * hidden_last[j];
                }
                logits[i] = sum;
            }
        }

        acts
    }

    /// Reset hidden state to zeros
    ///
    /// Allocates all per-call scratch buffers (d_logits, dh, dc_tanh,
    /// di_pre/df_pre/dc_tilde_pre/do_pre, dx, dh_prev) once and reuses
    /// them across all timesteps and layers. The original implementation
    /// re-allocated each of these Vecs on every iteration of both the
    /// timestep and layer loops — for a 128-token sequence with 2 LSTM
    /// layers that's ~2,500 heap allocations per sequence (~80,000 per
    /// batch). The refactored version allocates exactly 9 scratch Vecs
    /// per call regardless of sequence length or layer count.
    pub fn backward_sequence(&mut self, sequence: &[usize], activations: &SequenceActivations, target: &[usize]) -> ModelGradients {
        let seq_len = sequence.len();
        let hidden_size = self.config.hidden_size;
        let embedding_dim = self.config.embedding_dim;
        let vocab_size = self.config.vocab_size;
        let num_layers = self.config.num_layers;

        let mut gradients = ModelGradients {
            embedding: vec![0.0; self.config.vocab_size * embedding_dim],
            layers: (0..num_layers).map(|layer_idx| {
                let input_size = if layer_idx == 0 { embedding_dim } else { hidden_size };
                LayerGradients::new(hidden_size, input_size)
            }).collect(),
            output_weight: vec![0.0; hidden_size * vocab_size],
            output_bias: vec![0.0; vocab_size],
        };

        // Persistent carry-over state for dh_next / dc_next across timesteps.
        let dh_next = vec![vec![0.0f32; hidden_size]; num_layers];
        let mut dc_next = vec![vec![0.0f32; hidden_size]; num_layers];

        // Per-call scratch — sized to the widest per-layer requirement
        // and reused across every timestep and every layer.
        let max_input_size = embedding_dim.max(hidden_size);
        let mut d_logits_buf = vec![0.0f32; vocab_size];
        let mut dh_buf = vec![0.0f32; hidden_size];
        let mut dc_tanh_buf = vec![0.0f32; hidden_size];
        let mut di_pre_buf = vec![0.0f32; hidden_size];
        let mut df_pre_buf = vec![0.0f32; hidden_size];
        let mut dc_tilde_pre_buf = vec![0.0f32; hidden_size];
        let mut do_pre_buf = vec![0.0f32; hidden_size];
        let mut dx_buf = vec![0.0f32; max_input_size];
        let mut dh_prev_buf = vec![0.0f32; hidden_size];

        for t in (0..seq_len.saturating_sub(1)).rev() {
            // d_logits = softmax(logits); d_logits[target] -= 1.0
            let logits = activations.output_logits_at(t);
            softmax_into(&mut d_logits_buf, logits);
            let target_idx = target[t];
            d_logits_buf[target_idx] -= 1.0;

            // Output projection gradients
            for i in 0..vocab_size {
                gradients.output_bias[i] += d_logits_buf[i];
            }
            let h_last = activations.h_at(t, num_layers - 1);
            for j in 0..hidden_size {
                for i in 0..vocab_size {
                    gradients.output_weight[j * vocab_size + i] += h_last[j] * d_logits_buf[i];
                }
            }

            // dh = output_weight^T * d_logits (initial for this timestep)
            for j in 0..hidden_size {
                let mut sum = 0.0f32;
                for i in 0..vocab_size {
                    sum += self.output_weight[j * vocab_size + i] * d_logits_buf[i];
                }
                dh_buf[j] = sum;
            }

            for layer_idx in (0..num_layers).rev() {
                let input_size = if layer_idx == 0 { embedding_dim } else { hidden_size };
                let total = input_size + hidden_size;

                // Cache cell weight slices once per layer
                let cell = &self.lstm[layer_idx];
                let i_w = cell.i_weight();
                let f_w = cell.f_weight();
                let c_w = cell.c_weight();
                let o_w = cell.o_weight();

                // Activations at (t, layer_idx)
                let i_act = activations.i_at(t, layer_idx);
                let f_act = activations.f_at(t, layer_idx);
                let c_tilde_act = activations.c_tilde_at(t, layer_idx);
                let o_act = activations.o_at(t, layer_idx);
                let c_act = activations.c_at(t, layer_idx);
                let combined = activations.combined_at(t, layer_idx);

                // dh += dh_next[layer_idx] (carry-over from next timestep)
                for j in 0..hidden_size {
                    dh_buf[j] += dh_next[layer_idx][j];
                }

                // dc_tanh = dh * o * (1 - tanh(c)^2) + dc_next[layer_idx]
                for idx in 0..hidden_size {
                    let c_val = c_act[idx];
                    let tanh_c = f32::tanh(c_val);
                    let sech_sq = 1.0 - tanh_c * tanh_c;
                    dc_tanh_buf[idx] = dh_buf[idx] * o_act[idx] * sech_sq + dc_next[layer_idx][idx];
                }

                // Pre-activation gradients
                for idx in 0..hidden_size {
                    let sig_i = i_act[idx] * (1.0 - i_act[idx]);
                    let sig_f = f_act[idx] * (1.0 - f_act[idx]);
                    let tanh_deriv = 1.0 - c_tilde_act[idx] * c_tilde_act[idx];
                    let sig_o = o_act[idx] * (1.0 - o_act[idx]);

                    di_pre_buf[idx] = dc_tanh_buf[idx] * c_tilde_act[idx] * sig_i;
                    df_pre_buf[idx] = dc_tanh_buf[idx] * c_act[idx] * sig_f;
                    dc_tilde_pre_buf[idx] = dc_tanh_buf[idx] * i_act[idx] * tanh_deriv;
                    do_pre_buf[idx] = dh_buf[idx] * f32::tanh(c_act[idx]) * sig_o;
                }

                // Accumulate weight gradients into the flat per-layer buffer.
                // Borrow the whole flat buffer once, then slice it into the
                // 8 disjoint tensor regions (the borrow checker accepts
                // multiple mutable slices of the same Vec when they are
                // provably non-overlapping).
                {
                    let lg = &mut gradients.layers[layer_idx];
                    // Copy the layout offsets (Copy type) before mutably
                    // borrowing the weights buffer, so the borrow checker
                    // doesn't think we're re-borrowing `lg`.
                    let lo = lg.layout;
                    let buf = lg.weights_mut();
                    let (i_w_grad, rest) = buf.split_at_mut(lo.i_b_off - lo.i_w_off);
                    let (i_b_grad, rest) = rest.split_at_mut(lo.f_w_off - lo.i_b_off);
                    let (f_w_grad, rest) = rest.split_at_mut(lo.f_b_off - lo.f_w_off);
                    let (f_b_grad, rest) = rest.split_at_mut(lo.c_w_off - lo.f_b_off);
                    let (c_w_grad, rest) = rest.split_at_mut(lo.c_b_off - lo.c_w_off);
                    let (c_b_grad, rest) = rest.split_at_mut(lo.o_w_off - lo.c_b_off);
                    let (o_w_grad, o_b_grad) = rest.split_at_mut(lo.o_b_off - lo.o_w_off);
                    for i in 0..hidden_size {
                        let di = di_pre_buf[i];
                        let df = df_pre_buf[i];
                        let dc = dc_tilde_pre_buf[i];
                        let dop = do_pre_buf[i];
                        for j in 0..total {
                            i_w_grad[i * total + j] += di * combined[j];
                            f_w_grad[i * total + j] += df * combined[j];
                            c_w_grad[i * total + j] += dc * combined[j];
                            o_w_grad[i * total + j] += dop * combined[j];
                        }
                    }
                    for i in 0..hidden_size {
                        i_b_grad[i] += di_pre_buf[i];
                        f_b_grad[i] += df_pre_buf[i];
                        c_b_grad[i] += dc_tilde_pre_buf[i];
                        o_b_grad[i] += do_pre_buf[i];
                    }
                }

                // dx = W[:, :input_size]^T * d_pre  (input-side gradient)
                for j in 0..input_size {
                    let mut sum = 0.0f32;
                    for i in 0..hidden_size {
                        sum += i_w[i * total + j] * di_pre_buf[i];
                        sum += f_w[i * total + j] * df_pre_buf[i];
                        sum += c_w[i * total + j] * dc_tilde_pre_buf[i];
                        sum += o_w[i * total + j] * do_pre_buf[i];
                    }
                    dx_buf[j] = sum;
                }

                // dh_prev = W[:, input_size:]^T * d_pre  (hidden-side gradient)
                for j in 0..hidden_size {
                    let h_j = input_size + j;
                    let mut sum = 0.0f32;
                    for i in 0..hidden_size {
                        sum += i_w[i * total + h_j] * di_pre_buf[i];
                        sum += f_w[i * total + h_j] * df_pre_buf[i];
                        sum += c_w[i * total + h_j] * dc_tilde_pre_buf[i];
                        sum += o_w[i * total + h_j] * do_pre_buf[i];
                    }
                    dh_prev_buf[j] = sum;
                }

                if layer_idx == 0 {
                    for j in 0..embedding_dim {
                        gradients.embedding[sequence[t] * embedding_dim + j] += dx_buf[j];
                    }
                }

                // dh becomes dh_prev for the next layer's iteration (going up).
                // At layer_idx == 0 the loop is about to exit, so skip the copy.
                if layer_idx > 0 {
                    dh_buf.copy_from_slice(&dh_prev_buf);
                }

                // dc_next for this layer (used in next earlier timestep)
                for idx in 0..hidden_size {
                    dc_next[layer_idx][idx] = dc_tanh_buf[idx] * f_act[idx];
                }
            }
        }

        gradients
    }

    /// Apply gradients with clipping and learning rate.
    ///
    /// Uses **one fused loop per contiguous weight buffer** instead of the
    /// original 16 disjoint per-tensor loops. The clip+sub operation is
    /// identical (`g = g.clamp(-clip, clip); w -= lr * g`) but it now
    /// runs over the LSTM layer's flat `weights` buffer in one pass per
    /// layer, letting LLVM auto-vectorize the whole region as a single
    /// SIMD loop rather than 8 separate ones.
    pub fn apply_gradients(&mut self, gradients: &ModelGradients, lr: f32, clip_val: f32) {
        // Embedding — single fused loop over the contiguous embedding buffer
        apply_fused(&mut self.embedding, &gradients.embedding, lr, clip_val);

        // LSTM layers — one fused loop per layer over its contiguous
        // weight buffer (was 8 disjoint per-tensor loops per layer).
        for (cell, layer_grad) in self.lstm.iter_mut().zip(gradients.layers.iter()) {
            apply_fused(&mut cell.weights, layer_grad.weights(), lr, clip_val);
        }

        // Output projection — single fused loop per buffer
        apply_fused(&mut self.output_weight, &gradients.output_weight, lr, clip_val);
        apply_fused(&mut self.output_bias, &gradients.output_bias, lr, clip_val);
    }

    /// Get total parameter count
    pub fn num_params(&self) -> usize {
        let embedding_params = self.config.vocab_size * self.config.embedding_dim;

        // Per-layer input size: layer 0 takes the embedding, all other layers
        // take the previous layer's hidden state. Computing `total` from
        // embedding_dim + hidden_size for every layer under-counts every layer
        // beyond the first (it should be hidden_size + hidden_size there).
        let lstm_params: usize = self.lstm.iter().enumerate().map(|(layer_idx, _cell)| {
            let input_size = if layer_idx == 0 {
                self.config.embedding_dim
            } else {
                self.config.hidden_size
            };
            let total = input_size + self.config.hidden_size;
            4 * self.config.hidden_size * total + 4 * self.config.hidden_size
        }).sum();

        let output_params = self.config.hidden_size * self.config.vocab_size + self.config.vocab_size;

        embedding_params + lstm_params + output_params
    }
    
    /// Save model to binary format
    pub fn save(&self, path: &str) -> std::io::Result<()> {
        use std::io::{BufWriter, Write};

        let file = std::fs::File::create(path)?;
        let mut writer = BufWriter::new(file);

        // Write config as fixed-size types for portability
        writer.write_all(&(self.config.vocab_size as u32).to_le_bytes())?;
        writer.write_all(&(self.config.embedding_dim as u32).to_le_bytes())?;
        writer.write_all(&(self.config.hidden_size as u32).to_le_bytes())?;
        writer.write_all(&(self.config.num_layers as u32).to_le_bytes())?;
        writer.write_all(&self.config.dropout.to_le_bytes())?;

        // Write weights. The on-disk order is the original per-tensor
        // order (i_w, i_b, f_w, f_b, c_w, c_b, o_w, o_b) so existing
        // .bin files remain compatible across the storage refactor.
        write_f32_vec(&mut writer, &self.embedding)?;

        for cell in &self.lstm {
            write_f32_vec(&mut writer, cell.i_weight())?;
            write_f32_vec(&mut writer, cell.i_bias())?;
            write_f32_vec(&mut writer, cell.f_weight())?;
            write_f32_vec(&mut writer, cell.f_bias())?;
            write_f32_vec(&mut writer, cell.c_weight())?;
            write_f32_vec(&mut writer, cell.c_bias())?;
            write_f32_vec(&mut writer, cell.o_weight())?;
            write_f32_vec(&mut writer, cell.o_bias())?;
        }

        write_f32_vec(&mut writer, &self.output_weight)?;
        write_f32_vec(&mut writer, &self.output_bias)?;

        Ok(())
    }

    /// Load model from binary format
    pub fn load(path: &str) -> std::io::Result<Self> {
        use std::io::Read;

        let file = std::fs::File::open(path)?;
        let mut reader = std::io::BufReader::new(file);

        let mut buf = [0u8; 4];

        reader.read_exact(&mut buf)?;
        let vocab_size = u32::from_le_bytes(buf) as usize;

        reader.read_exact(&mut buf)?;
        let embedding_dim = u32::from_le_bytes(buf) as usize;

        reader.read_exact(&mut buf)?;
        let hidden_size = u32::from_le_bytes(buf) as usize;

        reader.read_exact(&mut buf)?;
        let num_layers = u32::from_le_bytes(buf) as usize;

        reader.read_exact(&mut buf)?;
        let dropout = f32::from_le_bytes(buf);

        // Validate config *before* any allocation. Without this guard, a
        // file that is NOT a CharRNN save (e.g. a Python pickle misnamed
        // with a .pt extension) would feed its first 4 bytes into
        // `vocab_size` and then `Self::new` would attempt to allocate
        // petabytes of memory. See `MAX_*` constants for the rationale.
        if vocab_size == 0 || vocab_size > MAX_VOCAB_SIZE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "vocab_size out of range [1, {}]: {} (is this really a CharRNN save?)",
                    MAX_VOCAB_SIZE, vocab_size
                ),
            ));
        }
        if embedding_dim == 0 || embedding_dim > MAX_EMBEDDING_DIM {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "embedding_dim out of range [1, {}]: {} (is this really a CharRNN save?)",
                    MAX_EMBEDDING_DIM, embedding_dim
                ),
            ));
        }
        if hidden_size == 0 || hidden_size > MAX_HIDDEN_SIZE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "hidden_size out of range [1, {}]: {} (is this really a CharRNN save?)",
                    MAX_HIDDEN_SIZE, hidden_size
                ),
            ));
        }
        if num_layers == 0 || num_layers > MAX_NUM_LAYERS {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "num_layers out of range [1, {}]: {} (is this really a CharRNN save?)",
                    MAX_NUM_LAYERS, num_layers
                ),
            ));
        }

        let config = ModelConfig {
            vocab_size,
            embedding_dim,
            hidden_size,
            num_layers,
            dropout,
        };

        let mut model = Self::new(config);

        model.embedding = read_f32_vec(&mut reader)?;

        for cell in &mut model.lstm {
            // Same wire order as save(): i_w, i_b, f_w, f_b, c_w, c_b, o_w, o_b.
            // Each `read_f32_vec` reads one tensor and we copy into the
            // matching slice of the flat weight buffer.
            macro_rules! read_into {
                ($reader:expr, $dst:expr, $name:expr) => {{
                    let v = read_f32_vec($reader)?;
                    if v.len() != $dst.len() {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!(
                                "{} len mismatch: file={} expected={}",
                                $name,
                                v.len(),
                                $dst.len()
                            ),
                        ));
                    }
                    $dst.copy_from_slice(&v);
                }};
            }
            read_into!(&mut reader, cell.i_weight_mut(), "i_weight");
            read_into!(&mut reader, cell.i_bias_mut(), "i_bias");
            read_into!(&mut reader, cell.f_weight_mut(), "f_weight");
            read_into!(&mut reader, cell.f_bias_mut(), "f_bias");
            read_into!(&mut reader, cell.c_weight_mut(), "c_weight");
            read_into!(&mut reader, cell.c_bias_mut(), "c_bias");
            read_into!(&mut reader, cell.o_weight_mut(), "o_weight");
            read_into!(&mut reader, cell.o_bias_mut(), "o_bias");
        }

        model.output_weight = read_f32_vec(&mut reader)?;
        model.output_bias = read_f32_vec(&mut reader)?;

        Ok(model)
    }
}

// Utility functions

fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + E.powf(-x))
}

fn softmax(v: &[f32]) -> Vec<f32> {
    let max = v.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let exps: Vec<f32> = v.iter().map(|&x| (x - max).exp()).collect();
    let sum: f32 = exps.iter().sum();
    exps.iter().map(|&x| x / sum).collect()
}

/// In-place softmax: writes softmax(v) into `out` without allocating.
/// Used by the backward pass to avoid a per-timestep `Vec<f32>` allocation
/// in the gradient w.r.t. the output logits.
fn softmax_into(out: &mut [f32], v: &[f32]) {
    debug_assert_eq!(out.len(), v.len());
    let max = v.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let mut sum = 0.0f32;
    for i in 0..v.len() {
        let e = (v[i] - max).exp();
        out[i] = e;
        sum += e;
    }
    let inv = 1.0 / sum;
    for x in out.iter_mut() {
        *x *= inv;
    }
}

fn mat_mul(weights: &[f32], input: &[f32], rows: usize, cols: usize) -> Vec<f32> {
    let mut result = Vec::with_capacity(rows);
    for i in 0..rows {
        let mut sum = 0.0f32;
        for j in 0..cols {
            sum += weights[i * cols + j] * input[j];
        }
        result.push(sum);
    }
    result
}

/// Fused gradient clip + SGD step: for every element, `w -= lr * clamp(g, -c, c)`.
/// The body is the same shape as the original per-tensor loop in
/// `apply_gradients`, but operating on one contiguous slice (instead of
/// 16 disjoint ones) lets LLVM auto-vectorize the entire region as a
/// single SIMD pass.
#[inline]
fn apply_fused(w: &mut [f32], g: &[f32], lr: f32, clip: f32) {
    debug_assert_eq!(w.len(), g.len());
    for i in 0..w.len() {
        let mut gi = g[i];
        if gi > clip {
            gi = clip;
        } else if gi < -clip {
            gi = -clip;
        }
        w[i] -= lr * gi;
    }
}

fn write_f32_vec(writer: &mut impl Write, v: &[f32]) -> std::io::Result<()> {
    writer.write_all(&(v.len() as u64).to_le_bytes())?;
    let bytes = v.iter().flat_map(|f| f.to_le_bytes()).collect::<Vec<_>>();
    writer.write_all(&bytes)?;
    Ok(())
}

fn read_f32_vec(reader: &mut impl Read) -> std::io::Result<Vec<f32>> {
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf)?;
    let len = u64::from_le_bytes(buf) as usize;

    // Defense in depth: even after config validation, the per-vector
    // length prefix could in principle claim a multi-GB tensor. Refuse
    // anything beyond `MAX_VEC_LEN` so a corrupt or hostile file cannot
    // trigger an unbounded allocation inside this function.
    if len > MAX_VEC_LEN {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!(
                "vector length {} exceeds max {} (corrupt CharRNN save?)",
                len, MAX_VEC_LEN
            ),
        ));
    }

    let mut result = Vec::with_capacity(len);
    let mut bytes = [0u8; 4];
    for _ in 0..len {
        reader.read_exact(&mut bytes)?;
        result.push(f32::from_le_bytes(bytes));
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_model_creation() {
        let config = ModelConfig::default();
        let model = CharRNN::new(config);
        assert_eq!(model.num_params() > 0, true);
    }

    #[test]
    fn test_num_params_matches_actual_storage() {
        // Regression: num_params() must match the actual weight/bias storage
        // across all layers, including the layer-1 input-dimension widening.
        let config = ModelConfig::default();
        let model = CharRNN::new(config.clone());

        let reported = model.num_params();

        // Count actual stored params directly from the struct.
        let mut actual: usize = 0;
        actual += model.embedding.len();
        for cell in &model.lstm {
            actual += cell.i_weight().len() + cell.i_bias().len();
            actual += cell.f_weight().len() + cell.f_bias().len();
            actual += cell.c_weight().len() + cell.c_bias().len();
            actual += cell.o_weight().len() + cell.o_bias().len();
        }
        actual += model.output_weight.len() + model.output_bias.len();

        assert_eq!(reported, actual, "num_params() must match actual stored tensor sizes");
    }

    #[test]
    fn test_num_params_three_layers() {
        // Three layers should be even further off the buggy formula, which
        // would always treat layers 1..N as if they had embedding_dim input.
        let config = ModelConfig {
            vocab_size: 227,
            embedding_dim: 64,
            hidden_size: 256,
            num_layers: 3,
            dropout: 0.1,
        };
        let model = CharRNN::new(config);

        let expected_lstm_layer_0 = 4 * 256 * (64 + 256) + 4 * 256;
        let expected_lstm_layer_n = 4 * 256 * (256 + 256) + 4 * 256;
        let expected_lstm_total = expected_lstm_layer_0 + 2 * expected_lstm_layer_n;
        let expected_total =
            227 * 64 + expected_lstm_total + 256 * 227 + 227;

        assert_eq!(model.num_params(), expected_total);
    }
    
    #[test]
    fn test_forward_pass() {
        let config = ModelConfig::default();
        let vocab_size = config.vocab_size;
        let mut model = CharRNN::new(config);
        model.reset_hidden();
        
        // Single forward pass
        let probs = model.predict_next(0);
        assert_eq!(probs.len(), vocab_size);
        assert!((probs.iter().sum::<f32>() - 1.0).abs() < 0.01);
    }
    
    #[test]
    fn test_hidden_reset() {
        let config = ModelConfig::default();
        let vocab_size = config.vocab_size;
        let mut model = CharRNN::new(config);

        // Run some steps
        model.predict_next(0);
        model.predict_next(0);

        // Reset should work
        model.reset_hidden();

        // Model should still work after reset
        let probs = model.predict_next(0);
        assert_eq!(probs.len(), vocab_size);
    }

    // ----- CharRNN::load: config validation -----
    //
    // The loader reads 4 u32 + 1 f32 fields as ModelConfig and then
    // calls `Self::new(config)` which pre-allocates weight tensors sized
    // by those fields. Without validation, any non-ChaRNN file (e.g. a
    // Python pickle misnamed with .pt) would feed its first 4 bytes
    // into `vocab_size` and trigger a multi-petabyte allocation.
    //
    // These tests pin the validation. The reproduction is the actual
    // `models/ckpt_e28_b500.pt` header bytes (PROTO 4 pickle header,
    // which decodes to vocab=67M, embed=2.3B).

    /// Write a buffer to a temp file and return the path.
    fn write_temp(name: &str, bytes: &[u8]) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join("star_charrnn_load_tests");
        std::fs::create_dir_all(&dir).expect("create temp dir");
        let path = dir.join(name);
        std::fs::write(&path, bytes).expect("write temp file");
        path
    }

    /// Write a 5-field CharRNN config header (vocab, embed, hidden, layers,
    /// dropout) as little-endian bytes.
    fn write_charrnn_config(
        vocab_size: u32,
        embedding_dim: u32,
        hidden_size: u32,
        num_layers: u32,
        dropout: f32,
    ) -> Vec<u8> {
        let mut buf = Vec::with_capacity(20);
        buf.extend_from_slice(&vocab_size.to_le_bytes());
        buf.extend_from_slice(&embedding_dim.to_le_bytes());
        buf.extend_from_slice(&hidden_size.to_le_bytes());
        buf.extend_from_slice(&num_layers.to_le_bytes());
        buf.extend_from_slice(&dropout.to_le_bytes());
        buf
    }

    /// `Result::unwrap_err` requires `T: Debug`, but `CharRNN` doesn't
    /// implement `Debug`. This wrapper turns a `Result<CharRNN, _>` into
    /// its error, panicking if it was unexpectedly `Ok`.
    fn expect_io_err(r: std::io::Result<CharRNN>) -> std::io::Error {
        match r {
            Err(e) => e,
            Ok(_) => panic!("expected Err, got Ok"),
        }
    }

    #[test]
    fn test_load_rejects_pickle_header() {
        // The first 20 bytes of `models/ckpt_e28_b500.pt` (the file that
        // triggered the 36PB allocation). Python pickle protocol 4 header
        // (`\x80\x04` = PROTO 4) followed by pickle opcodes that decode
        // to absurd `vocab_size` / `embedding_dim` values.
        let pickle_header: [u8; 20] = [
            0x80, 0x75, 0x03, 0x04, 0x00, 0x00, 0x08, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let path = write_temp("pickle_header.bin", &pickle_header);

        let result = CharRNN::load(path.to_str().unwrap());
        assert!(result.is_err(), "expected Err for pickle header, got Ok");
        let err = expect_io_err(result);
        assert_eq!(
            err.kind(),
            std::io::ErrorKind::InvalidData,
            "expected InvalidData, got {:?}: {}",
            err.kind(),
            err
        );
        assert!(
            err.to_string().contains("vocab_size")
                || err.to_string().contains("embedding_dim")
                || err.to_string().contains("hidden_size")
                || err.to_string().contains("num_layers"),
            "error should mention a config field, got: {}",
            err
        );
    }

    #[test]
    fn test_load_rejects_zero_vocab_size() {
        // Valid header shape, but vocab_size = 0. Should be rejected.
        let bytes = write_charrnn_config(0, 64, 256, 2, 0.1);
        let path = write_temp("zero_vocab.bin", &bytes);

        let err = expect_io_err(CharRNN::load(path.to_str().unwrap()));
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn test_load_rejects_zero_hidden_size() {
        let bytes = write_charrnn_config(227, 64, 0, 2, 0.1);
        let path = write_temp("zero_hidden.bin", &bytes);

        let err = expect_io_err(CharRNN::load(path.to_str().unwrap()));
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn test_load_rejects_oversized_vocab_size() {
        // vocab_size = 100M, beyond MAX_VOCAB_SIZE. Should be rejected
        // before any allocation.
        let bytes = write_charrnn_config(100_000_001, 64, 256, 2, 0.1);
        let path = write_temp("oversized_vocab.bin", &bytes);

        let err = expect_io_err(CharRNN::load(path.to_str().unwrap()));
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn test_load_rejects_oversized_num_layers() {
        // num_layers = 100, beyond MAX_NUM_LAYERS. Should be rejected.
        let bytes = write_charrnn_config(227, 64, 256, 100, 0.1);
        let path = write_temp("oversized_layers.bin", &bytes);

        let err = expect_io_err(CharRNN::load(path.to_str().unwrap()));
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn test_load_rejects_oversized_vector_length() {
        // Defense in depth: valid config (would pass the config check),
        // but the very next 8 bytes claim a vector length of 1 billion
        // floats (= 4GB, in a 12-byte file). The read_f32_vec guard
        // must reject this without trying to allocate.
        let mut bytes = write_charrnn_config(227, 64, 256, 2, 0.1);
        bytes.extend_from_slice(&(1_000_000_000u64).to_le_bytes());
        let path = write_temp("oversized_vec.bin", &bytes);

        let err = expect_io_err(CharRNN::load(path.to_str().unwrap()));
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
        assert!(
            err.to_string().contains("vector length")
                || err.to_string().contains("exceeds max"),
            "error should mention vector length, got: {}",
            err
        );
    }

    #[test]
    fn test_load_rejects_truncated_file() {
        // Valid config (20 bytes) but no data after. The first
        // `read_f32_vec` call will hit UnexpectedEof, which is a valid
        // io error — confirms we don't silently produce a half-loaded
        // model. (Not InvalidData; just any error.)
        let bytes = write_charrnn_config(227, 64, 256, 2, 0.1);
        let path = write_temp("truncated.bin", &bytes);

        let result = CharRNN::load(path.to_str().unwrap());
        assert!(result.is_err(), "truncated file must error");
    }

    #[test]
    fn test_load_rejects_real_ckpt_e28_b500_pt() {
        // End-to-end smoke test: the actual `models/ckpt_e28_b500.pt`
        // file that triggered the 36-petabyte allocation in production
        // (it is a Python pickle, not a CharRNN save). Loading it must
        // return a clean `InvalidData` error WITHOUT attempting any
        // multi-petabyte allocation. Test is `#[ignore]`-free so it
        // runs in CI; the file path is resolved relative to the
        // project root where `cargo test` runs.
        let path = std::path::PathBuf::from("../models/ckpt_e28_b500.pt");
        if !path.exists() {
            // Skip silently if the model file isn't present in this
            // checkout (e.g. minimal CI). The synthetic `pickle_header`
            // test above pins the same behavior.
            eprintln!(
                "skipping real-file smoke: {} not found",
                path.display()
            );
            return;
        }

        let err = expect_io_err(CharRNN::load(path.to_str().unwrap()));
        assert_eq!(
            err.kind(),
            std::io::ErrorKind::InvalidData,
            "expected InvalidData, got {:?}: {}",
            err.kind(),
            err
        );
        eprintln!("real-file load rejected cleanly: {}", err);
    }

    #[test]
    fn test_load_roundtrips_default_config() {
        // Save with default config, load it back, confirm shape matches.
        // This pins the happy path so the validation can't reject a
        // legitimate charRNN file.
        let original = CharRNN::new(ModelConfig::default());
        let path = write_temp("roundtrip.bin", &[]);
        original
            .save(path.to_str().unwrap())
            .expect("save default config");
        let loaded = CharRNN::load(path.to_str().unwrap()).expect("load default config");

        assert_eq!(loaded.config.vocab_size, 227);
        assert_eq!(loaded.config.embedding_dim, 64);
        assert_eq!(loaded.config.hidden_size, 256);
        assert_eq!(loaded.config.num_layers, 2);
        assert_eq!(loaded.num_params(), original.num_params());
    }
}