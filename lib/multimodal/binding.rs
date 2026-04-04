//! Cross-modal binding — links content across different modalities
//!
//! Given a piece of content from one modality, finds related content
//! in other modalities (e.g., image with its DALL-E prompt).

use super::{BoundContent, ContentId, Modality, MultimodalEngine};

/// Cross-modal binder
#[derive(Debug, Clone)]
pub struct CrossModalBinder {
    /// similarity threshold for linking
    similarity_threshold: f64,
}

impl Default for CrossModalBinder {
    fn default() -> Self {
        Self::new(0.7)
    }
}

impl CrossModalBinder {
    pub fn new(similarity_threshold: f64) -> Self {
        Self { similarity_threshold }
    }

    /// Find related content across modalities
    pub fn find_related(&self, engine: &MultimodalEngine, content_id: &ContentId) -> Vec<RelatedContent> {
        let content = match engine.get(content_id) {
            Some(c) => c,
            None => return Vec::new(),
        };

        let mut related = Vec::new();

        // For text, find related images/audio
        for (id, bound) in engine.content.iter() {
            if id == content_id {
                continue;
            }

            let similarity = cosine_similarity(&content.embedding, &bound.embedding);
            if similarity >= self.similarity_threshold {
                let rel_type = self.determine_relation(&content.modalities[0], &bound.modalities[0]);
                related.push(RelatedContent {
                    id: id.clone(),
                    similarity,
                    relation: rel_type,
                });
            }
        }

        related.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());
        related
    }

    /// Determine the type of cross-modal relation
    fn determine_relation(&self, source: &Modality, target: &Modality) -> CrossModalRelation {
        match (source, target) {
            (Modality::Dalle { .. }, Modality::Image { .. }) => CrossModalRelation::GeneratedBy,
            (Modality::Image { .. }, Modality::Dalle { .. }) => CrossModalRelation::PromptFor,
            (Modality::Text { .. }, Modality::Dalle { .. }) => CrossModalRelation::PromptFor,
            (Modality::Audio { .. }, Modality::Text { .. }) => CrossModalRelation::Transcribed,
            (Modality::Text { .. }, Modality::Audio { .. }) => CrossModalRelation::SpokenAs,
            (Modality::Text { .. }, Modality::Text { .. }) => CrossModalRelation::SemanticSimilarity,
            (Modality::Image { .. }, Modality::Image { .. }) => CrossModalRelation::VisualSimilarity,
            _ => CrossModalRelation::Associated,
        }
    }

    /// Create a binding chain (e.g., text prompt → DALL-E → image)
    pub fn trace_binding_chain(&self, engine: &MultimodalEngine, content_id: &ContentId) -> Vec<ContentId> {
        let mut chain = vec![content_id.clone()];
        let mut visited = std::collections::HashSet::new();
        visited.insert(content_id.clone());

        let mut current = content_id.clone();
        while let Some(next) = self.find_next_in_chain(engine, &current, &visited) {
            chain.push(next.clone());
            visited.insert(next.clone());
            current = next;
        }

        chain
    }

    fn find_next_in_chain(
        &self,
        engine: &MultimodalEngine,
        current: &ContentId,
        visited: &std::collections::HashSet<ContentId>,
    ) -> Option<ContentId> {
        let related = self.find_related(engine, current);
        for rel in related {
            if !visited.contains(&rel.id) {
                if matches!(rel.relation, CrossModalRelation::PromptFor | CrossModalRelation::GeneratedBy) {
                    return Some(rel.id);
                }
            }
        }
        None
    }
}

/// A related content item
#[derive(Debug, Clone)]
pub struct RelatedContent {
    pub id: ContentId,
    pub similarity: f64,
    pub relation: CrossModalRelation,
}

/// Types of cross-modal relations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossModalRelation {
    /// A modality generated this content (e.g., DALL-E generated image)
    GeneratedBy,
    /// This content is the prompt for another (e.g., text prompt for DALL-E)
    PromptFor,
    /// This content is a transcription of another
    Transcribed,
    /// This content was spoken as another
    SpokenAs,
    /// Semantically similar text
    SemanticSimilarity,
    /// Visually similar images
    VisualSimilarity,
    /// Associated but no specific relation type
    Associated,
}

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
    fn test_cross_modal_binder_new() {
        let binder = CrossModalBinder::new(0.8);
        assert_eq!(binder.similarity_threshold, 0.8);
    }

    #[test]
    fn test_relation_determination() {
        let binder = CrossModalBinder::default();
        // Dalle generates Image → GeneratedBy
        let rel1 = binder.determine_relation(
            &Modality::Dalle { prompt: "a cat".to_string(), image_path: "".to_string(), generation_id: "".to_string(), generation_time: None },
            &Modality::Image { path: "img.png".to_string(), description: None, dalle_generation_id: None, width: None, height: None },
        );
        assert_eq!(rel1, CrossModalRelation::GeneratedBy);

        // Text is prompt for Dalle → PromptFor
        let rel2 = binder.determine_relation(
            &Modality::Text { content: "a cat".to_string(), role: None, timestamp: None },
            &Modality::Dalle { prompt: "a cat".to_string(), image_path: "".to_string(), generation_id: "".to_string(), generation_time: None },
        );
        assert_eq!(rel2, CrossModalRelation::PromptFor);

        // Two texts are semantically similar
        let rel3 = binder.determine_relation(
            &Modality::Text { content: "hello".to_string(), role: None, timestamp: None },
            &Modality::Text { content: "world".to_string(), role: None, timestamp: None },
        );
        assert_eq!(rel3, CrossModalRelation::SemanticSimilarity);
    }
}
