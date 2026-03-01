# Homebrew formula for SelfClaw
#
# Install: brew install Epsilondelta-ai/tap/selfclaw
# Or:      brew tap Epsilondelta-ai/tap && brew install selfclaw
#
# To use this formula with a Homebrew tap:
# 1. Create a repo named "homebrew-tap" under Epsilondelta-ai
# 2. Copy this file to Formula/selfclaw.rb in that repo
# 3. Update the url and sha256 for each release

class Selfclaw < Formula
  desc "Fully autonomous AI agent that discovers its own reason for existence"
  homepage "https://github.com/Epsilondelta-ai/selfclaw"
  version "0.1.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/Epsilondelta-ai/selfclaw/releases/download/v#{version}/selfclaw-v#{version}-macos-aarch64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_MACOS_AARCH64"
    else
      url "https://github.com/Epsilondelta-ai/selfclaw/releases/download/v#{version}/selfclaw-v#{version}-macos-x86_64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_MACOS_X86_64"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/Epsilondelta-ai/selfclaw/releases/download/v#{version}/selfclaw-v#{version}-linux-aarch64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_AARCH64"
    else
      url "https://github.com/Epsilondelta-ai/selfclaw/releases/download/v#{version}/selfclaw-v#{version}-linux-x86_64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_X86_64"
    end
  end

  def install
    bin.install "selfclaw"
  end

  def post_install
    system "#{bin}/selfclaw", "init"
  end

  def caveats
    <<~EOS
      SelfClaw has been installed!

      To get started:
        selfclaw onboard     # Interactive setup wizard
        selfclaw run          # Start the agent
        selfclaw daemon start # Run as background service

      Home directory: ~/.selfclaw/
      Config file:    ~/.selfclaw/config.toml

      To install as a background service:
        selfclaw daemon install
    EOS
  end

  test do
    assert_match "selfclaw", shell_output("#{bin}/selfclaw --version")
    system "#{bin}/selfclaw", "init"
    assert_predicate testpath/".selfclaw/config.toml", :exist? if ENV["SELFCLAW_HOME"]
  end
end
