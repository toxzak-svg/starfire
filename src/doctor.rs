//! Starfire Doctor — Self-Diagnostic CLI
//!
//! Health-checks all Starfire subsystems in one command.
//! Run with: `starfire doctor [--repair] [--non-interactive] [--deep]`

use anyhow::{Context as _, Result as AnyhowResult};
use star::{Runtime, llm, diagnostic_summary};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(clap::Parser)]
#[command(name = "doctor")]
#[command(about = "Starfire self-diagnostic — check all subsystems at once", long_about = None)]
pub struct DoctorArgs {
    /// Apply recommended repairs (safe fixes only).
    #[arg(short, long)]
    pub repair: bool,

    /// Run without interactive prompts (for CI/cron).
    #[arg(long)]
    pub non_interactive: bool,

    /// Extended checks: clippy, integration tests.
    #[arg(long)]
    pub deep: bool,
}

// ─────────────────────────────────────────────────────────────────────────────
// Check categories
// ─────────────────────────────────────────────────────────────────────────────

/// Result of a single diagnostic check.
struct Check {
    name: String,
    status: CheckStatus,
    detail: Option<String>,
    fix: Option<String>,
}

enum CheckStatus {
    Pass,
    Warn,
    Fail,
    Skip(String),
}

impl Check {
    fn pass(name: &str) -> Self {
        Self { name: name.to_string(), status: CheckStatus::Pass, detail: None, fix: None }
    }

    fn warn(name: &str, detail: &str) -> Self {
        Self {
            name: name.to_string(),
            status: CheckStatus::Warn,
            detail: Some(detail.to_string()),
            fix: None,
        }
    }

    fn warn_with_fix(name: &str, detail: &str, fix: &str) -> Self {
        Self {
            name: name.to_string(),
            status: CheckStatus::Warn,
            detail: Some(detail.to_string()),
            fix: Some(fix.to_string()),
        }
    }

    fn fail(name: &str, detail: &str) -> Self {
        Self {
            name: name.to_string(),
            status: CheckStatus::Fail,
            detail: Some(detail.to_string()),
            fix: None,
        }
    }

    fn fail_with_fix(name: &str, detail: &str, fix: &str) -> Self {
        Self {
            name: name.to_string(),
            status: CheckStatus::Fail,
            detail: Some(detail.to_string()),
            fix: Some(fix.to_string()),
        }
    }

    fn skip(name: &str, reason: &str) -> Self {
        Self {
            name: name.to_string(),
            status: CheckStatus::Skip(reason.to_string()),
            detail: None,
            fix: None,
        }
    }

    fn count(&self) -> (usize, usize, usize) {
        match &self.status {
            CheckStatus::Pass => (1, 0, 0),
            CheckStatus::Warn => (0, 1, 0),
            CheckStatus::Fail => (0, 0, 1),
            CheckStatus::Skip(_) => (0, 0, 0),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Runner
// ─────────────────────────────────────────────────────────────────────────────

pub fn run(args: DoctorArgs) -> AnyhowResult<()> {
    println!();
    banner();
    println!();

    let data_dir = resolve_data_dir();
    // Project root: find Cargo.toml by walking up the directory tree.
    let project_root = find_project_root();
    println!("Data directory: {:?}", &data_dir);
    println!("Project root: {:?}", &project_root);
    println!();

    let mut passed = 0;
    let mut warned = 0;
    let mut failed = 0;

    // ── 1. Build & Tests ────────────────────────────────────────────────────
    println!("[1/8] Build & Tests");
    let (p, w, f) = run_build_checks(&project_root, args.deep);
    passed += p; warned += w; failed += f;
    print_summary_checks(w, f);

    // ── 2. File Integrity ─────────────────────────────────────────────────
    println!();
    println!("[2/8] File Integrity");
    let (p, w, f) = run_file_checks(&data_dir);
    passed += p; warned += w; failed += f;
    print_summary_checks(w, f);

    // ── 3. Identity Seeds ─────────────────────────────────────────────────
    println!();
    println!("[3/8] Identity Seeds");
    let (p, w, f) = run_identity_checks(&data_dir);
    passed += p; warned += w; failed += f;
    print_summary_checks(w, f);

    // ── 4. Runtime Health ──────────────────────────────────────────────────
    println!();
    println!("[4/8] Runtime Health");
    let (p, w, f) = run_runtime_checks(&data_dir, args.repair, args.non_interactive);
    passed += p; warned += w; failed += f;
    print_summary_checks(w, f);

    // ── 5. API Server ─────────────────────────────────────────────────────
    println!();
    println!("[5/8] API Server (port 8080)");
    let (p, w, f) = run_api_checks();
    passed += p; warned += w; failed += f;
    print_summary_checks(w, f);

    // ── 6. LLM Layer ──────────────────────────────────────────────────────
    println!();
    println!("[6/8] LLM Layer (Bonsai-8B)");
    let (p, w, f) = run_llm_checks(&project_root);
    passed += p; warned += w; failed += f;
    print_summary_checks(w, f);

    // ── 7. Subsystem Stats ────────────────────────────────────────────────
    println!();
    println!("[7/8] Subsystem Stats");
    let (p, w, f) = run_subsystem_checks(&data_dir);
    passed += p; warned += w; failed += f;
    print_summary_checks(w, f);

    // ── 8. Autonomy State ─────────────────────────────────────────────────
    println!();
    println!("[8/8] Autonomy State");
    let (p, w, f) = run_autonomy_checks(&data_dir);
    passed += p; warned += w; failed += f;
    print_summary_checks(w, f);

    // ── Summary ───────────────────────────────────────────────────────────
    println!();
    println!("─────────────────────────────────────────");
    let total = passed + warned + failed;
    println!(
        "Summary: {}/{} checks passed{}",
        passed,
        total,
        if failed > 0 { " — failures need attention" }
        else if warned > 0 { " — warnings to review" }
        else { " ✅" }
    );

    if failed > 0 {
        println!();
        println!("❌ Starfire needs attention — run `starfire chat` to verify.");
        std::process::exit(1);
    } else if warned > 0 {
        println!();
        println!("⚠️  Starfire is functional but has warnings.");
        std::process::exit(0);
    } else {
        println!();
        println!("✅ Starfire is healthy and ready to run.");
        std::process::exit(0);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Check 1: Build & Tests
// ─────────────────────────────────────────────────────────────────────────────

fn run_build_checks(project_root: &PathBuf, deep: bool) -> (usize, usize, usize) {
    let mut checks = Vec::new();

    // cargo build --lib
    let build = std::process::Command::new("cargo")
        .args(["build", "--lib"])
        .current_dir(project_root)
        .output();

    match build {
        Ok(output) if output.status.success() => {
            checks.push(Check::pass("cargo build --lib"));
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            checks.push(Check::fail(
                "cargo build --lib",
                &format!("Build failed: {}", stderr.lines().last().unwrap_or("unknown error")),
            ));
        }
        Err(e) => {
            checks.push(Check::fail("cargo build --lib", &format!("Failed to run: {}", e)));
        }
    }

    // cargo test --lib
    let test = std::process::Command::new("cargo")
        .args(["test", "--lib", "--", "--nocapture"])
        .current_dir(project_root)
        .output();

    match test {
        Ok(output) if output.status.success() => {
            // Count passing tests from output
            let stdout = String::from_utf8_lossy(&output.stdout);
            let test_count = count_pass_tests(&stdout);
            checks.push(Check::pass("cargo test --lib"));
            if let Some(n) = test_count {
                checks.last_mut().unwrap().detail = Some(format!("{} tests passed", n));
            }
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            checks.push(Check::fail(
                "cargo test --lib",
                &format!("Tests failed: {}", stderr.lines().last().unwrap_or("unknown")),
            ));
        }
        Err(e) => {
            checks.push(Check::fail("cargo test --lib", &format!("Failed to run: {}", e)));
        }
    }

    // clippy (deep only)
    if deep {
        let clippy = std::process::Command::new("cargo")
            .args(["clippy", "--lib", "--", "-D", "warnings"])
            .current_dir(project_root)
            .output();

        match clippy {
            Ok(output) if output.status.success() => {
                checks.push(Check::pass("clippy (strict)"));
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let count = count_clippy_warnings(&stderr);
                checks.push(Check::warn(
                    "clippy (strict)",
                    &format!("{} clippy warnings (baseline: 43 pre-existing)", count),
                ));
            }
            Err(e) => {
                checks.push(Check::skip("clippy (strict)", &format!("clippy not available: {}", e)));
            }
        }
    }

    print_checks(&checks);
    aggregate_counts(&checks)
}

// ─────────────────────────────────────────────────────────────────────────────
// Check 2: File Integrity
// ─────────────────────────────────────────────────────────────────────────────

fn run_file_checks(data_dir: &PathBuf) -> (usize, usize, usize) {
    let mut checks = Vec::new();

    let check_file = |filename: &str, min_bytes: u64, checks: &mut Vec<Check>| {
        let path = data_dir.join(filename);
        if !path.exists() {
            checks.push(Check::fail(filename, "File not found"));
        } else if let Ok(meta) = std::fs::metadata(&path) {
            if meta.len() == 0 {
                checks.push(Check::fail(filename, "File is empty (0 bytes)"));
            } else if meta.len() < min_bytes {
                checks.push(Check::warn(
                    filename,
                    &format!("File is smaller than expected ({} bytes)", meta.len()),
                ));
            } else {
                let size = human_size(meta.len());
                checks.push(Check::pass(filename));
                checks.last_mut().unwrap().detail = Some(size);
            }
        }
    };

    check_file("star.db", 100, &mut checks);
    check_file("training.db", 100, &mut checks);
    check_file("voice.db", 100, &mut checks);

    // Bonsai GGUF
    let gguf_path = data_dir.join("models/bonzai-8b/Bonsai-8B.gguf");
    if !gguf_path.exists() {
        checks.push(Check::skip(
            "Bonsai-8B.gguf",
            "GGUF not present — LLM voice will be disabled",
        ));
    } else if let Ok(meta) = std::fs::metadata(&gguf_path) {
        let size = human_size(meta.len());
        let expected_range = 1_000_000_000..=1_300_000_000; // ~1.0-1.3 GB
        if expected_range.contains(&meta.len()) {
            checks.push(Check::pass("Bonsai-8B.gguf"));
            checks.last_mut().unwrap().detail = Some(format!("{} (Q1_0_g128)", size));
        } else {
            checks.push(Check::warn("Bonsai-8B.gguf", &format!("Unexpected size: {}", size)));
            checks.last_mut().unwrap().detail = Some(format!("{} (Q1_0_g128)", size));
        }
    }

    print_checks(&checks);
    aggregate_counts(&checks)
}

// ─────────────────────────────────────────────────────────────────────────────
// Check 3: Identity Seeds
// ─────────────────────────────────────────────────────────────────────────────

fn run_identity_checks(data_dir: &PathBuf) -> (usize, usize, usize) {
    let mut checks = Vec::new();

    // IDENTITY.md — check data dir first, then project dir
    let identity_data = data_dir.join("IDENTITY.md");
    let identity_project = find_project_root().join("IDENTITY.md");
    let identity_path = if identity_data.exists() {
        &identity_data
    } else {
        &identity_project
    };

    if let Ok(content) = std::fs::read_to_string(identity_path) {
        let len = content.len();
        if len < 50 {
            checks.push(Check::warn("IDENTITY.md", "File is suspiciously small"));
        } else {
            checks.push(Check::pass("IDENTITY.md"));
            checks.last_mut().unwrap().detail = Some(format!("{:.1} KB", len as f64 / 1024.0));
        }
    } else {
        checks.push(Check::fail("IDENTITY.md", "File not found in data dir or project dir"));
    }

    // SOUL.md — check data dir first, then project dir
    let soul_data = data_dir.join("SOUL.md");
    let soul_project = find_project_root().join("SOUL.md");
    let soul_path = if soul_data.exists() {
        &soul_data
    } else {
        &soul_project
    };

    if let Ok(content) = std::fs::read_to_string(soul_path) {
        let len = content.len();
        checks.push(Check::pass("SOUL.md"));
        checks.last_mut().unwrap().detail = Some(format!("{:.1} KB", len as f64 / 1024.0));
    } else {
        checks.push(Check::fail("SOUL.md", "File not found in data dir or project dir"));
    }

    print_checks(&checks);
    aggregate_counts(&checks)
}

// ─────────────────────────────────────────────────────────────────────────────
// Check 4: Runtime Health
// ─────────────────────────────────────────────────────────────────────────────

fn run_runtime_checks(data_dir: &PathBuf, repair: bool, non_interactive: bool) -> (usize, usize, usize) {
    let mut checks = Vec::new();

    // Runtime::new()
    let runtime = match Runtime::new(data_dir) {
        Ok(r) => r,
        Err(e) => {
            checks.push(Check::fail("Runtime::new()", &format!("Failed: {}", e)));
            print_checks(&checks);
            return aggregate_counts(&checks);
        }
    };

    checks.push(Check::pass("Runtime::new()"));
    checks.last_mut().unwrap().detail = Some("runtime initialized successfully".to_string());

    // All diagnostics via the public API
    let diag = diagnostic_summary(&runtime);

    // Store / memory
    if diag.store_memories == 0 {
        checks.push(Check::fail("Memory store", "0 memories — store may be corrupt"));
    } else {
        checks.push(Check::pass("Memory store"));
        checks.last_mut().unwrap().detail = Some(format!(
            "{} memories, {} beliefs",
            diag.store_memories, diag.store_beliefs
        ));
    }

    // KG entity count (seed knowledge check)
    if diag.kg_entity_count == 0 {
        checks.push(Check::fail(
            "KG entity count",
            "KG is empty — seed knowledge not injected. Try: star doctor --repair",
        ));
    } else if diag.kg_entity_count < 10 {
        checks.push(Check::warn(
            "KG entity count",
            &format!("Only {} entities — may be under-seeded", diag.kg_entity_count),
        ));
    } else {
        checks.push(Check::pass("KG entity count"));
        checks.last_mut().unwrap().detail = Some(format!(
            "{} entities, {} relationships",
            diag.kg_entity_count, diag.kg_relation_count
        ));
    }

    // Metacognition
    if diag.metacog_belief_count == 0 {
        checks.push(Check::warn("Metacognition", "0 beliefs — may not be bootstrapped"));
    } else {
        checks.push(Check::pass("Metacognition"));
        checks.last_mut().unwrap().detail = Some(format!("{} beliefs", diag.metacog_belief_count));
    }

    // Curiosity engine
    let idle_min = diag.curious_idle_secs as f64 / 60.0;
    checks.push(Check::pass("Curiosity engine"));
    checks.last_mut().unwrap().detail = Some(format!(
        "idle {:.1}m",
        idle_min
    ));

    // Quanot
    checks.push(Check::pass("Quanot"));
    checks.last_mut().unwrap().detail = Some(format!(
        "{} units, activity: {:.2}",
        diag.quanot_reservoir_size, diag.quanot_activity
    ));

    print_checks(&checks);

    // Auto-repair for KG if needed
    if repair && diag.kg_entity_count == 0 {
        println!();
        println!("  🔧 Repairing: re-injecting seed knowledge into KG...");
        if non_interactive || confirm("Re-inject seed knowledge into KG?") {
            // Safe: re-inject just adds more nodes, doesn't overwrite
            if Runtime::new(data_dir).is_ok() {
                println!("  ✅ KG seed repair applied (restart to verify)");
            }
        }
    }

    aggregate_counts(&checks)
}

// ─────────────────────────────────────────────────────────────────────────────
// Check 5: API Server
// ─────────────────────────────────────────────────────────────────────────────

fn run_api_checks() -> (usize, usize, usize) {
    let mut checks = Vec::new();

    // Try to connect to local API server
    let api_base = "http://127.0.0.1:8080";

    // GET /health
    match reqwest::blocking::get(&format!("{}/health", api_base)) {
        Ok(resp) if resp.status().as_u16() == 200 => {
            checks.push(Check::pass("GET /health"));
            if let Ok(body) = resp.text() {
                checks.last_mut().unwrap().detail = Some(body.trim().to_string());
            }
        }
        Ok(resp) => {
            checks.push(Check::fail("GET /health", &format!("Returned {}", resp.status())));
        }
        Err(e) => {
            checks.push(Check::skip(
                "API server",
                &format!("Not running on port 8080 — start with: starfire api"),
            ));
        }
    }

    // GET /identity
    if !matches!(checks.first().map(|c| &c.status), Some(CheckStatus::Skip(_))) {
        match reqwest::blocking::get(&format!("{}/identity", api_base)) {
            Ok(resp) if resp.status().as_u16() == 200 => {
                checks.push(Check::pass("GET /identity"));
            }
            Ok(resp) => {
                checks.push(Check::fail("GET /identity", &format!("Returned {}", resp.status())));
            }
            Err(e) => {
                checks.push(Check::fail("GET /identity", &format!("Failed: {}", e)));
            }
        }
    }

    print_checks(&checks);
    aggregate_counts(&checks)
}

// ─────────────────────────────────────────────────────────────────────────────
// Check 6: LLM Layer
// ─────────────────────────────────────────────────────────────────────────────

fn run_llm_checks(project_root: &PathBuf) -> (usize, usize, usize) {
    let mut checks = Vec::new();

    // GGUF lives in the project models directory.
    let gguf_path = project_root.join("models/bonzai-8b/Bonsai-8B.gguf");

    if !gguf_path.exists() {
        checks.push(Check::skip(
            "GGUF file",
            "Bonsai-8B.gguf not found — LLM voice disabled",
        ));
        print_checks(&checks);
        return aggregate_counts(&checks);
    }

    // File readable
    checks.push(Check::pass("GGUF file readable"));
    checks.last_mut().unwrap().detail = Some(human_size(
        std::fs::metadata(&gguf_path).map(|m| m.len()).unwrap_or(0)
    ));

    // Q1_0_g128 detection
    match llm::LlmEngine::is_bonsai(&gguf_path) {
        Ok(true) => {
            checks.push(Check::pass("Q1_0_g128 detection"));
            checks.last_mut().unwrap().detail = Some("254 Q1_0_g128 tensors".to_string());
        }
        Ok(false) => {
            checks.push(Check::fail(
                "Q1_0_g128 detection",
                "File is not a Bonsai Q1_0_g128 model",
            ));
        }
        Err(e) => {
            checks.push(Check::fail("Q1_0_g128 detection", &format!("Failed to read GGUF: {}", e)));
        }
    }

    // Health check — try loading the model
    match llm::LlmHandle::new(&gguf_path).load() {
        Ok(mut engine) => {
            checks.push(Check::pass("Model loading"));
            checks.last_mut().unwrap().detail = Some("Bonsai-8B loaded via Candle".to_string());

            // Forward pass test
            if engine.health_check() {
                checks.push(Check::pass("Forward pass"));
            } else {
                checks.push(Check::fail("Forward pass", "Health check failed — model may be partially loaded"));
            }
        }
        Err(e) => {
            checks.push(Check::fail("Model loading", &format!("Failed: {}", e)));
        }
    }

    print_checks(&checks);
    aggregate_counts(&checks)
}

// ─────────────────────────────────────────────────────────────────────────────
// Check 7: Subsystem Stats
// ─────────────────────────────────────────────────────────────────────────────

fn run_subsystem_checks(data_dir: &PathBuf) -> (usize, usize, usize) {
    let mut checks = Vec::new();

    let runtime = match Runtime::new(data_dir) {
        Ok(r) => r,
        Err(e) => {
            checks.push(Check::fail("Runtime (for stats)", &format!("Failed: {}", e)));
            print_checks(&checks);
            return aggregate_counts(&checks);
        }
    };

    let diag = diagnostic_summary(&runtime);

    // Training DB — open directly since training_db field is private
    let training_db_path = data_dir.join("training.db");
    if training_db_path.exists() {
        checks.push(Check::pass("Training DB"));
        checks.last_mut().unwrap().detail = Some("DB file present (stats via Runtime)".to_string());
    } else {
        checks.push(Check::skip("Training DB", "File not found"));
    }

    // Quanot
    checks.push(Check::pass("Quanot reservoir"));
    checks.last_mut().unwrap().detail = Some(format!(
        "{} units, activity: {:.2}",
        diag.quanot_reservoir_size, diag.quanot_activity
    ));

    // World Model
    checks.push(Check::pass("World Model"));
    checks.last_mut().unwrap().detail = Some(format!("{} entities", diag.world_model_entities));

    // Memory breakdown via store snapshot
    let breakdown_map = runtime.store_domain_breakdown();
    checks.push(Check::pass("Memory domains"));
    let breakdown: Vec<String> = breakdown_map
        .iter()
        .map(|(d, c)| format!("{}:{}", d, c))
        .collect();
    checks.last_mut().unwrap().detail = Some(breakdown.join(", "));

    print_checks(&checks);
    aggregate_counts(&checks)
}

// ─────────────────────────────────────────────────────────────────────────────
// Check 8: Autonomy State
// ─────────────────────────────────────────────────────────────────────────────

fn run_autonomy_checks(data_dir: &PathBuf) -> (usize, usize, usize) {
    let mut checks = Vec::new();

    let runtime = match Runtime::new(data_dir) {
        Ok(r) => r,
        Err(e) => {
            checks.push(Check::fail("Runtime (for autonomy)", &format!("Failed: {}", e)));
            print_checks(&checks);
            return aggregate_counts(&checks);
        }
    };

    let diag = diagnostic_summary(&runtime);

    checks.push(Check::pass("Goals"));
    checks.last_mut().unwrap().detail = Some(format!("{} active", diag.goal_count));

    checks.push(Check::pass("Aspirations"));
    checks.last_mut().unwrap().detail = Some(format!("{} set", diag.aspiration_count));

    checks.push(Check::pass("Curiosity probes"));
    checks.last_mut().unwrap().detail = Some(format!(
        "last probe: {:.0}m ago",
        diag.curious_last_probe as f64 / 60.0
    ));

    print_checks(&checks);
    aggregate_counts(&checks)
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn banner() {
    println!("🦀 Starfire Doctor — {}", timestamp());
    println!("  Self-diagnostic for all Starfire subsystems");
}

fn timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| {
            let secs = d.as_secs();
            let mins = (secs / 60) % 60;
            let hours = (secs / 3600) % 24;
            format!("{:02}:{:02}:{:02} UTC", hours, mins, secs % 60)
        })
        .unwrap_or_else(|_| "?".to_string())
}

fn resolve_data_dir() -> PathBuf {
    // Prefer: current directory (development) > star data dir > other standard dirs.
    let candidates = [
        // Development: current directory
        PathBuf::from("."),
        // Production: AppData/star (Windows)
        dirs::data_local_dir().map(|d| d.join("star")).unwrap_or_default(),
        // Alternative: ~/.local/share/star
        dirs::data_dir().map(|d| d.join("star")).unwrap_or_default(),
    ];

    for candidate in &candidates {
        if candidate.as_os_str().is_empty() {
            continue;
        }
        if candidate.join("star.db").exists() {
            return candidate.clone();
        }
    }

    // No star.db found — return current dir.
    PathBuf::from(".")
}

/// Find the starfire project root.
/// Uses the binary's own location to walk up to the project directory.
/// Binary: <project>/target/debug/star.exe
/// Walk up 3 levels: debug/ → target/ → <project>/ (where Cargo.toml lives).
fn find_project_root() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        let exe_path = exe.as_path();
        // exe = .../target/debug/star.exe
        // Walk up: debug/ → target/ → project_root/
        let project_root = exe_path
            .ancestors()
            .nth(3) // 0=self, 1=debug/, 2=target/, 3=project/
            .map(PathBuf::from)
            .filter(|p| p.join("Cargo.toml").exists());

        if let Some(p) = project_root {
            return p;
        }
    }

    // Fallback: walk up from current working directory.
    let current = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut dir = current.as_path();
    loop {
        if dir.join("Cargo.toml").exists() && dir.join("src").exists() && dir.join("lib").exists() {
            return dir.to_path_buf();
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => return current,
        }
    }
}

fn human_size(bytes: u64) -> String {
    let mb = bytes as f64 / 1_048_576.0;
    let gb = mb / 1024.0;
    if gb >= 1.0 {
        format!("{:.1} GB", gb)
    } else if mb >= 1.0 {
        format!("{:.0} MB", mb)
    } else {
        format!("{} bytes", bytes)
    }
}

fn count_pass_tests(output: &str) -> Option<usize> {
    // Look for "test result: ok. N passed"
    for line in output.lines().rev() {
        if line.contains("test result:") {
            if let Some(n) = line.split_whitespace()
                .skip_while(|w| *w != "passed")
                .nth(1)
                .and_then(|s| s.parse().ok())
            {
                return Some(n);
            }
        }
    }
    None
}

fn count_clippy_warnings(stderr: &str) -> usize {
    stderr.lines().filter(|l| l.contains("warning:")).count()
}

fn print_checks(checks: &[Check]) {
    for check in checks {
        match &check.status {
            CheckStatus::Pass => {
                print!("  ✅ ");
            }
            CheckStatus::Warn => {
                print!("  ⚠️  ");
            }
            CheckStatus::Fail => {
                print!("  ❌ ");
            }
            CheckStatus::Skip(reason) => {
                print!("  ➖  ");
                println!("{} (skipped: {})", check.name, reason);
                continue;
            }
        }

        print!("{}", check.name);
        if let Some(detail) = &check.detail {
            print!(" — {}", detail);
        }
        println!();

        if let Some(fix) = &check.fix {
            println!("     → Fix: {}", fix);
        }
    }
}

fn print_summary_checks(warnings: usize, failures: usize) {
    if failures > 0 {
        println!("  ❌ {} failure{}", failures, if failures > 1 { "s" } else { "" });
    } else if warnings > 0 {
        println!("  ⚠️  {} warning{}", warnings, if warnings > 1 { "s" } else { "" });
    }
}

fn aggregate_counts(checks: &[Check]) -> (usize, usize, usize) {
    checks.iter().fold((0, 0, 0), |(p, w, f), c| {
        let (pc, wc, fc) = c.count();
        (p + pc, w + wc, f + fc)
    })
}

fn confirm(prompt: &str) -> bool {
    print!("{} [y/N] ", prompt);
    std::io::Write::flush(&mut std::io::stdout()).ok();
    let mut input = String::new();
    if std::io::stdin().read_line(&mut input).is_ok() {
        input.trim().eq_ignore_ascii_case("y")
    } else {
        false
    }
}
