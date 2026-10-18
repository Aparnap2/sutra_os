//! `sutra audit` — Walk main commit history and validate CausalProofs.

use axiom_gitvfs::AxiomRepo;
use axiom_verify::CausalProof;

pub async fn run(repo_path: &str) -> anyhow::Result<()> {
    println!("🕉  SūtraOS Audit — Walking commit history at: {repo_path}");

    let repo = AxiomRepo::open(repo_path)
        .map_err(|e| anyhow::anyhow!("Cannot open repo '{}': {}", repo_path, e))?;

    let head_sha = repo.head_sha()?;
    println!("   HEAD (main): {head_sha}");
    println!();

    // Walk the git log
    let git_repo = repo.git_repo();
    let mut revwalk = git_repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.set_sorting(git2::Sort::TIME)?;

    let mut total = 0;
    let mut nyaya_commits = 0;
    let mut invalid = 0;

    for oid_result in revwalk {
        let oid = oid_result?;
        let commit = git_repo.find_commit(oid)?;
        let message = commit.message().unwrap_or("").to_string();
        total += 1;

        if message.contains("[causal-merge]") {
            nyaya_commits += 1;
            // Extract CausalProof JSON from message
            if let Some(proof_json) = extract_proof_json(&message) {
                match serde_json::from_str::<CausalProof>(&proof_json) {
                    Ok(proof) => {
                        match proof.verify_structure() {
                            Ok(()) => {
                                println!("✅ {} — Nyāya proof valid: {}", &oid.to_string()[..8], proof.pratijna_branch);
                            }
                            Err(e) => {
                                invalid += 1;
                                println!("❌ {} — Proof INVALID: {}", &oid.to_string()[..8], e);
                            }
                        }
                    }
                    Err(e) => {
                        invalid += 1;
                        println!("❌ {} — Cannot parse proof JSON: {}", &oid.to_string()[..8], e);
                    }
                }
            } else {
                println!("⚠️  {} — [causal-merge] commit but no proof JSON found", &oid.to_string()[..8]);
            }
        } else {
            println!("   {} — {}", &oid.to_string()[..8], first_line(&message));
        }
    }

    println!();
    println!("── Audit Summary ──────────────────────────────────────");
    println!("   Total commits:     {total}");
    println!("   Nyāya merge commits: {nyaya_commits}");
    println!("   Invalid proofs:    {invalid}");
    if invalid == 0 {
        println!("   Status: ✅ All proofs valid");
    } else {
        println!("   Status: ❌ {invalid} invalid proof(s) found");
    }

    Ok(())
}

fn extract_proof_json(message: &str) -> Option<String> {
    let marker = "Causal-Proof:\n";
    let start = message.find(marker)? + marker.len();
    Some(message[start..].trim().to_string())
}

fn first_line(s: &str) -> &str {
    s.lines().next().unwrap_or(s)
}
