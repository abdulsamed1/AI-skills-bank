use std::path::PathBuf;
use std::time::Instant;
use std::{env, fs};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arg_repo = env::args().nth(1);
    let repo_root = arg_repo
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".."));

    let batch_sizes: Vec<usize> = vec![8, 16, 32, 64];
    let concurrencies: Vec<usize> = vec![1, 4, 8, 16];

    println!("LLM benchmark: repo_root={} \n", repo_root.display());

    for &batch in &batch_sizes {
        for &concur in &concurrencies {
            env::set_var("LLM_ENABLED", "true");
            env::set_var("LLM_PROVIDER", "mock");
            env::set_var("LLM_API_KEY", "dummy");
            env::set_var("LLM_BATCH_SIZE", batch.to_string());
            env::set_var("LLM_CONCURRENCY", concur.to_string());

            let output_dir = repo_root.join(format!("target/llm-bench/{}_{}", batch, concur));
            if output_dir.exists() {
                let _ = fs::remove_dir_all(&output_dir);
            }

            println!("Running benchmark: batch={} concurrency={}...", batch, concur);

            // Run the `skill-manage` binary as a subprocess (mirrors typical user flow).
            let bin_path = PathBuf::from("target").join("debug").join(if cfg!(windows) { "skill-manage.exe" } else { "skill-manage" });
            let start = Instant::now();
            let output = std::process::Command::new(&bin_path)
                .arg("aggregate")
                .current_dir(&repo_root)
                .env("LLM_ENABLED", "true")
                .env("LLM_PROVIDER", "mock")
                .env("LLM_API_KEY", "dummy")
                .env("LLM_BATCH_SIZE", batch.to_string())
                .env("LLM_CONCURRENCY", concur.to_string())
                .output();

            match output {
                Ok(out) => {
                    let dur = start.elapsed();
                    if out.status.success() {
                        // Try to parse skills count from stdout if present
                        let stdout = String::from_utf8_lossy(&out.stdout);
                        // Print a short snippet and duration
                        println!("Result: batch={} concur={} -> time: {:.2?}\n", batch, concur, dur);
                        // Optionally, print captured stdout for debugging
                        // println!("stdout: {}", stdout);
                    } else {
                        let stderr = String::from_utf8_lossy(&out.stderr);
                        eprintln!("Process failed: {}\nstderr: {}", out.status, stderr);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to spawn process: {}", e);
                }
            }
        }
    }

    Ok(())
}
