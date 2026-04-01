//! aion-drivers — Platform helpers for Aion
//!
//! Provides polling helpers for external platforms (Telegram, etc.) that can be
//! embedded in MindLogic implementations.

pub mod telegram;

pub use telegram::{TgPoller, send_telegram_message};
