use selfclaw_tools::llm::ProviderKind;

/// List all supported LLM providers.
pub fn execute() {
    println!("Supported LLM Providers");
    println!("=======================");
    println!();
    println!(
        "{:<14} {:<42} {:<22} {:<30}",
        "PROVIDER", "DEFAULT MODEL", "ENV VAR", "DEFAULT BASE URL"
    );
    println!("{}", "-".repeat(110));

    for kind in ProviderKind::all() {
        println!(
            "{:<14} {:<42} {:<22} {}",
            kind.name(),
            kind.default_model(),
            kind.api_key_env_var(),
            kind.default_base_url(),
        );
    }

    println!();
    println!("Any unrecognized provider name uses the \"custom\" (OpenAI-compatible) format.");
    println!("Set llm.base_url in selfclaw.toml to point to your custom endpoint.");
    println!();
    println!("Configuration example:");
    println!("  [llm]");
    println!("  provider = \"openai\"");
    println!("  model = \"gpt-5.2\"");
    println!("  # api_key = \"sk-...\"        # optional, overrides env var");
    println!("  # base_url = \"https://...\"   # optional, overrides default");
}
