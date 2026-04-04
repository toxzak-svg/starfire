//! Multimodal — Cross-modal binding for text, image, audio, and video
//!
//! Processes and unifies content from multiple modalities into Starfire's
//! reasoning system. Designed to work with ChatGPT export data.
//!
//! # Supported Modalities
//!
//! - **Text**: Conversation messages, system prompts
//! - **Images**: DALL-E generations, uploaded photos
//! - **Audio**: Voice messages, transcriptions
//! - **DALL-E**: Generation metadata + images
//!
//! # Usage
//!
//! ```rust,ignore
//! use star::multimodal::MultimodalEngine;
//!
//! let engine = MultimodalEngine::new();
//! engine.load_chat_export("path/to/export").unwrap();
//! let bound = engine.bind_content("dalle-generations/...").unwrap();
//! ```

pub mod binding;
pub mod text;
pub mod image;
pub mod audio;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Unified content identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentId(pub String);

impl ContentId {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

/// A piece of content from any modality
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "modality")]
pub enum Modality {
    Text {
        content: String,
        role: Option<String>,
        timestamp: Option<i64>,
    },
    Image {
        path: String,
        description: Option<String>,
        dalle_generation_id: Option<String>,
        width: Option<u32>,
        height: Option<u32>,
    },
    Audio {
        path: String,
        transcription: Option<String>,
        duration_secs: Option<f64>,
    },
    Dalle {
        prompt: String,
        image_path: String,
        generation_id: String,
        generation_time: Option<i64>,
    },
}

/// A bound content item with unified embedding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundContent {
    pub id: ContentId,
    pub modalities: Vec<Modality>,
    /// Unified representation vector (for similarity/retrieval)
    pub embedding: Vec<f64>,
    pub timestamp: Option<i64>,
    pub provenance: String,
    pub conversation_id: Option<String>,
}

/// Multimodal engine for processing chat exports
#[derive(Debug, Clone)]
pub struct MultimodalEngine {
    /// All bound content
    content: HashMap<ContentId, BoundContent>,
    /// Text encoder for creating embeddings
    text_encoder: text::TextProcessor,
    /// Image processor
    image_processor: image::ImageProcessor,
    /// Audio processor
    audio_processor: audio::AudioProcessor,
    /// Embedding dimension
    embedding_dim: usize,
}

impl Default for MultimodalEngine {
    fn default() -> Self {
        Self::new(384)
    }
}

impl MultimodalEngine {
    /// Create a new engine with specified embedding dimension
    pub fn new(embedding_dim: usize) -> Self {
        Self {
            content: HashMap::new(),
            text_encoder: text::TextProcessor::new(embedding_dim),
            image_processor: image::ImageProcessor::new(),
            audio_processor: audio::AudioProcessor::new(),
            embedding_dim,
        }
    }

    /// Load a chat export directory
    pub fn load_chat_export(&mut self, path: &Path) -> std::io::Result<()> {
        // Load conversations
        let conversations_path = path.join("conversations-000.json");
        if conversations_path.exists() {
            self.load_conversations(&conversations_path)?;
        }

        // Load DALL-E generations
        let dalle_path = path.join("dalle-generations");
        if dalle_path.exists() {
            self.load_dalle_generations(&dalle_path)?;
        }

        // Load images
        let images_path = path.join("image");
        if images_path.exists() {
            self.load_images(&images_path)?;
        }

        // Load audio
        let audio_path = path.join("audio");
        if audio_path.exists() {
            self.load_audio(&audio_path)?;
        }

        Ok(())
    }

    /// Load conversations JSON
    fn load_conversations(&mut self, path: &Path) -> std::io::Result<()> {
        let data = std::fs::read_to_string(path)?;
        let conversations: Vec<ChatConversation> = match serde_json::from_str(&data) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("Failed to parse conversations: {}", e);
                return Ok(());
            }
        };

        for conv in conversations {
            for msg in conv.messages {
                let embedding = self.text_encoder.encode(&msg.content);
                let bound = BoundContent {
                    id: ContentId::new(&msg.id),
                    modalities: vec![Modality::Text {
                        content: msg.content,
                        role: Some(msg.role),
                        timestamp: msg.timestamp,
                    }],
                    embedding,
                    timestamp: msg.timestamp,
                    provenance: "chatgpt_export".to_string(),
                    conversation_id: Some(conv.id.clone()),
                };
                self.content.insert(bound.id.clone(), bound);
            }
        }

        Ok(())
    }

    /// Load DALL-E generations
    fn load_dalle_generations(&mut self, path: &Path) -> std::io::Result<()> {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path_str = entry.path().to_string_lossy().to_string();

            // Try to extract generation info from filename
            if let Some((generation_id, prompt)) = self.parse_dalle_filename(&entry.file_name()) {
                let embedding = self.text_encoder.encode(&prompt);
                let bound = BoundContent {
                    id: ContentId::new(&generation_id),
                    modalities: vec![Modality::Dalle {
                        prompt: prompt.clone(),
                        image_path: path_str,
                        generation_id: generation_id.clone(),
                        generation_time: None,
                    }],
                    embedding,
                    timestamp: None,
                    provenance: "dalle_generation".to_string(),
                    conversation_id: None,
                };
                self.content.insert(bound.id.clone(), bound);
            }
        }

        Ok(())
    }

    /// Parse DALL-E generation filename to extract ID and prompt hint
    fn parse_dalle_filename(&self, filename: &std::ffi::OsStr) -> Option<(String, String)> {
        let s = filename.to_string_lossy();
        // Format: file-{id}-{rest}.{ext}
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() >= 2 {
            let id = format!("dalle-{}", parts[1]);
            let prompt_hint = s.replace(".webp", "").replace(".png", "").replace(".jpg", "");
            Some((id, prompt_hint))
        } else {
            None
        }
    }

    /// Load images
    fn load_images(&mut self, _path: &Path) -> std::io::Result<()> {
        // Image loading would use image crate for processing
        // For now, we create placeholders - full implementation would decode images
        Ok(())
    }

    /// Load audio
    fn load_audio(&mut self, _path: &Path) -> std::io::Result<()> {
        // Audio loading would use rodio or similar for processing
        // For now, we create placeholders - full implementation would decode audio
        Ok(())
    }

    /// Bind content by ID
    pub fn get(&self, id: &ContentId) -> Option<&BoundContent> {
        self.content.get(id)
    }

    /// Search by text similarity
    pub fn search(&self, query: &str, limit: usize) -> Vec<(ContentId, f64)> {
        let query_embedding = self.text_encoder.encode(query);
        let mut results: Vec<_> = self.content
            .iter()
            .map(|(id, content)| {
                let sim = cosine_similarity(&query_embedding, &content.embedding);
                (id.clone(), sim)
            })
            .collect();

        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);
        results
    }

    /// Get content from a specific conversation
    pub fn get_conversation(&self, conv_id: &str) -> Vec<&BoundContent> {
        self.content
            .values()
            .filter(|c| c.conversation_id.as_deref() == Some(conv_id))
            .collect()
    }

    /// Get all DALL-E generations
    pub fn get_dalle_generations(&self) -> Vec<&BoundContent> {
        self.content
            .values()
            .filter(|c| c.modalities.iter().any(|m| matches!(m, Modality::Dalle { .. })))
            .collect()
    }

    /// Get count of all content
    pub fn len(&self) -> usize {
        self.content.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }
}

/// Chat conversation structure from export
#[derive(Debug, Deserialize)]
struct ChatConversation {
    id: String,
    messages: Vec<ChatMessage>,
}

/// Chat message from export
#[derive(Debug, Deserialize)]
struct ChatMessage {
    id: String,
    role: String,
    content: String,
    timestamp: Option<i64>,
}

/// Cosine similarity helper
fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let mut dot = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;

    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }

    let denom = (norm_a.sqrt() * norm_b.sqrt()).max(1e-10);
    dot / denom
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_id() {
        let id = ContentId::new("test123");
        assert_eq!(id.0, "test123");
    }

    #[test]
    fn test_modality_text() {
        let text = Modality::Text {
            content: "Hello world".to_string(),
            role: Some("user".to_string()),
            timestamp: Some(1234567890),
        };
        match text {
            Modality::Text { content, .. } => assert_eq!(content, "Hello world"),
            _ => panic!("Expected Text modality"),
        }
    }

    #[test]
    fn test_multimodal_engine_search() {
        let engine = MultimodalEngine::new(128);
        let results = engine.search("hello", 5);
        assert!(results.is_empty()); // No content loaded yet
    }

    #[test]
    fn test_cosine_similarity() {
        let a = &[1.0, 0.0, 0.0];
        let b = &[1.0, 0.0, 0.0];
        let c = &[0.0, 1.0, 0.0];

        assert!((cosine_similarity(a, b) - 1.0).abs() < 1e-6);
        assert!((cosine_similarity(a, c) - 0.0).abs() < 1e-6);
    }
}
