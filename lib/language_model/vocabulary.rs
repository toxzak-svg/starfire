//! Vocabulary — Character-level encoding
//!
//! Maps characters to indices and back. Handles ASCII + common punctuation.
//! Unicode support can be added later if needed.

use std::collections::HashMap;

/// Maximum different characters we'll support
const MAX_CHARS: usize = 256;

#[derive(Debug, Clone)]
pub struct Vocabulary {
    /// Character to index mapping
    char_to_idx: HashMap<char, usize>,
    /// Index to character mapping
    idx_to_char: Vec<char>,
    /// Special tokens
    pub eos: usize,   // End of string
    pub pad: usize,   // Padding
    pub unk: usize,   // Unknown character
}

impl Vocabulary {
    pub fn new() -> Self {
        let mut char_to_idx = HashMap::new();
        let mut idx_to_char = Vec::with_capacity(MAX_CHARS);
        
        // Reserved indices
        let eos = 0;  // End of sequence
        let pad = 1;  // Padding
        let unk = 2;  // Unknown
        
        // Add reserved tokens
        idx_to_char.push('\0'); // 0: EOS
        idx_to_char.push(' '); // 1: PAD (placeholder)
        idx_to_char.push('\u{fffd}'); // 2: UNK
        
        // Build character set: ASCII printable + newline
        let mut next_idx = 3;
        
        // Printable ASCII
        for c in ' '..='~' {
            char_to_idx.insert(c, next_idx);
            idx_to_char.push(c);
            next_idx += 1;
        }
        
        // Newline character
        char_to_idx.insert('\n', next_idx);
        idx_to_char.push('\n');
        next_idx += 1;
        
        // Extended ASCII (codes 128-255 = 128 chars) → total vocab = 227
        for c in 128u8..=255u8 {
            let ch = c as char;
            char_to_idx.insert(ch, next_idx);
            idx_to_char.push(ch);
            next_idx += 1;
        }
        
        Self {
            char_to_idx,
            idx_to_char,
            eos,
            pad,
            unk,
        }
    }
    
    /// Encode a string into character indices
    pub fn encode(&self, text: &str) -> Vec<usize> {
        text.chars()
            .map(|c| self.char_to_idx.get(&c).copied().unwrap_or(self.unk))
            .collect()
    }
    
    /// Decode indices back to string
    pub fn decode(&self, indices: &[usize]) -> String {
        indices.iter()
            .map(|&i| self.idx_to_char.get(i).copied().unwrap_or('\u{fffd}'))
            .collect()
    }
    
    /// Vocabulary size
    pub fn size(&self) -> usize {
        self.idx_to_char.len()
    }
    
    /// Encode a string and add EOS token
    pub fn encode_with_eos(&self, text: &str) -> Vec<usize> {
        let mut result = self.encode(text);
        result.push(self.eos);
        result
    }
}

impl Default for Vocabulary {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_encode_decode() {
        let vocab = Vocabulary::new();
        let text = "Hello, world!";
        let encoded = vocab.encode(text);
        let decoded = vocab.decode(&encoded);
        assert_eq!(decoded, text);
    }
    
    #[test]
    fn test_encode_with_eos() {
        let vocab = Vocabulary::new();
        let text = "Hi";
        let encoded = vocab.encode_with_eos(text);
        assert_eq!(encoded.last(), Some(&vocab.eos));
    }
    
    #[test]
    fn test_vocab_size() {
        let vocab = Vocabulary::new();
        // Should have: 3 reserved + 95 printable + 1 newline + 126 extended = 225
        let size = vocab.size();
        assert!(size > 100, "Vocabulary too small: {}", size);
    }
}