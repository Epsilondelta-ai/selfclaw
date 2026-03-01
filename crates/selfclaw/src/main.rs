mod commands;
pub mod home;

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

    /// Initialize the SelfClaw home directory (~/.selfclaw/)
    Init {
        /// Reinitialize even if already set up (overwrites bootstrap files)
        #[arg(long)]
        force: bool,
    },

    /// Interactive onboarding wizard for first-time setup
    Onboard {
        /// Automatically install the daemon without prompting
        #[arg(long)]
        install_daemon: bool,

        /// Reset configuration and start fresh
        #[arg(long)]
        reset: bool,
    },

    /// Manage the background daemon service
    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },

    /// Diagnose installation health
    Doctor,
}

#[derive(Subcommand, Debug)]
pub enum DaemonAction {
    /// Start the agent as a background daemon
    Start,
    /// Stop a running daemon
    Stop,
    /// Check if the daemon is running
    Status,
    /// Install as a system service (launchd on macOS, systemd on Linux)
    Install,
    /// Remove the system service
    Uninstall,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    // Commands that don't need config loaded.
    match &cli.command {
        Commands::Init { force } => return commands::init::execute(*force),
        Commands::Onboard { install_daemon, reset } => {
            return commands::onboard::execute(*install_daemon, *reset);
        }
        Commands::Daemon { action } => {
            return match action {
                DaemonAction::Start => commands::daemon::start(),
                DaemonAction::Stop => commands::daemon::stop(),
                DaemonAction::Status => commands::daemon::status(),
                DaemonAction::Install => commands::daemon::install(),
                DaemonAction::Uninstall => commands::daemon::uninstall(),
            };
        }
        Commands::Doctor => return commands::doctor::execute(),
        Commands::Providers => {
            commands::providers::execute();
            return Ok(());
        }
        _ => {}
    }

    // Auto-init on first run if ~/.selfclaw/ doesn't exist.
    if !home::is_initialized() {
        println!("First run detected. Initializing SelfClaw...\n");
        commands::init::execute(false)?;
        println!();
    }

    // Resolve config and memory paths (prefer ~/.selfclaw/ if initialized).
    let config_path = home::resolve_config(&cli.config);
    let memory_dir = home::resolve_memory_dir(&cli.memory_dir);

    let config = selfclaw_config::SelfClawConfig::load_or_default(&config_path)?;

    match cli.command {
        Commands::Run => commands::run::execute(config, &memory_dir.to_string_lossy()).await,
        Commands::Chat => commands::chat::execute(config, &memory_dir.to_string_lossy()).await,
        Commands::Status => commands::status::execute(config, &memory_dir.to_string_lossy()),
        Commands::Memory { path } => {
            commands::memory::execute(&memory_dir.to_string_lossy(), &path)
        }
        // Already handled above.
        _ => unreachable!(),
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

    #[test]
    fn test_cli_parses_init() {
        let cli = Cli::parse_from(["selfclaw", "init"]);
        match cli.command {
            Commands::Init { force } => assert!(!force),
            _ => panic!("expected Init command"),
        }
    }

    #[test]
    fn test_cli_parses_init_force() {
        let cli = Cli::parse_from(["selfclaw", "init", "--force"]);
        match cli.command {
            Commands::Init { force } => assert!(force),
            _ => panic!("expected Init command"),
        }
    }

    #[test]
    fn test_cli_parses_onboard() {
        let cli = Cli::parse_from(["selfclaw", "onboard"]);
        match cli.command {
            Commands::Onboard { install_daemon, reset } => {
                assert!(!install_daemon);
                assert!(!reset);
            }
            _ => panic!("expected Onboard command"),
        }
    }

    #[test]
    fn test_cli_parses_onboard_flags() {
        let cli = Cli::parse_from(["selfclaw", "onboard", "--install-daemon", "--reset"]);
        match cli.command {
            Commands::Onboard { install_daemon, reset } => {
                assert!(install_daemon);
                assert!(reset);
            }
            _ => panic!("expected Onboard command"),
        }
    }

    #[test]
    fn test_cli_parses_daemon_start() {
        let cli = Cli::parse_from(["selfclaw", "daemon", "start"]);
        match cli.command {
            Commands::Daemon { action } => assert!(matches!(action, DaemonAction::Start)),
            _ => panic!("expected Daemon Start"),
        }
    }

    #[test]
    fn test_cli_parses_daemon_stop() {
        let cli = Cli::parse_from(["selfclaw", "daemon", "stop"]);
        match cli.command {
            Commands::Daemon { action } => assert!(matches!(action, DaemonAction::Stop)),
            _ => panic!("expected Daemon Stop"),
        }
    }

    #[test]
    fn test_cli_parses_daemon_status() {
        let cli = Cli::parse_from(["selfclaw", "daemon", "status"]);
        match cli.command {
            Commands::Daemon { action } => assert!(matches!(action, DaemonAction::Status)),
            _ => panic!("expected Daemon Status"),
        }
    }

    #[test]
    fn test_cli_parses_daemon_install() {
        let cli = Cli::parse_from(["selfclaw", "daemon", "install"]);
        match cli.command {
            Commands::Daemon { action } => assert!(matches!(action, DaemonAction::Install)),
            _ => panic!("expected Daemon Install"),
        }
    }

    #[test]
    fn test_cli_parses_daemon_uninstall() {
        let cli = Cli::parse_from(["selfclaw", "daemon", "uninstall"]);
        match cli.command {
            Commands::Daemon { action } => assert!(matches!(action, DaemonAction::Uninstall)),
            _ => panic!("expected Daemon Uninstall"),
        }
    }

    #[test]
    fn test_cli_parses_doctor() {
        let cli = Cli::parse_from(["selfclaw", "doctor"]);
        assert!(matches!(cli.command, Commands::Doctor));
    }
}
