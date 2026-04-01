//! Channel — pub/sub routing for impulses.

use std::collections::HashMap;
use crate::MindId;

/// A named channel through which impulses are routed.
#[derive(Debug, Clone)]
pub struct Channel {
    pub name: String,
    pub subscribers: Vec<MindId>,
}

/// Manages all channels and their subscriptions.
#[derive(Default)]
pub struct ChannelManager {
    channels: HashMap<String, Channel>,
}

impl ChannelManager {
    pub fn new() -> Self { Self::default() }

    pub async fn subscribe(&mut self, channel_name: &str, mind_id: MindId) {
        self.channels.entry(channel_name.to_string())
            .or_insert_with(|| Channel { name: channel_name.to_string(), subscribers: Vec::new() });
        let ch = self.channels.get_mut(channel_name).unwrap();
        if !ch.subscribers.contains(&mind_id) {
            ch.subscribers.push(mind_id);
        }
    }

    pub async fn unsubscribe(&mut self, channel_name: &str, mind_id: MindId) {
        if let Some(ch) = self.channels.get_mut(channel_name) {
            ch.subscribers.retain(|&id| id != mind_id);
        }
    }

    pub async fn subscribers(&self, channel_name: &str) -> Vec<MindId> {
        self.channels.get(channel_name).map(|ch| ch.subscribers.clone()).unwrap_or_default()
    }

    pub async fn list_channels(&self) -> Vec<String> {
        self.channels.keys().cloned().collect()
    }

    pub async fn subscriber_count(&self, channel_name: &str) -> usize {
        self.channels.get(channel_name).map(|ch| ch.subscribers.len()).unwrap_or(0)
    }
}
