//! `sutra status` — Show current Catuṣkoṭi node states.

use axiom_gitvfs::AxiomRepo;
use axiom_verify::HealthState;
use std::path::Path;

pub async fn run(repo_path: &str) -> anyhow::Result<()> {
    println!("🕉  SūtraOS Status — Node Catuṣkoṭi States");
    println!("   Repository: {repo_path}");
    println!();

    if !Path::new(repo_path).exists() {
        println!("⚠️  Repository not found. Run 'sutra init {repo_path}' first.");
        return Ok(());
    }

    let repo = AxiomRepo::open(repo_path)
        .map_err(|e| anyhow::anyhow!("Cannot open repo: {}", e))?;

    let head_sha = repo.head_sha()?;
    println!("   HEAD: {}", &head_sha[..12]);
    println!();

    println!("   {:<30} {:<30}", "Node ID", "Catuṣkoṭi State");
    println!("   {}", "─".repeat(62));

    let iter_ids = repo.list_nodes().unwrap_or_else(|_| vec![]);
    let iter_ids = if iter_ids.is_empty() {
        vec!["k8s-worker-1".to_string(), "aws-us-east-1".to_string(),
             "pg-primary".to_string(), "auth-service".to_string()]
    } else {
        iter_ids
    };

    for node_id in &iter_ids {
        match repo.read_node(node_id) {
            Ok(node) => {
                let state_str = match node.state {
                    HealthState::Asti => "✅ Asti (Healthy)",
                    HealthState::Nasti => "❌ Nāsti (Faulted)",
                    HealthState::Ubhaya => "⚠️  Ubhaya (Flapping)",
                    HealthState::Avaktavya => "🔕 Avaktavya (Unobservable — LOCKED)",
                };
                println!("   {:<30} {}", node.id, state_str);
            }
            Err(_) => {
                println!("   {:<30} (not found in ontology)", node_id);
            }
        }
    }

    println!();
    println!("   Use 'sutra audit' to inspect the full Nyāya merge history.");

    Ok(())
}
