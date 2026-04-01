//! Pathway Fusion — reasoning pathway combination (R&D-E architecture)

/// A single reasoning pathway — a candidate conclusion from one reasoning method.
#[derive(Debug, Clone)]
pub struct Pathway {
    /// The conclusion this pathway reached
    pub conclusion: String,
    /// How this pathway voted (support, oppose, neutral)
    pub vote: PathwayVote,
    /// Confidence in this pathway's conclusion
    pub confidence: f64,
    /// Which reasoning method produced this
    pub source: &'static str,
    /// Chain of reasoning steps
    pub chain: Vec<String>,
}

/// How a pathway voted on a conclusion.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathwayVote {
    Support,
    Oppose,
    Neutral,
}

/// Fused result from multiple pathways.
#[derive(Debug)]
pub struct FusedResult {
    /// The fused conclusion
    pub conclusion: String,
    /// Overall confidence (0-1)
    pub confidence: f64,
    /// What the majority voted
    pub majority_vote: PathwayVote,
    /// The pathways that contributed
    pub pathways: Vec<Pathway>,
}

/// Pathway fusion — combines conclusions from multiple reasoning pathways.
#[derive(Default, Clone)]
pub struct PathwayFusion {
    pathways: Vec<Pathway>,
}

impl PathwayFusion {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a pathway vote.
    pub fn add(&mut self, pathway: Pathway) {
        self.pathways.push(pathway);
    }

    /// Fuse all pathways into a result.
    pub fn fuse(&self) -> Option<FusedResult> {
        if self.pathways.is_empty() {
            return None;
        }
        let votes: Vec<_> = self.pathways.iter().collect();
        let total: f64 = votes.iter().map(|p| p.confidence).sum();
        let avg_confidence = total / votes.len() as f64;
        let support_count = votes.iter().filter(|p| p.vote == PathwayVote::Support).count();
        let oppose_count = votes.iter().filter(|p| p.vote == PathwayVote::Oppose).count();
        let majority = if support_count > oppose_count {
            PathwayVote::Support
        } else if oppose_count > support_count {
            PathwayVote::Oppose
        } else {
            PathwayVote::Neutral
        };
        Some(FusedResult {
            conclusion: votes.first().map(|p| p.conclusion.clone()).unwrap_or_default(),
            confidence: avg_confidence,
            majority_vote: majority,
            pathways: self.pathways.clone(),
        })
    }

    /// Clear all pathways.
    pub fn reset(&mut self) {
        self.pathways.clear();
    }
}
