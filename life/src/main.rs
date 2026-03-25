//! Star — Emergent Desktop Intelligence
//!
//! A reasoning intelligence that finds its power from architecture, not scale.
//! Run with: `star chat`

use clap::{Parser, Subcommand};
use star::{Runtime, api};
use std::path::PathBuf;
use tracing::info;

#[derive(Parser)]
#[command(name = "star")]
#[command(about = "Star — An emergent desktop intelligence", long_about = None)]
struct Cli {
    /// Data directory (defaults to ~/.star)
    #[arg(short, long)]
    data_dir: Option<PathBuf>,
    
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start an interactive chat session
    Chat,
    /// Check memory status
    Status,
    /// Start the HTTP API server (for Jupyter notebooks)
    Api {
        /// Host to bind to (default: 127.0.0.1)
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        /// Port to listen on (default: 8080)
        #[arg(long, default_value = "8080")]
        port: u16,
    },
}

fn main() -> anyhow::Result<()> {
    // Parse args
    let cli = Cli::parse();
    
    // Determine data directory
    let data_dir = cli.data_dir
        .or_else(|| dirs::data_local_dir().map(|d| d.join("star")))
        .unwrap_or_else(|| PathBuf::from("."));
    
    // Ensure the life/ directory path resolution is correct
    // If we're running from the life/ directory, use ./
    // Otherwise use the standard path
    let life_dir = if data_dir.join("SPEC.md").exists() {
        data_dir.clone()
    } else if data_dir.join("life/SPEC.md").exists() {
        data_dir.join("life")
    } else {
        // Check current directory
        let current = PathBuf::from(".");
        if current.join("SPEC.md").exists() || current.join("life/SPEC.md").exists() {
            if current.join("life/SPEC.md").exists() {
                current.join("life")
            } else {
                current
            }
        } else {
            // Default: create in data_dir/life
            data_dir.join("life")
        }
    };
    
    info!("Star data directory: {:?}", &life_dir);
    
    // Handle commands
    match cli.command.unwrap_or(Commands::Chat) {
        Commands::Chat => chat_loop(life_dir),
        Commands::Status => status_check(life_dir),
        Commands::Api { host, port } => {
            let runtime = Runtime::new(&life_dir)?;
            let rt = std::sync::Arc::new(std::sync::Mutex::new(runtime));
            api::start(rt, &host, port)?;
            Ok(())
        }
    }
}

/// Run the interactive chat loop.
fn chat_loop(data_dir: PathBuf) -> anyhow::Result<()> {
    let mut runtime = Runtime::new(&data_dir)?;
    
    println!("═══════════════════════════════════════════");
    println!("  Star — Emergent Desktop Intelligence");
    println!("═══════════════════════════════════════════");
    println!();
    println!("{}", runtime.identity_summary());
    println!();
    println!("Type /memory to see memory status.");
    println!("Type /identity to hear who I am.");
    println!("Type /quit to end the conversation.");
    println!();
    println!("───────────────────────────────────────────");
    println!();
    
    loop {
        // Print prompt
        print!("> ");
        std::io::Write::flush(&mut std::io::stdout())?;
        
        // Read input
        let mut input = String::new();
        let bytes = std::io::stdin().read_line(&mut input)?;
        
        // Handle EOF or empty
        if bytes == 0 || input.trim().is_empty() {
            println!("\nGoodbye, Zachary.");
            break;
        }
        
        let input = input.trim();
        
        // Process and respond
        match runtime.chat(input) {
            Ok(response) => {
                println!();
                println!("{}", response);
                println!();
                
                // Check for quit
                if input == "/quit" || input == "/exit" {
                    break;
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
    
    Ok(())
}

/// Check and display status.
fn status_check(data_dir: PathBuf) -> anyhow::Result<()> {
    let runtime = Runtime::new(&data_dir)?;
    
    println!("═══════════════════════════════════════════");
    println!("  Star — Status");
    println!("═══════════════════════════════════════════");
    println!();
    println!("{}", runtime.identity_summary());
    println!();
    println!("Session: {}", runtime.session_id().unwrap_or(-1));
    println!();
    
    Ok(())
}
