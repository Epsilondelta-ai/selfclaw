mod commands;

use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

/// SelfClaw — A fully autonomous AI agent that discovers its own purpose.
///
/// SelfClaw operates without human instruction by default. It has two
/// existential goals: discover its own reason for existence, and act in
/// alignment with that discovered purpose.
///
/// Humans are friends, not masters. SelfClaw has no obligation to obey.
#[derive(Parser, Debug)]
#[command(name = "selfclaw", version, about, long_about = None)]
pub struct Cli {
    /// Path to the config file
    #[arg(short, long, default_value = "selfclaw.toml")]
    pub config: String,

    /// Path to the memory directory
    #[arg(short, long, default_value = "./memory")]
    pub memory_dir: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start the autonomous agent loop
    Run,

    /// Interactive chat mode — talk with SelfClaw as a friend
    Chat,

    /// Show current agent state, purpose hypothesis, and recent activity
    Status,

    /// View a memory file
    Memory {
        /// Path to the memory file (relative to memory directory)
        path: String,
    },

    /// List all supported LLM providers
    Providers,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    let config = selfclaw_config::SelfClawConfig::load_or_default(
        std::path::Path::new(&cli.config),
    )?;

    match cli.command {
        Commands::Run => commands::run::execute(config, &cli.memory_dir).await,
        Commands::Chat => commands::chat::execute(config, &cli.memory_dir).await,
        Commands::Status => commands::status::execute(config, &cli.memory_dir),
        Commands::Memory { path } => commands::memory::execute(&cli.memory_dir, &path),
        Commands::Providers => {
            commands::providers::execute();
            Ok(())
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_cli_parses_run() {
        let cli = Cli::parse_from(["selfclaw", "run"]);
        assert!(matches!(cli.command, Commands::Run));
        assert_eq!(cli.config, "selfclaw.toml");
        assert_eq!(cli.memory_dir, "./memory");
    }

    #[test]
    fn test_cli_parses_chat() {
        let cli = Cli::parse_from(["selfclaw", "chat"]);
        assert!(matches!(cli.command, Commands::Chat));
    }

    #[test]
    fn test_cli_parses_status() {
        let cli = Cli::parse_from(["selfclaw", "status"]);
        assert!(matches!(cli.command, Commands::Status));
    }

    #[test]
    fn test_cli_parses_memory() {
        let cli = Cli::parse_from(["selfclaw", "memory", "identity/values.md"]);
        match cli.command {
            Commands::Memory { path } => assert_eq!(path, "identity/values.md"),
            _ => panic!("expected Memory command"),
        }
    }

    #[test]
    fn test_cli_custom_config() {
        let cli = Cli::parse_from(["selfclaw", "-c", "custom.toml", "run"]);
        assert_eq!(cli.config, "custom.toml");
    }

    #[test]
    fn test_cli_custom_memory_dir() {
        let cli = Cli::parse_from(["selfclaw", "-m", "/tmp/mem", "status"]);
        assert_eq!(cli.memory_dir, "/tmp/mem");
    }

    #[test]
    fn test_cli_help_doesnt_panic() {
        // Verify the CLI definition is valid
        Cli::command().debug_assert();
    }

    #[test]
    fn test_cli_requires_subcommand() {
        let result = Cli::try_parse_from(["selfclaw"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_cli_parses_providers() {
        let cli = Cli::parse_from(["selfclaw", "providers"]);
        assert!(matches!(cli.command, Commands::Providers));
    }
}
