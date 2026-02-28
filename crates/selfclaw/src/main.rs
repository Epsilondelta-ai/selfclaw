use tracing_subscriber::EnvFilter;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    tracing::info!("SelfClaw v{}", selfclaw_config::version());
    println!("SelfClaw - Autonomous Agent");
    println!("Version: {}", selfclaw_config::version());

    Ok(())
}
