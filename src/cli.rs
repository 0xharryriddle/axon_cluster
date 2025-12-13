//! Cli

use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "axon_cluster")]
#[command(about = "Axon-Cluster: Private P2P AI Inference Network", long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub mode: Mode,
}

#[derive(Debug, Parser)]
pub enum Mode {
    /// Leader mode: Listen for inference requests and process them with Ollama
    #[command(name = "serve")]
    Serve {
        /// Ollama API endpoint (default: http://127.0.0.1:11434)
        #[arg(long, default_value = "http://127.0.0.1:11434")]
        ollama_url: String,

        /// Model name to use (default: qwen:0.5b)
        #[arg(long, default_value = "qwen:0.5b")]
        model: String,
    },

    /// Subordinate mode: Send an inference request to the Leader
    #[command(name = "ask")]
    Ask {
        /// The prompt to send for inference
        prompt: String,
    },
}

impl Args {
    pub async fn run(self) -> Result<()> {
        Ok(())
    }
}
