//! `sutra init` — Initialise a SūtraOS state repository.

use sutra_gitvfs::{InfraNode, SutraRepo};
use sutra_nyaya::CatuskotiState;
use tracing::info;

pub async fn run(dir: &str) -> anyhow::Result<()> {
    println!("🕉  SūtraOS — Initialising state repository at: {dir}");

    let repo = SutraRepo::init(dir)?;

    // Seed sample ontology nodes
    let nodes = vec![
        {
            let mut n = InfraNode::new("k8s-worker-1", "Kubernetes Worker Node 1", "kubernetes_node");
            n.metadata.insert("namespace".to_string(), "production".to_string());
            n.metadata.insert("region".to_string(), "us-east-1".to_string());
            n.state = CatuskotiState::Asti;
            n
        },
        {
            let mut n = InfraNode::new("aws-us-east-1", "AWS US-East-1 Region", "aws_region");
            n.metadata.insert("provider".to_string(), "aws".to_string());
            n.state = CatuskotiState::Asti;
            n
        },
        {
            let mut n = InfraNode::new("pg-primary", "PostgreSQL Primary", "postgres_primary");
            n.metadata.insert("version".to_string(), "16".to_string());
            n.depends_on.push("aws-us-east-1".to_string());
            n.state = CatuskotiState::Asti;
            n
        },
        {
            let mut n = InfraNode::new("auth-service", "Auth Service Pod", "kubernetes_pod");
            n.metadata.insert("namespace".to_string(), "production".to_string());
            n.depends_on.push("k8s-worker-1".to_string());
            n.depends_on.push("pg-primary".to_string());
            n.state = CatuskotiState::Asti;
            n
        },
    ];

    for node in &nodes {
        repo.upsert_node(node)?;
        info!(node = %node.id, state = ?node.state, "Seeded ontology node");
    }

    // Seed a sample runbook
    let runbook = serde_json::json!({
        "id": "runbook-oom-restart-v1",
        "title": "Kubernetes Pod OOM Crash Loop Remediation",
        "vyapti_rule": "When NIC firmware drops packets while CPU < 20%, draining the host resolves latency without restarting the database",
        "pratijna_template": "Increase memory limit and restart pod",
        "nigamana_cmd": "kubectl get pod auth-service -o json | jq .status.phase",
        "version": "1"
    });
    let runbook_bytes = serde_json::to_vec_pretty(&runbook)?;
    repo.write_to_main(
        "runbooks/sutra/runbook-oom-restart-v1.json",
        &runbook_bytes,
        "docs: add OOM restart runbook (Vyāpti rule #42)",
    )?;

    println!("✅ Initialised SūtraOS state repository");
    println!("   Nodes seeded: {}", nodes.iter().map(|n| n.id.as_str()).collect::<Vec<_>>().join(", "));
    println!("   Runbooks: runbook-oom-restart-v1");
    println!("   Git VFS at: {dir}");
    println!();
    println!("Next steps:");
    println!("   sutra status --repo {dir}");
    println!("   sutra trigger event.json");

    Ok(())
}
