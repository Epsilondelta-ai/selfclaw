//! `selfclaw daemon` — Background service management.
//!
//! Subcommands:
//! - `start`    — Start the agent as a background daemon
//! - `stop`     — Stop a running daemon
//! - `status`   — Check if the daemon is running
//! - `install`  — Install as a system service (launchd / systemd)
//! - `uninstall` — Remove the system service

use crate::home;
use std::fs;
use std::path::Path;
use std::process::Command;

// ── Start ───────────────────────────────────────────────────────────

pub fn start() -> anyhow::Result<()> {
    if is_running() {
        let pid = read_pid()?;
        println!("SelfClaw daemon is already running (PID: {}).", pid);
        return Ok(());
    }

    let exe = std::env::current_exe()?;
    let config_path = home::resolve_config("selfclaw.toml");
    let memory_dir = home::resolve_memory_dir("./memory");
    let log_file = home::daemon_log_file();

    // Ensure log directory exists.
    if let Some(parent) = log_file.parent() {
        fs::create_dir_all(parent)?;
    }
    // Ensure state directory exists.
    if let Some(parent) = home::pid_file().parent() {
        fs::create_dir_all(parent)?;
    }

    println!("Starting SelfClaw daemon...");
    println!("  Config:  {}", config_path.display());
    println!("  Memory:  {}", memory_dir.display());
    println!("  Log:     {}", log_file.display());

    let log = fs::File::create(&log_file)?;
    let log_err = log.try_clone()?;

    let child = Command::new(exe)
        .args([
            "-c", &config_path.to_string_lossy(),
            "-m", &memory_dir.to_string_lossy(),
            "run",
        ])
        .stdout(log)
        .stderr(log_err)
        .stdin(std::process::Stdio::null())
        .spawn()?;

    let pid = child.id();
    fs::write(home::pid_file(), pid.to_string())?;

    println!("  PID:     {}", pid);
    println!("\nSelfClaw daemon started.");
    println!("  View logs:  tail -f {}", log_file.display());
    println!("  Stop:       selfclaw daemon stop");

    Ok(())
}

// ── Stop ────────────────────────────────────────────────────────────

pub fn stop() -> anyhow::Result<()> {
    if !is_running() {
        println!("SelfClaw daemon is not running.");
        return Ok(());
    }

    let pid = read_pid()?;
    println!("Stopping SelfClaw daemon (PID: {})...", pid);

    // Send SIGTERM.
    #[cfg(unix)]
    {
        use std::process::Command;
        Command::new("kill")
            .arg(pid.to_string())
            .status()?;
    }

    #[cfg(not(unix))]
    {
        // On non-Unix, try taskkill.
        Command::new("taskkill")
            .args(["/PID", &pid.to_string()])
            .status()?;
    }

    // Remove PID file.
    let pid_path = home::pid_file();
    if pid_path.exists() {
        fs::remove_file(pid_path)?;
    }

    println!("SelfClaw daemon stopped.");
    Ok(())
}

// ── Restart ────────────────────────────────────────────────────────

pub fn restart() -> anyhow::Result<()> {
    if is_running() {
        stop()?;
    }
    start()
}

// ── Status ──────────────────────────────────────────────────────────

pub fn status() -> anyhow::Result<()> {
    if is_running() {
        let pid = read_pid()?;
        println!("SelfClaw daemon is running (PID: {}).", pid);
        println!("  Log: {}", home::daemon_log_file().display());
    } else {
        println!("SelfClaw daemon is not running.");
        let pid_path = home::pid_file();
        if pid_path.exists() {
            // Stale PID file.
            fs::remove_file(pid_path)?;
        }
    }

    // Check for installed service.
    if launchd_plist_path().exists() {
        println!("  Service: launchd (macOS) — installed");
    } else if systemd_unit_path().exists() {
        println!("  Service: systemd (Linux) — installed");
    } else {
        println!("  Service: not installed (use `selfclaw daemon install`)");
    }

    Ok(())
}

// ── Install (system service) ────────────────────────────────────────

pub fn install() -> anyhow::Result<()> {
    let exe = std::env::current_exe()?;
    let config_path = home::config_path();
    let memory_dir = home::memory_dir();
    let log_file = home::daemon_log_file();

    #[cfg(target_os = "macos")]
    {
        install_launchd(&exe, &config_path, &memory_dir, &log_file)?;
    }

    #[cfg(target_os = "linux")]
    {
        install_systemd(&exe, &config_path, &memory_dir, &log_file)?;
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        println!("  Automatic service installation is not supported on this platform.");
        println!("  Use `selfclaw daemon start` to run manually.");
    }

    Ok(())
}

// ── Uninstall (system service) ──────────────────────────────────────

pub fn uninstall() -> anyhow::Result<()> {
    #[cfg(target_os = "macos")]
    {
        uninstall_launchd()?;
    }

    #[cfg(target_os = "linux")]
    {
        uninstall_systemd()?;
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        println!("  No system service to uninstall.");
    }

    Ok(())
}

// ── macOS: launchd ──────────────────────────────────────────────────

fn launchd_plist_path() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join("Library/LaunchAgents/ai.selfclaw.agent.plist")
}

#[cfg(target_os = "macos")]
fn install_launchd(
    exe: &Path,
    config: &Path,
    memory: &Path,
    log: &Path,
) -> anyhow::Result<()> {
    let plist_path = launchd_plist_path();

    if plist_path.exists() {
        println!("  LaunchAgent already installed at {}", plist_path.display());
        println!("  Use `selfclaw daemon uninstall` first to reinstall.");
        return Ok(());
    }

    // Collect API key env vars to forward into the daemon environment.
    let env_vars_to_forward = [
        "ANTHROPIC_API_KEY", "OPENAI_API_KEY", "GOOGLE_API_KEY",
        "OPENROUTER_API_KEY", "GROQ_API_KEY", "XAI_API_KEY",
        "MISTRAL_API_KEY", "DEEPSEEK_API_KEY", "TOGETHER_API_KEY",
        "MOONSHOT_API_KEY", "AWS_ACCESS_KEY_ID", "AWS_SECRET_ACCESS_KEY",
        "SELFCLAW_HOME",
    ];
    let mut extra_env = String::new();
    for var in &env_vars_to_forward {
        if let Ok(val) = std::env::var(var) {
            extra_env.push_str(&format!(
                "        <key>{}</key>\n        <string>{}</string>\n",
                var, val
            ));
        }
    }

    let plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>ai.selfclaw.agent</string>
    <key>ProgramArguments</key>
    <array>
        <string>{exe}</string>
        <string>-c</string>
        <string>{config}</string>
        <string>-m</string>
        <string>{memory}</string>
        <string>run</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>{log}</string>
    <key>StandardErrorPath</key>
    <string>{log}</string>
    <key>EnvironmentVariables</key>
    <dict>
        <key>PATH</key>
        <string>/usr/local/bin:/usr/bin:/bin:/opt/homebrew/bin</string>
{extra_env}    </dict>
</dict>
</plist>"#,
        exe = exe.display(),
        config = config.display(),
        memory = memory.display(),
        log = log.display(),
        extra_env = extra_env,
    );

    if let Some(parent) = plist_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&plist_path, plist)?;

    // Load the agent.
    Command::new("launchctl")
        .args(["load", &plist_path.to_string_lossy()])
        .status()?;

    println!("  Installed LaunchAgent: {}", plist_path.display());
    println!("  SelfClaw will start automatically on login.");
    println!("  Control: launchctl start/stop ai.selfclaw.agent");

    Ok(())
}

#[cfg(target_os = "macos")]
fn uninstall_launchd() -> anyhow::Result<()> {
    let plist_path = launchd_plist_path();
    if !plist_path.exists() {
        println!("  No LaunchAgent installed.");
        return Ok(());
    }

    Command::new("launchctl")
        .args(["unload", &plist_path.to_string_lossy()])
        .status()?;

    fs::remove_file(&plist_path)?;
    println!("  Uninstalled LaunchAgent.");
    Ok(())
}

// ── Linux: systemd ──────────────────────────────────────────────────

fn systemd_unit_path() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".config/systemd/user/selfclaw.service")
}

#[cfg(target_os = "linux")]
fn install_systemd(
    exe: &Path,
    config: &Path,
    memory: &Path,
    _log: &Path,
) -> anyhow::Result<()> {
    let unit_path = systemd_unit_path();

    if unit_path.exists() {
        println!("  systemd unit already installed at {}", unit_path.display());
        println!("  Use `selfclaw daemon uninstall` first to reinstall.");
        return Ok(());
    }

    // Collect API key env vars to forward into the service environment.
    let env_vars_to_forward = [
        "ANTHROPIC_API_KEY", "OPENAI_API_KEY", "GOOGLE_API_KEY",
        "OPENROUTER_API_KEY", "GROQ_API_KEY", "XAI_API_KEY",
        "MISTRAL_API_KEY", "DEEPSEEK_API_KEY", "TOGETHER_API_KEY",
        "MOONSHOT_API_KEY", "AWS_ACCESS_KEY_ID", "AWS_SECRET_ACCESS_KEY",
        "SELFCLAW_HOME",
    ];
    let mut env_lines = String::from("Environment=PATH=/usr/local/bin:/usr/bin:/bin\n");
    for var in &env_vars_to_forward {
        if let Ok(val) = std::env::var(var) {
            env_lines.push_str(&format!("Environment={}={}\n", var, val));
        }
    }

    let unit = format!(
        r#"[Unit]
Description=SelfClaw Autonomous AI Agent
After=network.target

[Service]
Type=simple
ExecStart={exe} -c {config} -m {memory} run
Restart=on-failure
RestartSec=10
{env_lines}
[Install]
WantedBy=default.target
"#,
        exe = exe.display(),
        config = config.display(),
        memory = memory.display(),
        env_lines = env_lines,
    );

    if let Some(parent) = unit_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&unit_path, unit)?;

    // Reload and enable.
    Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status()?;
    Command::new("systemctl")
        .args(["--user", "enable", "selfclaw"])
        .status()?;
    Command::new("systemctl")
        .args(["--user", "start", "selfclaw"])
        .status()?;

    println!("  Installed systemd user service: {}", unit_path.display());
    println!("  SelfClaw will start automatically on login.");
    println!("  Control: systemctl --user start/stop/status selfclaw");

    Ok(())
}

#[cfg(target_os = "linux")]
fn uninstall_systemd() -> anyhow::Result<()> {
    let unit_path = systemd_unit_path();
    if !unit_path.exists() {
        println!("  No systemd unit installed.");
        return Ok(());
    }

    Command::new("systemctl")
        .args(["--user", "stop", "selfclaw"])
        .status()?;
    Command::new("systemctl")
        .args(["--user", "disable", "selfclaw"])
        .status()?;

    fs::remove_file(&unit_path)?;

    Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status()?;

    println!("  Uninstalled systemd unit.");
    Ok(())
}

// ── Helpers ─────────────────────────────────────────────────────────

fn read_pid() -> anyhow::Result<u32> {
    let pid_str = fs::read_to_string(home::pid_file())?;
    Ok(pid_str.trim().parse()?)
}

fn is_running() -> bool {
    let pid_path = home::pid_file();
    if !pid_path.exists() {
        return false;
    }

    let Ok(pid) = read_pid() else {
        return false;
    };

    // Check if process is still alive.
    #[cfg(unix)]
    {
        // kill -0 checks process existence without sending a signal.
        Command::new("kill")
            .args(["-0", &pid.to_string()])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    #[cfg(not(unix))]
    {
        // On Windows, check via tasklist.
        Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid)])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_is_running_returns_false_when_no_pid() {
        let tmp = TempDir::new().unwrap();
        std::env::set_var("SELFCLAW_HOME", tmp.path().join(".selfclaw"));
        assert!(!is_running());
        std::env::remove_var("SELFCLAW_HOME");
    }

    #[test]
    fn test_is_running_returns_false_for_stale_pid() {
        let tmp = TempDir::new().unwrap();
        let home = tmp.path().join(".selfclaw");
        std::env::set_var("SELFCLAW_HOME", &home);

        let state = home.join("state");
        fs::create_dir_all(&state).unwrap();
        // PID 999999 almost certainly doesn't exist.
        fs::write(state.join("selfclaw.pid"), "999999").unwrap();

        assert!(!is_running());
        std::env::remove_var("SELFCLAW_HOME");
    }

    #[test]
    fn test_launchd_plist_path_format() {
        let path = launchd_plist_path();
        assert!(path.to_string_lossy().contains("ai.selfclaw.agent.plist"));
    }

    #[test]
    fn test_systemd_unit_path_format() {
        let path = systemd_unit_path();
        assert!(path.to_string_lossy().contains("selfclaw.service"));
    }
}
