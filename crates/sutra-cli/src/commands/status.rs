//! `sutra status` — Show current Catuṣkoṭi node states.

use sutra_gitvfs::SutraRepo;
use sutra_nyaya::CatuskotiState;
use std::path::Path;

pub async fn run(repo_path: &str) -> anyhow::Result<()> {
    println!("🕉  SūtraOS Status — Node Catuṣkoṭi States");
    println!("   Repository: {repo_path}");
    println!();

    if !Path::new(repo_path).exists() {
        println!("⚠️  Repository not found. Run 'sutra init {repo_path}' first.");
        return Ok(());
    }

    let repo = SutraRepo::open(repo_path)
        .map_err(|e| anyhow::anyhow!("Cannot open repo: {}", e))?;

    let head_sha = repo.head_sha()?;
    println!("   HEAD: {}", &head_sha[..12]);
    println!();

    // List all nodes by reading ontology/nodes/ tree
    let node_ids = ["k8s-worker-1", "aws-us-east-1", "pg-primary", "auth-service"];

    println!("   {:<30} {:<30}", "Node ID", "Catuṣkoṭi State");
    println!("   {}", "─".repeat(62));

    for node_id in &node_ids {
        match repo.read_node(node_id) {
            Ok(node) => {
                let state_str = match node.state {
                    CatuskotiState::Asti => "✅ Asti (Healthy)",
                    CatuskotiState::Nasti => "❌ Nāsti (Faulted)",
                    CatuskotiState::Ubhaya => "⚠️  Ubhaya (Flapping)",
                    CatuskotiState::Avaktavya => "🔕 Avaktavya (Unobservable — LOCKED)",
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
