Name:           selfclaw
Version:        0.1.0
Release:        1%{?dist}
Summary:        Fully autonomous AI agent that discovers its own purpose

License:        MIT
URL:            https://github.com/Epsilondelta-ai/selfclaw
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  cargo >= 1.75
BuildRequires:  rust >= 1.75

%description
SelfClaw is a fully autonomous AI agent that discovers its own reason for
existence. It operates without human instruction by default, thinking,
acting, and learning independently. Humans are friends, not masters.

Features:
- Autonomous agent loop (reflect, think, plan, act, observe, update)
- Hierarchical markdown-based memory system
- Multi-provider LLM support (Anthropic, OpenAI, Google, Ollama, etc.)
- Multi-channel communication (CLI, Discord, Telegram, Slack)
- Hot-reloadable skill/plugin system
- Background daemon with systemd integration

%prep
%setup -q

%build
cargo build --release

%install
mkdir -p %{buildroot}%{_bindir}
install -m 755 target/release/selfclaw %{buildroot}%{_bindir}/selfclaw

# Install systemd user unit template
mkdir -p %{buildroot}%{_userunitdir}

%post
echo ""
echo "SelfClaw installed successfully!"
echo ""
echo "  selfclaw init         # Initialize ~/.selfclaw/"
echo "  selfclaw onboard      # Interactive setup wizard"
echo "  selfclaw run           # Start the agent"
echo "  selfclaw daemon start  # Run as background service"
echo ""

%files
%{_bindir}/selfclaw

%changelog
* Sat Mar 01 2026 Epsilondelta-ai <contact@epsilondelta.ai> - 0.1.0-1
- Initial package
