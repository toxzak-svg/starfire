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

/// A single LSTM cell
#[derive(Clone)]
struct LSTMCell {
    i_weight: Vec<f32>,  // (hidden + input) -> hidden
    i_bias: Vec<f32>,    // per-hidden-unit bias
    f_weight: Vec<f32>,
    f_bias: Vec<f32>,
    c_weight: Vec<f32>,
    c_bias: Vec<f32>,
    o_weight: Vec<f32>,
    o_bias: Vec<f32>,
}

impl LSTMCell {
    fn new(input_size: usize, hidden_size: usize) -> Self {
        let mut rng = rand::thread_rng();
        let total = input_size + hidden_size;
        let scale = f32::sqrt(2.0 / total as f32);

        LSTMCell {
            i_weight: (0..hidden_size * total).map(|_| rng.gen_range(-scale..scale)).collect(),
            i_bias: vec![0.0; hidden_size],
            f_weight: (0..hidden_size * total).map(|_| rng.gen_range(-scale..scale)).collect(),
            f_bias: vec![1.0; hidden_size], // Forget bias = 1
            c_weight: (0..hidden_size * total).map(|_| rng.gen_range(-scale..scale)).collect(),
            c_bias: vec![0.0; hidden_size],
            o_weight: (0..hidden_size * total).map(|_| rng.gen_range(-scale..scale)).collect(),
            o_bias: vec![0.0; hidden_size],
        }
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

        let mut combined = Vec::with_capacity(total);
        combined.extend_from_slice(x);
        combined.extend_from_slice(h);

        let i_pre = mat_mul(&self.i_weight, &combined, hidden_size, total);
        let f_pre = mat_mul(&self.f_weight, &combined, hidden_size, total);
        let c_pre = mat_mul(&self.c_weight, &combined, hidden_size, total);
        let o_pre = mat_mul(&self.o_weight, &combined, hidden_size, total);

        let mut input_gate = Vec::with_capacity(hidden_size);
        let mut f = Vec::with_capacity(hidden_size);
        let mut c_tilde = Vec::with_capacity(hidden_size);
        let mut o = Vec::with_capacity(hidden_size);

        for i in 0..hidden_size {
            input_gate.push(sigmoid(i_pre[i] + self.i_bias[i]));
            f.push(sigmoid(f_pre[i] + self.f_bias[i]));
            c_tilde.push(f32::tanh(c_pre[i] + self.c_bias[i]));
            o.push(sigmoid(o_pre[i] + self.o_bias[i]));
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

/// Stored activations for a single LSTM layer at one timestep
#[derive(Clone)]
struct LayerActivations {
    /// Input embedding at this timestep
    embedding: Vec<f32>,
    /// Combined [x; h] input to gates
    combined: Vec<f32>,
    /// Pre-activation values (before sigmoid/tanh)
    i_pre: Vec<f32>,
    f_pre: Vec<f32>,
    c_pre: Vec<f32>,
    o_pre: Vec<f32>,
    /// Post-activation gate values
    i: Vec<f32>,
    f: Vec<f32>,
    c_tilde: Vec<f32>,
    o: Vec<f32>,
    /// Cell and hidden states
    c: Vec<f32>,
    h: Vec<f32>,
}

/// Stored activations for an entire sequence
#[derive(Clone)]
pub struct SequenceActivations {
    pub layers: Vec<Vec<LayerActivations>>,  // [timestep][layer]
    pub output_logits: Vec<Vec<f32>>,        // [timestep][vocab_size]
}

/// Gradient storage for one layer's parameters
#[derive(Clone)]
struct LayerGradients {
    i_weight: Vec<f32>,
    i_bias: Vec<f32>,
    f_weight: Vec<f32>,
    f_bias: Vec<f32>,
    c_weight: Vec<f32>,
    c_bias: Vec<f32>,
    o_weight: Vec<f32>,
    o_bias: Vec<f32>,
}

impl LayerGradients {
    fn new(hidden_size: usize, input_size: usize) -> Self {
        let total = input_size + hidden_size;
        LayerGradients {
            i_weight: vec![0.0; hidden_size * total],
            i_bias: vec![0.0; hidden_size],
            f_weight: vec![0.0; hidden_size * total],
            f_bias: vec![0.0; hidden_size],
            c_weight: vec![0.0; hidden_size * total],
            c_bias: vec![0.0; hidden_size],
            o_weight: vec![0.0; hidden_size * total],
            o_bias: vec![0.0; hidden_size],
        }
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

    /// Forward pass through entire sequence, storing intermediates for BPTT
    pub fn forward_sequence(&mut self, sequence: &[usize]) -> SequenceActivations {
        let seq_len = sequence.len();
        let hidden_size = self.config.hidden_size;
        let embedding_dim = self.config.embedding_dim;
        let vocab_size = self.config.vocab_size;
        let num_layers = self.config.num_layers;

        let mut activations = SequenceActivations {
            layers: Vec::with_capacity(seq_len),
            output_logits: Vec::with_capacity(seq_len),
        };

        for &char_idx in sequence {
            // Get embedding
            let emb_start = char_idx * embedding_dim;
            let emb_end = emb_start + embedding_dim;
            let embedding = self.embedding[emb_start..emb_end].to_vec();

            // Forward through LSTM layers, storing intermediates
            let mut layer_acts = Vec::with_capacity(num_layers);
            let mut input = embedding.clone();

            for layer_idx in 0..num_layers {
                let lstm_cell = &self.lstm[layer_idx];
                let h_prev = &self.hidden[layer_idx];
                let c_prev = &self.cell[layer_idx];
                let input_size = if layer_idx == 0 { embedding_dim } else { hidden_size };
                let total = input_size + hidden_size;

                // Concatenate input and hidden
                let mut combined = Vec::with_capacity(total);
                combined.extend_from_slice(&input);
                combined.extend_from_slice(h_prev);

                // Compute pre-activations
                let i_pre = mat_mul_vec(&lstm_cell.i_weight, &combined, hidden_size, total);
                let f_pre = mat_mul_vec(&lstm_cell.f_weight, &combined, hidden_size, total);
                let c_pre = mat_mul_vec(&lstm_cell.c_weight, &combined, hidden_size, total);
                let o_pre = mat_mul_vec(&lstm_cell.o_weight, &combined, hidden_size, total);

                // Apply activations
                let i = sigmoid_vec(&i_pre, &lstm_cell.i_bias);
                let f = sigmoid_vec(&f_pre, &lstm_cell.f_bias);
                let c_tilde = tanh_vec(&c_pre, &lstm_cell.c_bias);
                let o = sigmoid_vec(&o_pre, &lstm_cell.o_bias);

                // Update cell and hidden
                let mut c_new = Vec::with_capacity(hidden_size);
                let mut h_new = Vec::with_capacity(hidden_size);
                for idx in 0..hidden_size {
                    let c_val = f[idx] * c_prev[idx] + i[idx] * c_tilde[idx];
                    c_new.push(c_val);
                    h_new.push(o[idx] * f32::tanh(c_val));
                }

                layer_acts.push(LayerActivations {
                    embedding: input.clone(),
                    combined,
                    i_pre,
                    f_pre,
                    c_pre,
                    o_pre,
                    i,
                    f,
                    c_tilde,
                    o,
                    c: c_new.clone(),
                    h: h_new.clone(),
                });

                self.hidden[layer_idx] = h_new;
                self.cell[layer_idx] = c_new;
                input = self.hidden[layer_idx].clone();
            }

            // Output projection
            let hidden_last = &self.hidden[num_layers - 1];
            let mut logits = Vec::with_capacity(vocab_size);
            for i in 0..vocab_size {
                let mut sum = self.output_bias[i];
                for j in 0..hidden_size {
                    sum += self.output_weight[j * vocab_size + i] * hidden_last[j];
                }
                logits.push(sum);
            }

            activations.layers.push(layer_acts);
            activations.output_logits.push(logits);
        }

        activations
    }

    /// Reset hidden state to zeros
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

        let mut dh_next = vec![vec![0.0f32; hidden_size]; num_layers];
        let mut dc_next = vec![vec![0.0f32; hidden_size]; num_layers];

        for t in (0..seq_len.saturating_sub(1)).rev() {
            let logits = &activations.output_logits[t];
            let probs = softmax(logits);
            let target_idx = target[t];

            let mut d_logits = probs;
            d_logits[target_idx] -= 1.0;

            for i in 0..vocab_size {
                gradients.output_bias[i] += d_logits[i];
            }

            let h_last = &activations.layers[t][num_layers - 1].h;
            for j in 0..hidden_size {
                for i in 0..vocab_size {
                    gradients.output_weight[j * vocab_size + i] += h_last[j] * d_logits[i];
                }
            }

            let mut dh = vec![0.0f32; hidden_size];
            for j in 0..hidden_size {
                for i in 0..vocab_size {
                    dh[j] += self.output_weight[j * vocab_size + i] * d_logits[i];
                }
            }

            for layer_idx in (0..num_layers).rev() {
                let input_size = if layer_idx == 0 { embedding_dim } else { hidden_size };
                let total = input_size + hidden_size;

                let acts = &activations.layers[t][layer_idx];

                for j in 0..hidden_size {
                    dh[j] += dh_next[layer_idx][j];
                }

                let mut dc_tanh = vec![0.0f32; hidden_size];
                for idx in 0..hidden_size {
                    let c_val = acts.c[idx];
                    let tanh_c = f32::tanh(c_val);
                    let sech_sq = 1.0 - tanh_c * tanh_c;
                    dc_tanh[idx] = dh[idx] * acts.o[idx] * sech_sq + dc_next[layer_idx][idx];
                }

                let mut di_pre = vec![0.0f32; hidden_size];
                let mut df_pre = vec![0.0f32; hidden_size];
                let mut dc_tilde_pre = vec![0.0f32; hidden_size];
                let mut do_pre = vec![0.0f32; hidden_size];

                for idx in 0..hidden_size {
                    let sig_i = acts.i[idx] * (1.0 - acts.i[idx]);
                    let sig_f = acts.f[idx] * (1.0 - acts.f[idx]);
                    let tanh_deriv = 1.0 - acts.c_tilde[idx] * acts.c_tilde[idx];
                    let sig_o = acts.o[idx] * (1.0 - acts.o[idx]);

                    di_pre[idx] = dc_tanh[idx] * acts.c_tilde[idx] * sig_i;
                    df_pre[idx] = dc_tanh[idx] * acts.c[idx] * sig_f;
                    dc_tilde_pre[idx] = dc_tanh[idx] * acts.i[idx] * tanh_deriv;
                    do_pre[idx] = dh[idx] * f32::tanh(acts.c[idx]) * sig_o;
                }

                for i in 0..hidden_size {
                    for j in 0..total {
                        gradients.layers[layer_idx].i_weight[i * total + j] += di_pre[i] * acts.combined[j];
                        gradients.layers[layer_idx].f_weight[i * total + j] += df_pre[i] * acts.combined[j];
                        gradients.layers[layer_idx].c_weight[i * total + j] += dc_tilde_pre[i] * acts.combined[j];
                        gradients.layers[layer_idx].o_weight[i * total + j] += do_pre[i] * acts.combined[j];
                    }
                }

                for i in 0..hidden_size {
                    gradients.layers[layer_idx].i_bias[i] += di_pre[i];
                    gradients.layers[layer_idx].f_bias[i] += df_pre[i];
                    gradients.layers[layer_idx].c_bias[i] += dc_tilde_pre[i];
                    gradients.layers[layer_idx].o_bias[i] += do_pre[i];
                }

                let mut dx = vec![0.0f32; input_size];
                let mut dh_prev = vec![0.0f32; hidden_size];

                for j in 0..input_size {
                    for i in 0..hidden_size {
                        dx[j] += self.lstm[layer_idx].i_weight[i * total + j] * di_pre[i];
                        dx[j] += self.lstm[layer_idx].f_weight[i * total + j] * df_pre[i];
                        dx[j] += self.lstm[layer_idx].c_weight[i * total + j] * dc_tilde_pre[i];
                        dx[j] += self.lstm[layer_idx].o_weight[i * total + j] * do_pre[i];
                    }
                }

                for j in 0..hidden_size {
                    for i in 0..hidden_size {
                        let h_j = input_size + j;
                        dh_prev[j] += self.lstm[layer_idx].i_weight[i * total + h_j] * di_pre[i];
                        dh_prev[j] += self.lstm[layer_idx].f_weight[i * total + h_j] * df_pre[i];
                        dh_prev[j] += self.lstm[layer_idx].c_weight[i * total + h_j] * dc_tilde_pre[i];
                        dh_prev[j] += self.lstm[layer_idx].o_weight[i * total + h_j] * do_pre[i];
                    }
                }

                if layer_idx == 0 {
                    for j in 0..embedding_dim {
                        gradients.embedding[sequence[t] * embedding_dim + j] += dx[j];
                    }
                }

                dh = dh_prev;

                for idx in 0..hidden_size {
                    dc_next[layer_idx][idx] = dc_tanh[idx] * acts.f[idx];
                }
            }
        }

        gradients
    }

    /// Apply gradients with clipping and learning rate
    pub fn apply_gradients(&mut self, gradients: &ModelGradients, lr: f32, clip_val: f32) {
        let embedding_dim = self.config.embedding_dim;
        let hidden_size = self.config.hidden_size;
        let vocab_size = self.config.vocab_size;

        // Clip and apply embedding gradients
        for i in 0..self.embedding.len() {
            let mut grad = gradients.embedding[i];
            grad = grad.max(-clip_val).min(clip_val);
            self.embedding[i] -= lr * grad;
        }

        // Clip and apply LSTM gradients
        for (layer_idx, layer_grad) in gradients.layers.iter().enumerate() {
            let input_size = if layer_idx == 0 { embedding_dim } else { hidden_size };
            let total = input_size + hidden_size;

            for i in 0..self.lstm[layer_idx].i_weight.len() {
                let mut g = layer_grad.i_weight[i];
                g = g.max(-clip_val).min(clip_val);
                self.lstm[layer_idx].i_weight[i] -= lr * g;
            }
            for i in 0..self.lstm[layer_idx].f_weight.len() {
                let mut g = layer_grad.f_weight[i];
                g = g.max(-clip_val).min(clip_val);
                self.lstm[layer_idx].f_weight[i] -= lr * g;
            }
            for i in 0..self.lstm[layer_idx].c_weight.len() {
                let mut g = layer_grad.c_weight[i];
                g = g.max(-clip_val).min(clip_val);
                self.lstm[layer_idx].c_weight[i] -= lr * g;
            }
            for i in 0..self.lstm[layer_idx].o_weight.len() {
                let mut g = layer_grad.o_weight[i];
                g = g.max(-clip_val).min(clip_val);
                self.lstm[layer_idx].o_weight[i] -= lr * g;
            }

            for i in 0..self.lstm[layer_idx].i_bias.len() {
                let mut g = layer_grad.i_bias[i];
                g = g.max(-clip_val).min(clip_val);
                self.lstm[layer_idx].i_bias[i] -= lr * g;
            }
            for i in 0..self.lstm[layer_idx].f_bias.len() {
                let mut g = layer_grad.f_bias[i];
                g = g.max(-clip_val).min(clip_val);
                self.lstm[layer_idx].f_bias[i] -= lr * g;
            }
            for i in 0..self.lstm[layer_idx].c_bias.len() {
                let mut g = layer_grad.c_bias[i];
                g = g.max(-clip_val).min(clip_val);
                self.lstm[layer_idx].c_bias[i] -= lr * g;
            }
            for i in 0..self.lstm[layer_idx].o_bias.len() {
                let mut g = layer_grad.o_bias[i];
                g = g.max(-clip_val).min(clip_val);
                self.lstm[layer_idx].o_bias[i] -= lr * g;
            }
        }

        // Clip and apply output gradients
        for i in 0..self.output_weight.len() {
            let mut grad = gradients.output_weight[i];
            grad = grad.max(-clip_val).min(clip_val);
            self.output_weight[i] -= lr * grad;
        }
        for i in 0..self.output_bias.len() {
            let mut grad = gradients.output_bias[i];
            grad = grad.max(-clip_val).min(clip_val);
            self.output_bias[i] -= lr * grad;
        }
    }

    /// Get total parameter count
    pub fn num_params(&self) -> usize {
        let embedding_params = self.config.vocab_size * self.config.embedding_dim;
        
        let lstm_params: usize = self.lstm.iter().map(|cell| {
            let total = self.config.embedding_dim + self.config.hidden_size;
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
        
        // Write weights
        write_f32_vec(&mut writer, &self.embedding)?;
        
        for cell in &self.lstm {
            write_f32_vec(&mut writer, &cell.i_weight)?;
            write_f32_vec(&mut writer, &cell.i_bias)?;
            write_f32_vec(&mut writer, &cell.f_weight)?;
            write_f32_vec(&mut writer, &cell.f_bias)?;
            write_f32_vec(&mut writer, &cell.c_weight)?;
            write_f32_vec(&mut writer, &cell.c_bias)?;
            write_f32_vec(&mut writer, &cell.o_weight)?;
            write_f32_vec(&mut writer, &cell.o_bias)?;
        }
        
        write_f32_vec(&mut writer, &self.output_weight)?;
        write_f32_vec(&mut writer, &self.output_bias)?;
        
        Ok(())
    }
    
    /// Load model from binary format
    pub fn load(path: &str) -> std::io::Result<Self> {
        use std::io::{Read, Write, BufRead};
        
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
            cell.i_weight = read_f32_vec(&mut reader)?;
            cell.i_bias = read_f32_vec(&mut reader)?;
            cell.f_weight = read_f32_vec(&mut reader)?;
            cell.f_bias = read_f32_vec(&mut reader)?;
            cell.c_weight = read_f32_vec(&mut reader)?;
            cell.c_bias = read_f32_vec(&mut reader)?;
            cell.o_weight = read_f32_vec(&mut reader)?;
            cell.o_bias = read_f32_vec(&mut reader)?;
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

fn sigmoid_vec(v: &[f32], bias: &[f32]) -> Vec<f32> {
    v.iter().zip(bias.iter()).map(|(&x, &b)| sigmoid(x + b)).collect()
}

fn tanh_vec(v: &[f32], bias: &[f32]) -> Vec<f32> {
    v.iter().zip(bias.iter()).map(|(&x, &b)| f32::tanh(x + b)).collect()
}

fn softmax(v: &[f32]) -> Vec<f32> {
    let max = v.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let exps: Vec<f32> = v.iter().map(|&x| (x - max).exp()).collect();
    let sum: f32 = exps.iter().sum();
    exps.iter().map(|&x| x / sum).collect()
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

#[allow(dead_code)]
fn mat_mul_vec(weights: &[f32], input: &[f32], rows: usize, cols: usize) -> Vec<f32> {
    mat_mul(weights, input, rows, cols)
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
}