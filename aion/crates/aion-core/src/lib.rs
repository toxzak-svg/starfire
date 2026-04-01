//! Aion — Durable Execution Runtime for AI Agents

//! A persistent, checkpoint-based runtime for AI agents. An agent's thinking
//! survives process crashes and restarts.

pub mod error;
pub mod impulse;
pub mod mind;
pub mod thought;
pub mod store;
pub mod channel;
pub mod scheduler;

pub use error::{AionError, AionResult};
pub use impulse::Impulse;
pub use mind::{MindId, MindLogic, MindConfig, MindStatus, MindInfo, ControlFlow, MindKind};
pub use thought::{Thought, ThoughtKind, ThoughtOutcome};
pub use store::Store;
pub use channel::{Channel, ChannelManager};
pub use scheduler::Scheduler;

use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use tracing::{info, warn, debug};

/// The main Aion runtime. Cloneable and Send + Sync thanks to tokio-rusqlite.
pub struct Aion<M: MindLogic> {
    store: Arc<Store>,
    channel_manager: Arc<RwLock<ChannelManager>>,
    active_mind: Arc<RwLock<Option<ActiveMind<M>>>>,
    scheduler: Arc<RwLock<Scheduler>>,
    timer_tx: broadcast::Sender<Impulse>,
}

struct ActiveMind<M: MindLogic> {
    id: MindId,
    inner: M,
    last_checkpoint: i64,
    events_since_checkpoint: u32,
}

impl<M: MindLogic> Aion<M> {
    /// Create a new Aion runtime.
    pub async fn new(db_path: &str) -> AionResult<Self> {
        Self::with_tick(db_path, 1000).await
    }

    /// Create with custom scheduler tick duration (milliseconds).
    pub async fn with_tick(db_path: &str, tick_ms: u64) -> AionResult<Self> {
        let store = Arc::new(Store::new(db_path).await?);
        let channel_manager = Arc::new(RwLock::new(ChannelManager::new()));
        let active_mind: Arc<RwLock<Option<ActiveMind<M>>>> = Arc::new(RwLock::new(None));
        let scheduler = Arc::new(RwLock::new(Scheduler::with_tick(tick_ms)));
        let (timer_tx, _) = broadcast::channel(100);

        // Spawn the scheduler loop
        let scheduler_clone = scheduler.clone();
        let active_clone = active_mind.clone();
        let _timer_rx = timer_tx.subscribe();

        tokio::spawn(async move {
            let mut sched = scheduler_clone.write().await;
            sched.run(move |mind_id, _impulse| {
                // Dispatch timer impulse to the active mind if it matches
                let active = active_clone.blocking_read();
                if let Some(ref a) = *active {
                    if a.id == mind_id {
                        // Timer fired for this mind — handle inline
                        // The actual dispatch happens via the tick check in the main loop
                    }
                }
            }).await;
        });

        info!("Aion runtime initialized (tick={}ms)", tick_ms);
        Ok(Aion { store, channel_manager, active_mind, scheduler, timer_tx })
    }

    /// Start a new Mind.
    pub async fn start(&self, name: Option<String>, config: MindConfig) -> AionResult<MindId> {
        let mut mind = M::new();
        mind.start().await?;

        let checkpoint = mind.checkpoint();
        let checkpoint_json = serde_json::to_string(&checkpoint)
            .map_err(|e| AionError::Serialization(e.to_string()))?;

        let mind_id = MindId::new();

        self.store.create_mind(
            mind_id.as_uuid(), M::KIND, name.as_deref(),
            &checkpoint_json, config.channel.as_deref(),
        ).await?;

        if let Some(ch) = &config.channel {
            self.store.subscribe(ch, mind_id.as_uuid()).await?;
            self.channel_manager.write().await.subscribe(ch, mind_id).await;
        }

        let active = ActiveMind {
            id: mind_id, inner: mind,
            last_checkpoint: chrono::Utc::now().timestamp(),
            events_since_checkpoint: 0,
        };

        *self.active_mind.write().await = Some(active);
        info!("Started mind {} of kind '{}'", mind_id, M::KIND);
        Ok(mind_id)
    }

    /// Send an impulse to the active Mind.
    pub async fn impulse(&self, impulse: Impulse) -> AionResult<()> {
        let mut guard = self.active_mind.write().await;
        let active = guard.as_mut()
            .ok_or_else(|| AionError::InvalidState("no active mind".to_string()))?;

        let control = active.inner.handle_impulse(&impulse).await?;
        active.events_since_checkpoint += 1;

        let _ = self.store.log_event(active.id.as_uuid(), "impulse", Some(&impulse.describe())).await;

        match control {
            ControlFlow::Continue => {
                if active.events_since_checkpoint >= active.inner.checkpoint_every() {
                    drop(guard);
                    self.checkpoint_mind().await?;
                }
            }
            ControlFlow::CheckpointAndWait => {
                drop(guard);
                self.checkpoint_mind().await?;
            }
            ControlFlow::Terminate => {
                drop(guard);
                self.shutdown_mind().await?;
            }
        }
        Ok(())
    }

    async fn checkpoint_mind(&self) -> AionResult<()> {
        let mut guard = self.active_mind.write().await;
        let active = guard.as_mut()
            .ok_or_else(|| AionError::InvalidState("no active mind".to_string()))?;

        let checkpoint = active.inner.checkpoint();
        let json = serde_json::to_string(&checkpoint)
            .map_err(|e| AionError::Serialization(e.to_string()))?;

        self.store.update_checkpoint(active.id.as_uuid(), &json).await?;
        self.store.update_status(active.id.as_uuid(), MindStatus::Checkpointed).await?;

        active.last_checkpoint = chrono::Utc::now().timestamp();
        active.events_since_checkpoint = 0;
        debug!("Checkpointed mind {}", active.id);
        Ok(())
    }

    async fn shutdown_mind(&self) -> AionResult<()> {
        let active = self.active_mind.write().await
            .take()
            .ok_or_else(|| AionError::InvalidState("no active mind".to_string()))?;

        let checkpoint = active.inner.checkpoint();
        let json = serde_json::to_string(&checkpoint)
            .map_err(|e| AionError::Serialization(e.to_string()))?;

        self.store.terminate_mind(active.id.as_uuid(), &json).await?;
        info!("Terminated mind {}", active.id);
        Ok(())
    }

    /// Schedule a timer.
    pub async fn schedule_timer(&self, delay_ms: u64) -> AionResult<String> {
        let timer_id = format!("timer_{}", chrono::Utc::now().timestamp_millis());

        let mut scheduler = self.scheduler.write().await;
        let guard = self.active_mind.read().await;
        let mind_id = guard.as_ref().map(|a| a.id).ok_or_else(|| AionError::InvalidState("no active mind".to_string()))?;
        scheduler.schedule_timer(timer_id.clone(), mind_id, delay_ms);

        let impulse = Impulse::timer(timer_id.clone());
        self.timer_tx.send(impulse).map_err(|_| AionError::Impulse("channel closed".to_string()))?;

        info!("Scheduled timer {} in {}ms", timer_id, delay_ms);
        Ok(timer_id)
    }

    /// Cancel a scheduled timer.
    pub async fn cancel_timer(&self, timer_id: &str) -> AionResult<()> {
        self.scheduler.write().await.cancel_timer(timer_id);
        Ok(())
    }

    /// Glance at the active Mind's checkpoint state (read-only).
    pub async fn glance<F, R>(&self, f: F) -> AionResult<R>
    where F: FnOnce(&serde_json::Value) -> R + Send, R: Send,
    {
        let guard = self.active_mind.read().await;
        let active = guard.as_ref()
            .ok_or_else(|| AionError::InvalidState("no active mind".to_string()))?;
        let checkpoint = active.inner.checkpoint();
        Ok(f(&checkpoint))
    }

    /// List all Minds.
    pub async fn list_minds(&self) -> AionResult<Vec<MindInfo>> {
        self.store.list_minds(None, None).await
    }

    /// List all registered Mind kinds.
    pub async fn list_kinds(&self) -> AionResult<Vec<MindKind>> {
        self.store.list_kinds().await
    }

    /// Subscribe the active Mind to a channel.
    pub async fn subscribe(&self, channel: &str) -> AionResult<()> {
        let guard = self.active_mind.read().await;
        let active = guard.as_ref()
            .ok_or_else(|| AionError::InvalidState("no active mind".to_string()))?;
        self.store.subscribe(channel, active.id.as_uuid()).await?;
        self.channel_manager.write().await.subscribe(channel, active.id).await;
        Ok(())
    }

    /// Broadcast an impulse to all Minds subscribed to a channel.
    pub async fn broadcast(&self, channel: &str, impulse: Impulse) -> AionResult<()> {
        let subscribers = {
            let cm = self.channel_manager.read().await;
            cm.subscribers(channel).await
        };
        for mind_id in subscribers {
            let guard = self.active_mind.read().await;
            if let Some(ref a) = *guard {
                if a.id == mind_id {
                    drop(guard);
                    if let Err(e) = self.impulse(impulse.clone()).await {
                        warn!("Failed to deliver impulse to {}: {}", mind_id, e);
                    }
                }
            }
        }
        Ok(())
    }

    /// Wait for the active Mind to reach a terminal state.
    pub async fn wait(&self) -> AionResult<()> {
        loop {
            let guard = self.active_mind.read().await;
            if guard.is_none() { break; }
            let status = guard.as_ref().map(|a| a.inner.status()).unwrap_or(MindStatus::Unknown);
            drop(guard);
            match status {
                MindStatus::Terminated | MindStatus::Failed => break,
                MindStatus::Unknown => return Err(AionError::InvalidState("mind not found".to_string())),
                _ => { tokio::time::sleep(tokio::time::Duration::from_secs(1)).await; }
            }
        }
        Ok(())
    }

    /// Check if a Mind is currently active.
    pub async fn is_active(&self) -> bool {
        self.active_mind.read().await.is_some()
    }

    /// List all channels.
    pub async fn list_channels(&self) -> AionResult<Vec<(String, usize)>> {
        self.store.list_channels().await
    }

    /// Get recent thoughts for the active Mind.
    pub async fn thoughts(&self, limit: usize) -> AionResult<Vec<Thought>> {
        let guard = self.active_mind.read().await;
        let active = guard.as_ref()
            .ok_or_else(|| AionError::InvalidState("no active mind".to_string()))?;
        self.store.get_thoughts(active.id.as_uuid(), limit).await
    }

    /// Shutdown gracefully, checkpointing the active Mind.
    pub async fn shutdown(&self) -> AionResult<()> {
        let mut guard = self.active_mind.write().await;
        if let Some(ref mut a) = *guard {
            let checkpoint = a.inner.checkpoint();
            let json = serde_json::to_string(&checkpoint)
                .map_err(|e| AionError::Serialization(e.to_string()))?;
            self.store.update_checkpoint(a.id.as_uuid(), &json).await?;
        }
        info!("Aion runtime shutdown complete");
        Ok(())
    }

    /// Get the active Mind's ID.
    pub async fn mind_id(&self) -> AionResult<MindId> {
        let guard = self.active_mind.read().await;
        guard.as_ref()
            .map(|a| a.id)
            .ok_or_else(|| AionError::InvalidState("no active mind".to_string()))
    }
}
