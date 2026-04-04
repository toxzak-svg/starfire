//! Text processing for multimodal engine
//!
//! Handles text encoding, embedding, and feature extraction.

use rand::Rng;
use std::collections::HashMap;

/// Text processor for creating embeddings
#[derive(Debug, Clone)]
pub struct TextProcessor {
    embedding_dim: usize,
    /// Character-level vocabulary
    vocab: HashMap<char, usize>,
    /// Embedding vectors
    embeddings: Vec<Vec<f64>>,
}

impl TextProcessor {
    pub fn new(embedding_dim: usize) -> Self {
        let chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 .,!?;:-'\"()[]{}@#$%^&*+=<>/\\|~`\n\r\t";
        let mut vocab = HashMap::new();
        let mut embeddings = Vec::new();

        let mut rng = rand::thread_rng();

        for (i, c) in chars.chars().enumerate() {
            vocab.insert(c, i);
            let embedding: Vec<f64> = (0..embedding_dim)
                .map(|_| (rng.gen::<f64>() - 0.5) * 0.1)
                .collect();
            embeddings.push(embedding);
        }

        Self {
            embedding_dim,
            vocab,
            embeddings,
        }
    }

    /// Encode text to embedding vector
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
        }

        if count == 0 {
            return vec![0.0; self.embedding_dim];
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

    /// Extract features from text
    pub fn extract_features(&self, text: &str) -> TextFeatures {
        let words: Vec<&str> = text.split_whitespace().collect();
        let word_count = words.len();
        let char_count = text.chars().count();

        // Sentiment hints (simple keyword-based)
        let positive = ["good", "great", "excellent", "love", "happy", "nice", "wonderful"];
        let negative = ["bad", "hate", "sad", "terrible", "awful", "horrible", "angry"];

        let _text_lower = text.to_lowercase();
        let mut positive_count = 0;
        let mut negative_count = 0;

        for word in &words {
            let w = word.to_lowercase();
            if positive.iter().any(|p| w.contains(p)) {
                positive_count += 1;
            }
            if negative.iter().any(|n| w.contains(n)) {
                negative_count += 1;
            }
        }

        TextFeatures {
            word_count,
            char_count,
            sentiment_score: (positive_count as f64 - negative_count as f64)
                / (word_count as f64).max(1.0),
        }
    }
}

/// Text features
#[derive(Debug, Clone)]
pub struct TextFeatures {
    pub word_count: usize,
    pub char_count: usize,
    pub sentiment_score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_processor_encode() {
        let processor = TextProcessor::new(128);
        let vec = processor.encode("hello world");
        assert_eq!(vec.len(), 128);
    }

    #[test]
    fn test_text_features() {
        let processor = TextProcessor::new(64);
        let features = processor.extract_features("This is a great day!");
        assert_eq!(features.word_count, 5);
        assert!(features.sentiment_score > 0.0);
    }
}
