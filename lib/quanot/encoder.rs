//! Encoder — Text to vector encoding
//!
//! Character-level embedding encoder for converting text to reservoir input vectors.

use rand::Rng;
use std::collections::HashMap;

/// Text encoder: character-level embedding
#[derive(Debug, Clone)]
pub struct TextEncoder {
    /// Vocabulary: character -> index
    vocab: HashMap<char, usize>,
    /// Embedding dimension
    embedding_dim: usize,
    /// Embedding vectors: [char_idx][embedding_dim]
    embeddings: Vec<Vec<f64>>,
    /// Default embedding (for unknown chars)
    default_embedding: Vec<f64>,
}

impl TextEncoder {
    /// Create a new encoder with given embedding dimension
    pub fn new(embedding_dim: usize) -> Self {
        // Standard character vocabulary
        let chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 .,!?;:-'\"()[]{}@#$%^&*+=<>/\\|~`\n\r\t";

        let mut vocab = HashMap::new();
        let mut embeddings = Vec::with_capacity(chars.len());

        // Initialize RNG
        let mut rng = rand::thread_rng();

        for (i, c) in chars.chars().enumerate() {
            vocab.insert(c, i);

            // Random embedding with small magnitude
            let embedding: Vec<f64> = (0..embedding_dim)
                .map(|_| (rng.gen::<f64>() - 0.5) * 0.1)
                .collect();
            embeddings.push(embedding);
        }

        // Default embedding (for unknown characters)
        let default_embedding = vec![0.0; embedding_dim];

        Self {
            vocab,
            embedding_dim,
            embeddings,
            default_embedding,
        }
    }

    /// Encode a single text string to a vector
    ///
    /// Returns: normalized mean-pooled embedding
    pub fn encode(&self, text: &str) -> Vec<f64> {
        let mut sum = vec![0.0; self.embedding_dim];
        let mut count = 0;

        for c in text.chars() {
            if let Some(&idx) = self.vocab.get(&c) {
                for (j, &val) in self.embeddings[idx].iter().enumerate() {
                    sum[j] += val;
                }
                count += 1;
            }
            // Unknown chars ignored (skip)
        }

        if count == 0 {
            return self.default_embedding.clone();
        }

        // Mean pool
        for val in &mut sum {
            *val /= count as f64;
        }

        // Normalize to unit vector
        let norm: f64 = sum.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-10);
        for val in &mut sum {
            *val /= norm;
        }

        sum
    }

    /// Batch encode multiple texts
    pub fn batch_encode(&self, texts: &[String]) -> Vec<Vec<f64>> {
        texts.iter().map(|t| self.encode(t)).collect()
    }

    /// Encode with position information (appended to embedding)
    pub fn encode_with_position(&self, text: &str) -> Vec<f64> {
        let mut sum = vec![0.0; self.embedding_dim];
        let mut count = 0;

        for (pos, c) in text.chars().enumerate() {
            if let Some(&idx) = self.vocab.get(&c) {
                // Add embedding + position signal
                for (j, &val) in self.embeddings[idx].iter().enumerate() {
                    sum[j] += val + (pos as f64 * 0.01);
                }
                count += 1;
            }
        }

        if count == 0 {
            return self.default_embedding.clone();
        }

        // Mean pool
        for val in &mut sum {
            *val /= count as f64;
        }

        // Normalize
        let norm: f64 = sum.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-10);
        for val in &mut sum {
            *val /= norm;
        }

        sum
    }

    /// Get vocabulary size
    pub fn vocab_size(&self) -> usize {
        self.vocab.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoder_new() {
        let encoder = TextEncoder::new(64);
        assert_eq!(encoder.embedding_dim, 64);
        assert!(encoder.vocab_size() > 50);
    }

    #[test]
    fn test_encode() {
        let encoder = TextEncoder::new(64);
        let vec = encoder.encode("hello world");

        assert_eq!(vec.len(), 64);

        // Check it's normalized
        let norm: f64 = vec.iter().map(|x| x * x).sum::<f64>().sqrt();
        assert!((norm - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_encode_empty() {
        let encoder = TextEncoder::new(64);
        let vec = encoder.encode("");

        assert_eq!(vec.len(), 64);
    }

    #[test]
    fn test_batch_encode() {
        let encoder = TextEncoder::new(32);
        let texts = vec!["hello".to_string(), "world".to_string()];
        let results = encoder.batch_encode(&texts);

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].len(), 32);
        assert_eq!(results[1].len(), 32);
    }

    #[test]
    fn test_encode_unknown_chars() {
        let encoder = TextEncoder::new(64);
        // These chars should be in vocab
        let vec = encoder.encode("abc");
        assert_eq!(vec.len(), 64);

        // Unknown chars should be skipped
        let vec2 = encoder.encode("abc\x00\x01\x02");
        // Should be same as "abc" since unknown chars are skipped
        let _vec1 = encoder.encode("abc");
        for (a, b) in vec.iter().zip(vec2.iter()) {
            assert!((a - b).abs() < 1e-10);
        }
    }
}
