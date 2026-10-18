//! `sutra` — SūtraOS Command-Line Interface
//!
//! Commands:
//! - `sutra init <dir>`        — Initialise a Git VFS with sample ontology
//! - `sutra trigger <file>`    — Fire a simulated event into the kernel
//! - `sutra audit`             — Walk main commit history and validate Nyāya proofs
//! - `sutra status`            — Show current node Catuṣkoṭi states

mod commands;

use clap::{Parser, Subcommand};
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(
    name = "sutra",
    about = "SūtraOS CLI — AI-Native DevOps Self-Healing Kernel",
    version = env!("CARGO_PKG_VERSION"),
    long_about = "SūtraOS: An AI-native headless OS kernel backed by Ancient Indian formal mathematics.\n\
                  \nUses Pāṇinian grammar, Navya-Nyāya logic, and Catuṣkoṭi state lattices\n\
                  for 24x7 autonomous DevOps self-healing."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialise a SūtraOS state repository with sample ontology nodes and runbooks.
    Init {
        /// Directory to initialise (default: ./axiom_state.git)
        #[arg(default_value = "axiom_state.git")]
        dir: String,
    },
    /// Fire a telemetry event into the kernel from a JSON file.
    Trigger {
        /// Path to event JSON file (or `-` for stdin)
        #[arg(default_value = "-")]
        event_file: String,
    },
    /// Audit the full commit history of main, validating every CausalProof.
    Audit {
        /// Path to the state repository (default: ./axiom_state.git)
        #[arg(long, default_value = "axiom_state.git")]
        repo: String,
    },
    /// Show current Catuṣkoṭi node states.
    Status {
        /// Path to the state repository (default: ./axiom_state.git)
        #[arg(long, default_value = "axiom_state.git")]
        repo: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env().add_directive("sutra=info".parse()?),
        )
        .compact()
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Init { dir } => commands::init::run(&dir).await,
        Commands::Trigger { event_file } => commands::trigger::run(&event_file).await,
        Commands::Audit { repo } => commands::audit::run(&repo).await,
        Commands::Status { repo } => commands::status::run(&repo).await,
    }
}
