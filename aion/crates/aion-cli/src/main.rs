//! Aion CLI

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;
use aion_core::{Aion, MindLogic, Impulse, MindConfig, ControlFlow};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

mod minds;
use minds::StarMind;

#[derive(Parser)]
#[command(name = "aion")]
#[command(about = "Aion — Durable execution runtime for AI agents")]
struct Cli {
    #[arg(short, long)]
    data_dir: Option<PathBuf>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a CuriousMind (default thinking mind)
    Start {
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short, long)]
        channel: Option<String>,
    },
    /// Start StarMind — Star's Runtime as a durable Aion Mind
    StartStar {
        #[arg(long, help = "Star API URL (or set STAR_API_URL env var)")]
        api_url: Option<String>,
        #[arg(long, help = "Telegram bot token (or set TELEGRAM_BOT_TOKEN env var)")]
        telegram_token: Option<String>,
        #[arg(long, default_value = "8", help = "Conversation history turns to retain")]
        history_turns: Option<usize>,
    },
    /// Send an impulse
    Impulse { message: String },
    /// Glance at state
    Glance,
    /// List all Minds
    List,
    /// Serve a channel
    Serve { channel: String },
    /// Interactive chat
    Chat,
    /// Show recent thoughts
    Thoughts { #[arg(default_value = "20")] limit: usize },
    /// Runtime status
    Status,
    /// Stop
    Stop,
}

/// CuriousMind — a simple thinking mind for testing.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct CuriousMind { thoughts: Vec<String>, tick_count: u32 }

#[async_trait]
impl MindLogic for CuriousMind {
    const KIND: &'static str = "curious_mind";
    const DESCRIPTION: &'static str = "A thinking mind that explores ideas";

    fn new() -> Self { Self::default() }

    async fn start(&mut self) -> aion_core::AionResult<()> {
        println!("CuriousMind is online. Commands: /timer, /checkpoint, /quit");
        Ok(())
    }

    async fn handle_impulse(&mut self, impulse: &Impulse) -> aion_core::AionResult<ControlFlow> {
        self.tick_count += 1;
        match impulse {
            Impulse::Message(text) => {
                if text == "/timer" {
                    println!("[Mind] Timer impulse triggered");
                } else if text == "/checkpoint" {
                    println!("[Mind] Checkpoint requested");
                    return Ok(ControlFlow::CheckpointAndWait);
                } else {
                    let thought = format!("Thinking about: {}", text);
                    self.thoughts.push(thought);
                    println!("[Mind] {}", self.thoughts.last().unwrap());
                }
            }
            Impulse::Timer(_) => {
                let t = format!("Background tick #{}", self.tick_count);
                self.thoughts.push(t.clone());
                println!("[Mind] {}", t);
            }
            Impulse::Priority(p) => {
                println!("[Mind] Priority: {}", p.reason);
            }
            _ => {}
        }
        Ok(ControlFlow::Continue)
    }

    fn checkpoint_every(&self) -> u32 { 50 }
    fn checkpoint(&self) -> serde_json::Value {
        serde_json::json!({ "thoughts": self.thoughts, "tick_count": self.tick_count })
    }
}

fn default_db_path() -> PathBuf {
    dirs::data_dir().unwrap_or_else(|| PathBuf::from(".")).join("aion").join("aion.db")
}

fn init_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("aion_core=info,aion_cli=info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();
}

// ── Per-mind helpers ───────────────────────────────────────────────────────────

async fn run_curiousmind(db_str: &str, name: Option<String>, channel: Option<String>) -> Result<()> {
    let aion = Aion::<CuriousMind>::new(db_str).await?;
    let mut config = MindConfig::new();
    if let Some(n) = name { config = config.name(n); }
    if let Some(ch) = channel { config = config.channel(ch); }
    let id = aion.start(None, config).await?;
    println!("Started CuriousMind {} (kind='{}')", id, CuriousMind::KIND);
    aion.wait().await?;
    Ok(())
}

async fn run_star(db_str: &str, api_url: Option<String>, telegram_token: Option<String>, history_turns: Option<usize>) -> Result<()> {
    let config = minds::star_mind::StarMindConfig {
        api_url: api_url.unwrap_or_else(|| {
            std::env::var("STAR_API_URL").unwrap_or_else(|_| "http://localhost:8080".to_string())
        }),
        telegram_token: telegram_token.or_else(|| std::env::var("TELEGRAM_BOT_TOKEN").ok()),
        history_turns: history_turns.unwrap_or(8),
        telegram_offset: None,
    };

    let mut mind = StarMind::with_config(config);
    mind.checkpoint.config.telegram_offset = None;

    // Check if Star is reachable
    let http = reqwest::Client::new();
    let health_url = format!("{}/health", mind.checkpoint.config.api_url);
    match http.get(&health_url).send().await {
        Ok(resp) if resp.status().is_success() => {
            println!("✓ Star is alive at {}", mind.checkpoint.config.api_url);
        }
        _ => {
            eprintln!("⚠️  Star not responding at {}. Start her with:", mind.checkpoint.config.api_url);
            eprintln!("   cd ~/.openclaw/workspace/life/life && cargo run --release");
            eprintln!("   (or set STAR_API_URL to point at an existing instance)");
        }
    }

    let aion = Aion::<StarMind>::new(db_str).await?;
    let config = MindConfig::new().name("StarMind");
    let id = aion.start(None, config).await?;
    println!("Started StarMind {} (kind='{}')", id, StarMind::KIND);
    if mind.checkpoint.config.telegram_token.is_some() {
        println!("📱 Telegram polling enabled — messages to your bot will reach Star");
    } else {
        println!("💬 No Telegram token — run with --telegram-token or set TELEGRAM_BOT_TOKEN");
        println!("   Or just send impulses via: aion impulse 'Hello Star'");
    }

    // Schedule a recurring timer so StarMind::handle_impulse polls Telegram
    // This tick drives the background polling when using Telegram
    aion.schedule_timer(3000).await?; // every 3 seconds

    aion.wait().await?;
    Ok(())
}

// ── Main ───────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    init_logging();

    let cli = Cli::parse();
    let db_path = cli
        .data_dir
        .map(|p| p.join("aion.db"))
        .unwrap_or_else(default_db_path);
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?
    }
    let db_str = db_path.to_str().unwrap_or("aion.db");

    match cli.command {
        Commands::Start { name, channel } => {
            run_curiousmind(db_str, name, channel).await?;
        }

        Commands::StartStar { api_url, telegram_token, history_turns } => {
            run_star(db_str, api_url, telegram_token, history_turns).await?;
        }

        Commands::Impulse { message } => {
            let aion = Aion::<CuriousMind>::new(db_str).await?;
            aion.impulse(Impulse::message(message)).await?;
            println!("Impulse sent");
        }

        Commands::Glance => {
            let aion = Aion::<CuriousMind>::new(db_str).await?;
            let state = aion
                .glance(|v| serde_json::to_string_pretty(v).unwrap())
                .await?;
            println!("{}", state);
        }

        Commands::List => {
            let aion = Aion::<CuriousMind>::new(db_str).await?;
            let minds = aion.list_minds().await?;
            if minds.is_empty() {
                println!("No minds");
            } else {
                for m in minds {
                    println!(
                        "{} | {} | {} | {}",
                        m.id,
                        m.kind,
                        m.name.unwrap_or_default(),
                        m.status
                    );
                }
            }
        }

        Commands::Serve { channel } => {
            let aion = Aion::<CuriousMind>::new(db_str).await?;
            let id = aion
                .start(None, MindConfig::new().channel(&channel))
                .await?;
            println!(
                "Mind {} on channel '{}'. Ctrl+C to stop",
                id, channel
            );
            aion.subscribe(&channel).await?;
            tokio::signal::ctrl_c().await?;
            aion.shutdown().await?;
        }

        Commands::Chat => {
            println!("Interactive chat. /timer, /checkpoint, /quit\n");
            let aion = Aion::<CuriousMind>::new(db_str).await?;
            let id = aion.start(None, MindConfig::new()).await?;
            println!("Mind {} started.\n", id);
            loop {
                print!("> ");
                std::io::Write::flush(&mut std::io::stdout())?;
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                let input = input.trim().to_string();
                if input.is_empty() {
                    continue;
                }
                if input == "/quit" {
                    println!("Goodbye!");
                    break;
                }
                if let Err(e) = aion.impulse(Impulse::message(input)).await {
                    eprintln!("Error: {}", e);
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            }
            aion.shutdown().await?;
        }

        Commands::Thoughts { limit } => {
            let aion = Aion::<CuriousMind>::new(db_str).await?;
            let thoughts = aion.thoughts(limit).await?;
            if thoughts.is_empty() {
                println!("No thoughts");
            } else {
                for t in thoughts {
                    println!("{:?} | {:?}", t.started_at, t.kind);
                }
            }
        }

        Commands::Status => {
            let aion = Aion::<CuriousMind>::new(db_str).await?;
            println!("Runtime: {}", db_path.display());
            println!("Active: {}", aion.is_active().await);
            let channels = aion.list_channels().await?;
            println!("Channels: {:?}", channels);
        }

        Commands::Stop => {
            println!("Stopping...");
        }
    }

    Ok(())
}
