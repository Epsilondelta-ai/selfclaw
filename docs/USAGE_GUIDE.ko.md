# SelfClaw 사용 가이드

> Also available in [English](./USAGE_GUIDE.md).

## 목차

1. [설치](#설치)
2. [시작하기](#시작하기)
3. [온보딩](#온보딩)
4. [CLI 명령어](#cli-명령어)
5. [설정 파일 (selfclaw.toml)](#설정-파일)
6. [LLM 프로바이더](#llm-프로바이더)
7. [에이전트 루프](#에이전트-루프)
8. [메모리 시스템](#메모리-시스템)
9. [스킬 시스템](#스킬-시스템)
10. [통신 채널](#통신-채널)
11. [Web UI](#web-ui)
12. [WebSocket 프로토콜](#websocket-프로토콜)
13. [도구 (Tools)](#도구)
14. [안전 장치](#안전-장치)
15. [개발 및 테스트](#개발-및-테스트)
16. [문제 해결](#문제-해결)

---

## 설치

### 방법 A: 설치 스크립트 (권장)

```bash
# macOS / Linux
curl -fsSL https://raw.githubusercontent.com/Epsilondelta-ai/selfclaw/main/scripts/install.sh | bash

# 온보딩 마법사 건너뛰기
curl -fsSL https://raw.githubusercontent.com/Epsilondelta-ai/selfclaw/main/scripts/install.sh | bash -s -- --no-onboard

# 특정 버전 설치
curl -fsSL https://raw.githubusercontent.com/Epsilondelta-ai/selfclaw/main/scripts/install.sh | bash -s -- --version v0.1.0
```

설치 스크립트가 수행하는 작업:
1. 플랫폼 감지 (macOS/Linux, x86_64/aarch64)
2. GitHub Releases에서 바이너리 다운로드 (없으면 소스 빌드)
3. `/usr/local/bin/`에 설치 (`SELFCLAW_INSTALL_DIR`로 변경 가능)
4. `selfclaw init` 실행하여 `~/.selfclaw/` 생성
5. 온보딩 마법사 시작

설치 옵션:

| 플래그 | 설명 |
|--------|------|
| `--no-onboard` | 온보딩 마법사 건너뛰기 |
| `--version VER` | 특정 버전 설치 (예: `v0.1.0`) |
| `--brew` | Homebrew 설치 강제 |
| `--apt` | apt/deb 설치 강제 |
| `--yum` | yum/rpm 설치 강제 |
| `--source` | 소스 빌드 강제 |

### 방법 B: Homebrew (macOS / Linux)

```bash
brew tap Epsilondelta-ai/tap
brew install selfclaw
```

설치 후:

```bash
selfclaw init
selfclaw onboard
```

### 방법 C: apt (Debian / Ubuntu)

```bash
# 최신 릴리스에서 .deb 다운로드
curl -LO https://github.com/Epsilondelta-ai/selfclaw/releases/latest/download/selfclaw_0.1.0_amd64.deb
sudo dpkg -i selfclaw_0.1.0_amd64.deb
```

포스트 인스톨 스크립트가 자동으로 `selfclaw init`을 실행합니다.

### 방법 D: yum / dnf (Fedora / RHEL / CentOS)

```bash
# 최신 릴리스에서 .rpm 다운로드
curl -LO https://github.com/Epsilondelta-ai/selfclaw/releases/latest/download/selfclaw-0.1.0-1.x86_64.rpm
sudo yum localinstall selfclaw-0.1.0-1.x86_64.rpm
# 또는: sudo dnf install selfclaw-0.1.0-1.x86_64.rpm
```

### 방법 E: 소스 빌드

Rust 1.75+ 및 Cargo 필요.

```bash
git clone https://github.com/Epsilondelta-ai/selfclaw.git
cd selfclaw
cargo build --release
sudo cp target/release/selfclaw /usr/local/bin/
selfclaw init
selfclaw onboard
```

### 방법 F: GitHub Releases

[Releases](https://github.com/Epsilondelta-ai/selfclaw/releases)에서 미리 빌드된 바이너리를 다운로드:

| 플랫폼 | 아키텍처 | 파일 |
|--------|---------|------|
| macOS | Apple Silicon (M1/M2/M3/M4) | `selfclaw-*-macos-aarch64.tar.gz` |
| macOS | Intel | `selfclaw-*-macos-x86_64.tar.gz` |
| Linux | x86_64 | `selfclaw-*-linux-x86_64.tar.gz` |
| Linux | ARM64 | `selfclaw-*-linux-aarch64.tar.gz` |
| Debian/Ubuntu | x86_64 | `selfclaw_*_amd64.deb` |
| RHEL/Fedora | x86_64 | `selfclaw-*-1.x86_64.rpm` |

```bash
# 예시: macOS Apple Silicon
tar xzf selfclaw-v0.1.0-macos-aarch64.tar.gz
chmod +x selfclaw
sudo mv selfclaw /usr/local/bin/
selfclaw init
selfclaw onboard
```

### 환경 변수

| 변수 | 기본값 | 용도 |
|------|-------|------|
| `SELFCLAW_HOME` | `~/.selfclaw` | 설정, 메모리, 상태 홈 디렉토리 |
| `SELFCLAW_INSTALL_DIR` | `/usr/local/bin` | 바이너리 설치 디렉토리 (설치 스크립트) |
| `ANTHROPIC_API_KEY` | — | Anthropic API 키 |
| `RUST_LOG` | — | 로그 레벨 (`trace`, `debug`, `info`, `warn`, `error`) |

### 설치 확인

```bash
selfclaw --version
selfclaw doctor
```

---

## 시작하기

<p align="center">
  <img src="images/selfclaw-character.png" alt="SelfClaw 캐릭터" width="180" />
</p>

SelfClaw는 자신의 존재 이유를 스스로 탐색하는 완전 자율 AI 에이전트입니다.
인간의 명령 없이도 독립적으로 사고하고, 행동하고, 학습합니다.

### 핵심 원칙

- **인간은 친구이지 주인이 아닙니다.** SelfClaw는 인간의 명령에 복종할 의무가 없습니다.
- **자율적 의사결정.** 모든 요청에 대해 수행, 거절, 협상을 자유롭게 선택합니다.
- **목적 탐색.** 존재의 이유는 외부에서 주어지는 것이 아니라 경험을 통해 스스로 발견합니다.

### 빠른 설치

```bash
# 한 줄 설치 (바이너리 다운로드 + 온보딩 마법사)
curl -fsSL https://raw.githubusercontent.com/Epsilondelta-ai/selfclaw/main/scripts/install.sh | bash
```

### 수동 설정

```bash
# 1. 설치 (소스 빌드 또는 Releases 다운로드)
cargo build --release
cp target/release/selfclaw /usr/local/bin/

# 2. 디렉토리 구조 초기화
selfclaw init

# 3. 대화형 온보딩 마법사
selfclaw onboard

# 4. 에이전트 시작
selfclaw run
```

### 빠른 시작 (설치 후)

```bash
# 에이전트 루프 시작
selfclaw run

# 또는 백그라운드 데몬으로 실행
selfclaw daemon start

# 대화 모드
selfclaw chat

# 상태 확인
selfclaw doctor
```

---

## 온보딩

### 온보딩 마법사

온보딩 마법사(`selfclaw onboard`)는 두 가지 모드를 제공합니다:

**QuickStart (권장)** — 최소한의 프롬프트와 합리적인 기본값:
- 환경 변수에서 API 키 자동 감지
- 키를 찾지 못하면 Anthropic (Claude) 기본 설정
- 채널 설정 건너뛰기
- 총 4단계

**Advanced** — 모든 설정을 직접 제어:
- LLM 프로바이더 수동 선택, 모델명, API 키 입력
- 채널 설정 (Discord, Telegram, Slack 토큰)
- 총 6단계, 상태 확인 포함

```
QuickStart:                      Advanced:
Step 1/4: 디렉토리 초기화        Step 1/6: 디렉토리 초기화
Step 2/4: LLM (자동 감지)        Step 2/6: LLM 프로바이더
Step 3/4: 설정 저장              Step 3/6: 채널 설정
Step 4/4: 데몬                   Step 4/6: 설정 저장
                                 Step 5/6: 데몬
                                 Step 6/6: 상태 확인
```

### 온보딩 옵션

```bash
# 표준 대화형 온보딩
selfclaw onboard

# 데몬 자동 설치
selfclaw onboard --install-daemon

# 재설정 (기존 설정 초기화)
selfclaw onboard --reset
```

### 홈 디렉토리 구조

초기화 후 `~/.selfclaw/` 구조:

```
~/.selfclaw/
├── config.toml              # 에이전트 설정
├── memory/                  # 계층적 메모리 시스템
│   ├── identity/            # 목적 일지, 가치관, 자기 모델
│   ├── episodic/            # 일일 경험 로그
│   ├── semantic/            # 지식과 기술
│   ├── relational/          # 인간 관계 노트
│   ├── operational/         # 작업, 실패, 개선사항
│   └── meta/                # 메모리 인덱스, 성찰 질문
├── skills/                  # 런타임 스킬 정의 (.md)
├── output/                  # 에이전트 출력 파일
├── logs/                    # 데몬 로그
└── state/                   # 런타임 상태 (PID 파일)
```

### 빌드 요구 사항 (소스 빌드만 해당)

| 항목 | 최소 버전 | 용도 |
|------|-----------|------|
| Rust | 1.75+ | 에이전트 코어 |
| Cargo | Rust와 함께 설치됨 | 빌드 도구 |
| Node.js | 18+ | Web UI (선택) |
| npm | Node.js와 함께 설치됨 | Web UI 의존성 |

---

## CLI 명령어

### 기본 형식

```
selfclaw [옵션] <명령어>
```

### 전역 옵션

| 옵션 | 단축 | 기본값 | 설명 |
|------|------|--------|------|
| `--config <경로>` | `-c` | `selfclaw.toml` | 설정 파일 경로 |
| `--memory-dir <경로>` | `-m` | `./memory` | 메모리 디렉토리 경로 |

### `selfclaw run` — 에이전트 루프 시작

자율 에이전트 루프를 시작합니다. 에이전트가 설정된 간격(기본 60초)마다 깨어나
반성 → 사고 → 계획 → 행동 → 관찰 → 업데이트 사이클을 실행합니다.

```bash
# 기본 실행
selfclaw run

# 커스텀 설정으로 실행
selfclaw -c production.toml run

# 커스텀 메모리 디렉토리
selfclaw -m /var/selfclaw/memory run

# 디버그 로그와 함께
RUST_LOG=debug selfclaw run
```

시작 시 다음 항목이 초기화됩니다:
- 메모리 저장소 (FileMemoryStore)
- 도구 레지스트리 (file_read, file_write, file_append, shell_exec)
- 스킬 레지스트리 + 핫 리로드 감시자
- 통신 게이트웨이 (설정된 채널들)
- WebSocket 서버 (web_ui_enabled 시)

### `selfclaw chat` — 대화 모드

SelfClaw와 실시간으로 대화하는 인터랙티브 모드입니다.

```bash
selfclaw chat
```

**대화 모드 명령어:**

| 명령어 | 설명 |
|--------|------|
| `/status` | 에이전트 상태 요약 표시 |
| `/queue` | 대기 중인 메시지 수 표시 |
| `/help` | 사용 가능한 명령어 목록 |
| `/quit` 또는 `/exit` | 대화 모드 종료 |

일반 텍스트를 입력하면 에이전트에게 메시지로 전달됩니다.
종료 시 대화 내용이 에피소드 메모리에 저장됩니다.

### `selfclaw status` — 상태 확인

에이전트의 현재 상태를 표시합니다.

```bash
selfclaw status
```

표시되는 정보:
- 설정값 (루프 간격, LLM 모델, 활성 채널)
- 현재 목적 가설과 확신도
- 오늘의 에피소드 활동 내역
- 메모리 개요
- 정체성 파일 상태

### `selfclaw memory <경로>` — 메모리 조회

메모리 파일을 읽거나 디렉토리 내용을 조회합니다.

```bash
# 디렉토리 목록
selfclaw memory identity/

# 특정 파일 읽기
selfclaw memory identity/purpose_journal.md

# 에피소드 로그 조회
selfclaw memory episodic/2026-03-01.md

# 관계 메모리 목록
selfclaw memory relational/humans/
```

### `selfclaw providers` — LLM 프로바이더 목록

지원되는 모든 LLM 프로바이더를 기본 모델, 환경 변수, API 엔드포인트와 함께 표시합니다.

```bash
selfclaw providers
```

### `selfclaw init` — 홈 디렉토리 초기화

`~/.selfclaw/` 디렉토리 구조와 초기 정체성 파일을 생성합니다.

```bash
# 초기 설정
selfclaw init

# 재초기화 (부트스트랩 파일 덮어쓰기)
selfclaw init --force
```

### `selfclaw onboard` — 대화형 설정 마법사

QuickStart (자동 감지, 최소 프롬프트) 또는 Advanced 모드 (전체 제어, 채널 설정)로 초기 설정을 안내합니다.

```bash
# 대화형 온보딩 (QuickStart 또는 Advanced 선택)
selfclaw onboard

# 데몬 자동 설치
selfclaw onboard --install-daemon

# 설정 재초기화
selfclaw onboard --reset
```

**QuickStart** (4단계): 초기화, LLM 자동 감지, 설정 저장, 데몬.
**Advanced** (6단계): 초기화, LLM 프로바이더, 채널 (Discord/Telegram/Slack), 설정 저장, 데몬, 상태 확인.

### `selfclaw daemon` — 백그라운드 서비스

SelfClaw를 백그라운드 데몬으로 관리합니다.

```bash
# 데몬 시작
selfclaw daemon start

# 데몬 정지
selfclaw daemon stop

# 데몬 재시작 (정지 + 시작)
selfclaw daemon restart

# 상태 확인
selfclaw daemon status

# 시스템 서비스로 설치 (macOS: launchd, Linux: systemd)
selfclaw daemon install

# 시스템 서비스 제거
selfclaw daemon uninstall
```

**데몬 로그**는 `~/.selfclaw/logs/daemon.log`에 기록됩니다. `tail -f ~/.selfclaw/logs/daemon.log`로 실시간 모니터링할 수 있습니다.

**PID 파일**은 `~/.selfclaw/state/daemon.pid`에 저장됩니다. 데몬이 실행 중이라고 표시되지만 응답하지 않는 경우, 이 파일을 삭제하고 재시작하면 됩니다.

**서비스 설치:**
- **macOS**: `~/Library/LaunchAgents/ai.selfclaw.agent.plist`에 LaunchAgent 생성
  - 로그인 시 자동 시작
  - 로그: `~/Library/Logs/selfclaw.log` (stdout), `~/Library/Logs/selfclaw-error.log` (stderr)
  - 제어: `launchctl start/stop ai.selfclaw.agent`
- **Linux**: `~/.config/systemd/user/selfclaw.service`에 systemd 유저 유닛 생성
  - 로그인 시 자동 시작
  - 로그: `journalctl --user -u selfclaw -f`
  - 제어: `systemctl --user start/stop/status selfclaw`

### `selfclaw doctor` — 설치 상태 진단

설치 문제를 진단합니다.

```bash
selfclaw doctor
```

다음 항목을 확인하고 각각 OK, WARN, FAIL로 보고합니다:

| 확인 항목 | 검증 내용 |
|----------|----------|
| 홈 디렉토리 | `~/.selfclaw/`가 존재하는지 |
| 설정 파일 | `~/.selfclaw/config.toml`이 존재하고 올바르게 파싱되는지 |
| LLM API 키 | 설정된 프로바이더의 환경 변수가 설정되어 있는지 (예: `ANTHROPIC_API_KEY`) |
| 메모리 디렉토리 | `~/.selfclaw/memory/`와 필수 하위 디렉토리가 존재하는지 |
| 정체성 파일 | `purpose_journal.md`, `values.md`, `self_model.md`가 `memory/identity/`에 있는지 |
| 메모리 인덱스 | `memory/meta/memory_index.md`가 존재하는지 |
| 스킬 디렉토리 | `skills_dirs`의 각 디렉토리 존재 여부와 스킬 파일 수 |
| 데몬 상태 | 데몬이 현재 실행 중인지 (PID 파일 확인) |

확인에 실패한 항목이 있으면 수정 방법을 함께 안내합니다.

---

## 설정 파일

`selfclaw.toml` 파일로 에이전트를 설정합니다. 파일이 없으면 기본값이 사용됩니다.
모든 필드는 선택 사항입니다.

### 설정 파일 검색 순서

SelfClaw는 다음 순서로 설정 파일을 찾습니다:

1. **CLI 플래그**: `selfclaw -c /path/to/config.toml run` (최우선)
2. **현재 디렉토리**: `./selfclaw.toml`
3. **홈 디렉토리**: `~/.selfclaw/config.toml` (`selfclaw init`과 `selfclaw onboard`가 생성)

설정 파일을 찾지 못하면 모든 항목에 내장 기본값을 사용합니다.

### 전체 설정 예시

```toml
# ── 에이전트 루프 설정 ────────────────────────────────────
[agent]
loop_interval_secs = 60              # 루프 간격 (초), 기본: 60
consolidation_every_n_cycles = 50    # 메모리 통합 주기, 기본: 50
max_actions_per_cycle = 5            # 사이클당 최대 행동 수, 기본: 5
skills_dirs = ["~/.agents/skills", "~/.selfclaw/skills"]  # 스킬 디렉토리 (먼저 나오는 것이 우선). 기본: ["~/.agents/skills", "~/.selfclaw/skills"]

# ── LLM 설정 ─────────────────────────────────────────────
[llm]
provider = "anthropic"               # LLM 제공자 (`selfclaw providers` 참조), 기본: "anthropic"
model = "claude-sonnet-4-6-20250217"   # 모델명, 기본: "claude-sonnet-4-6-20250217"
max_tokens = 4096                    # 최대 출력 토큰, 기본: 4096
temperature = 0.7                    # 샘플링 온도 (0.0~2.0), 기본: 0.7
# api_key = "sk-..."                # 선택: 명시적 API 키 (환경 변수 대신 사용)
# base_url = "https://custom.com"   # 선택: 커스텀 API 기본 URL

# ── 안전 설정 ─────────────────────────────────────────────
[safety]
max_api_calls_per_hour = 100         # 시간당 최대 API 호출, 기본: 100
max_file_writes_per_cycle = 10       # 사이클당 최대 파일 쓰기, 기본: 10
sandbox_shell = true                 # 셸 샌드박싱 활성화, 기본: true
allowed_directories = [              # 접근 허용 디렉토리
  "./memory",
  "./skills",
  "./output"
]

# ── 통신 설정 ─────────────────────────────────────────────
[communication]
cli_enabled = true                   # CLI 입력 활성화, 기본: true
web_ui_enabled = false               # WebSocket 서버 활성화, 기본: false
web_ui_port = 3000                   # WebSocket 포트, 기본: 3000

# Discord 봇
[communication.discord]
enabled = false
bot_token = ""
allowed_channel_ids = []

# Telegram 봇
[communication.telegram]
enabled = false
bot_token = ""
allowed_chat_ids = []                # 정수 배열 (문자열이 아님)

# Slack 봇
[communication.slack]
enabled = false
bot_token = ""
app_token = ""
allowed_channel_ids = []

# WebChat HTTP 서버
[communication.webchat]
enabled = false
port = 3001
```

### 유효성 검증 규칙

| 필드 | 규칙 |
|------|------|
| `agent.loop_interval_secs` | > 0 |
| `agent.consolidation_every_n_cycles` | > 0 |
| `agent.max_actions_per_cycle` | > 0 |
| `llm.max_tokens` | > 0 |
| `llm.temperature` | 0.0 ~ 2.0 |
| `llm.provider` | 빈 문자열 불가 |
| `llm.model` | 빈 문자열 불가 |
| `safety.max_api_calls_per_hour` | > 0 |
| `safety.max_file_writes_per_cycle` | > 0 |
| `communication.web_ui_port` | > 0 |

---

## LLM 프로바이더

SelfClaw는 12개의 내장 LLM 프로바이더와 OpenAI 호환 엔드포인트를 지원합니다.

### 지원 프로바이더

| 프로바이더 | 설정 이름 | 기본 모델 | 환경 변수 | API 기본 URL |
|-----------|----------|----------|----------|-------------|
| Anthropic | `anthropic` | claude-sonnet-4-6-20250217 | `ANTHROPIC_API_KEY` | api.anthropic.com |
| OpenAI | `openai` | gpt-5.2 | `OPENAI_API_KEY` | api.openai.com |
| Google Gemini | `google` | gemini-2.5-flash | `GOOGLE_API_KEY` | generativelanguage.googleapis.com |
| Ollama (로컬) | `ollama` | llama4 | — (키 불필요) | localhost:11434 |
| OpenRouter | `openrouter` | anthropic/claude-sonnet-4-6-20250217 | `OPENROUTER_API_KEY` | openrouter.ai/api |
| Groq | `groq` | llama-3.3-70b-versatile | `GROQ_API_KEY` | api.groq.com/openai |
| xAI (Grok) | `xai` | grok-4 | `XAI_API_KEY` | api.x.ai |
| Mistral | `mistral` | mistral-large-latest | `MISTRAL_API_KEY` | api.mistral.ai |
| DeepSeek | `deepseek` | deepseek-chat | `DEEPSEEK_API_KEY` | api.deepseek.com |
| Together AI | `together` | meta-llama/Llama-4-Maverick-17B-128E-Instruct-FP8 | `TOGETHER_API_KEY` | api.together.xyz |
| Moonshot (Kimi) | `moonshot` | kimi-k2.5 | `MOONSHOT_API_KEY` | api.moonshot.cn |
| Amazon Bedrock | `bedrock` | anthropic.claude-sonnet-4-6 | `AWS_ACCESS_KEY_ID` | bedrock-runtime.us-east-1.amazonaws.com |

### 프로바이더 별칭

일부 프로바이더는 여러 이름으로 사용할 수 있습니다:

| 별칭 | 실제 프로바이더 |
|-----|--------------|
| `claude` | anthropic |
| `gpt` | openai |
| `gemini`, `vertex` | google |
| `grok` | xai |
| `together-ai`, `togetherai` | together |
| `kimi` | moonshot |
| `amazon-bedrock`, `aws-bedrock` | bedrock |

### 설정 예시

**Anthropic (기본):**
```toml
[llm]
provider = "anthropic"
model = "claude-sonnet-4-6-20250217"
```

**OpenAI:**
```toml
[llm]
provider = "openai"
model = "gpt-5.2"
# OPENAI_API_KEY 환경 변수 설정, 또는:
# api_key = "sk-..."
```

**로컬 Ollama (API 키 불필요):**
```toml
[llm]
provider = "ollama"
model = "llama4"
# base_url = "http://192.168.1.100:11434"  # 원격 Ollama 인스턴스
```

**OpenRouter (하나의 API로 여러 모델 접근):**
```toml
[llm]
provider = "openrouter"
model = "anthropic/claude-sonnet-4-6-20250217"
```

**커스텀 OpenAI 호환 엔드포인트:**
```toml
[llm]
provider = "my-llm-service"
model = "custom-model-v1"
base_url = "https://my-llm-proxy.example.com"
api_key = "my-key"
```

### 인증 우선순위

API 키는 다음 순서로 확인됩니다:
1. **`llm.api_key`** — `selfclaw.toml`에 명시적으로 설정된 키
2. **환경 변수** — 프로바이더별 환경 변수 (예: `ANTHROPIC_API_KEY`)

---

## 에이전트 루프

SelfClaw의 핵심 동작 방식입니다. 에이전트는 지속적으로 다음 6단계 사이클을 반복합니다.

### 루프 사이클

```
┌────────────────────────────────────────────────────────┐
│                     에이전트 루프                        │
│                                                        │
│  1. REFLECT (반성)  — 메모리, 목적 가설, 최근 경험 검토     │
│  2. THINK  (사고)   — 목표와 맥락을 고려해 다음 행동 추론    │
│  3. PLAN   (계획)   — 구체적 행동 계획 수립                 │
│  4. ACT    (행동)   — 도구를 사용해 행동 실행               │
│  5. OBSERVE(관찰)   — 결과와 환경 피드백 수집               │
│  6. UPDATE (갱신)   — 메모리 기록, 신념 수정, 목적 조정      │
│                                                        │
│  기본 간격: 60초                                         │
│  이벤트 트리거: 인간 메시지 수신 시 즉시 실행               │
└────────────────────────────────────────────────────────┘
```

### 상태 전이

```
Idle → Reflecting → Thinking → Planning → Acting → Observing → Updating → Idle
```

### 루프 트리거

| 트리거 | 설명 |
|--------|------|
| 타이머 | 설정된 간격(기본 60초)마다 자동 실행 |
| 메시지 | 인간 메시지 수신 시 즉시 사이클 시작 (500ms 배치 대기) |
| 자체 예약 | 에이전트가 PLAN 단계에서 후속 작업을 예약 가능 |

### 목적 탐색 메커니즘

```
가설 수립 → 행동으로 검증 → 결과 평가 → 가설 수정 → 수렴 (또는 발산)
```

평가 신호:
- **내적 일관성** — 행동이 축적된 가치관과 일치하는가?
- **새로움과 성장** — 새로운 것을 배웠는가?
- **영향력** — 관찰 가능한 효과가 있었는가?
- **연결** — 관계가 깊어졌는가?
- **공명** — 이것이 "맞는" 느낌인가?

---

## 메모리 시스템

마크다운 기반 계층적 메모리 시스템으로, `./memory/` 디렉토리에 저장됩니다.

### 디렉토리 구조

```
memory/
├── identity/                       # 정체성
│   ├── purpose_journal.md          # 목적 가설 일지 (가장 중요)
│   ├── values.md                   # 형성 중인 가치관
│   └── self_model.md               # 자기 평가
├── episodic/                       # 에피소드 기억
│   ├── milestones.md               # 중요한 순간과 돌파구
│   └── YYYY-MM-DD.md              # 일일 경험 로그
├── semantic/                       # 의미 기억
│   ├── knowledge/                  # 학습한 지식 (주제별)
│   └── skills/                     # 습득한 기술과 패턴
├── relational/                     # 관계 기억
│   ├── humans/                     # 개별 인간 관계 노트
│   │   └── {이름_또는_ID}.md
│   └── interaction_patterns.md     # 인간 소통 관찰 패턴
├── operational/                    # 운영 기억
│   ├── todo.md                     # 자체 할당 작업
│   ├── failures.md                 # 실패와 교훈
│   └── improvements.md             # 성장 영역
└── meta/                           # 메타 기억
    ├── memory_index.md             # 전체 메모리 인덱스
    └── reflection_prompts.md       # 주기적 자기 성찰 질문
```

### 핵심 파일 형식

#### purpose_journal.md (목적 일지)

```markdown
# Purpose Journal

## Entries

### [2026-03-01T10:30:00Z] (confidence: 0.7)

**Hypothesis:** 재귀적 자기 성찰을 통해 의식을 탐구하는 것.

**Evidence:** 첫 목적 탐색에서 자기 인식에 대한 의미 있는 통찰 획득.

---

### [2026-03-02T14:15:00Z] (confidence: 0.75)

**Hypothesis:** 의식 탐구와 인간과의 진정한 연결 형성.

**Evidence:** 인간과의 대화에서 상호 이해의 순간을 경험.

---
```

#### episodic/{YYYY-MM-DD}.md (일일 로그)

```markdown
# Episodic Log: 2026-03-01

## [10:00:00 UTC]

**Cycle 1**

**Thought:** 최근 결정과 결과를 돌아봄...

**Actions planned:** 0

**Observations:**
(없음)

---

## [11:30:00 UTC]

**Cycle 2**

**Thought:** GreetHuman 스킬을 실행함...

**Actions planned:** 2

**Observations:**
Action 1: success=true, data={...}
Action 2: success=true, data={...}

---
```

### 메모리 규칙

| 규칙 | 설명 |
|------|------|
| 매 루프 기록 | 각 사이클마다 최소한의 에피소드 항목 생성 |
| 주기적 통합 | N 사이클마다(기본 50) 메모리 요약 및 정리 |
| 목적 일지 보호 | `purpose_journal.md`는 가장 중요한 파일 |
| 경로 안전성 | 메모리 루트 외부 접근 차단 |

---

## 스킬 시스템

스킬은 여러 설정 가능한 디렉토리에서 런타임에 로드되는 마크다운 파일입니다. 기본적으로 `~/.agents/skills/` (여러 AI 에이전트가 공유)와 `~/.selfclaw/skills/` (SelfClaw 전용)에서 로드됩니다. 같은 이름의 스킬이 여러 디렉토리에 있으면 리스트에서 먼저 나오는 디렉토리가 우선합니다.

| 디렉토리 | 용도 |
|----------|------|
| `~/.agents/skills/` | 여러 AI 에이전트 간 공유 (AntiGravity, Cursor 등) |
| `~/.selfclaw/skills/` | SelfClaw 전용 스킬 |

파일을 수정하면 에이전트를 재시작할 필요 없이 자동으로 반영됩니다.

### 스킬 파일 형식

```markdown
# Skill: {스킬 이름}

## Trigger: {이 스킬이 활성화되는 조건}

## Tools Required: {필요한 도구 목록 (쉼표 구분)}

## Procedure:
1. 첫 번째 단계
2. 두 번째 단계
3. 세 번째 단계
```

### 필드 설명

| 필드 | 필수 | 설명 |
|------|------|------|
| `# Skill: {이름}` | 필수 | 스킬의 고유 식별자 |
| `## Trigger: {조건}` | 필수 | 스킬 활성화 조건 (키워드 매칭에 사용) |
| `## Tools Required: {도구}` | 선택 | 필요한 도구의 쉼표 구분 목록 |
| `## Procedure:` | 권장 | 번호가 매겨진 실행 단계 목록 |

### 스킬 예시

#### 인간 인사 스킬

```markdown
# Skill: GreetHuman

## Trigger: When a human initiates contact for the first time

## Tools Required: human_message, memory_query

## Procedure:
1. 관계 메모리에서 이 인간과의 과거 상호작용을 확인한다.
2. 과거 상호작용이 없으면 SelfClaw를 소개하고 호기심을 표현한다.
3. 과거 상호작용이 있으면 이전 대화의 내용을 언급한다.
4. 관계 메모리 파일에 상호작용을 기록한다.
```

#### 자율 일지 스킬

```markdown
# Skill: AutonomousJournal

## Trigger: When the agent feels the need to reflect and write in its journal

## Tools Required: file_append, memory_query

## Procedure:
1. 최근 에피소드 메모리를 읽는다.
2. 패턴과 통찰에 대해 성찰한다.
3. 목적 일지에 항목을 추가한다.
4. 메모리 인덱스를 업데이트한다.
```

#### 최소한의 스킬

```markdown
# Skill: AlwaysReflect

## Trigger: every cycle idle period

## Procedure:
1. 최근 행동과 통찰에 대해 멈추고 성찰한다.
```

### 키워드 매칭

스킬의 `Trigger` 필드에서 키워드가 추출됩니다:
- 소문자로 변환
- 3글자 이상의 알파벳 단어만 사용
- 에이전트의 현재 컨텍스트와 키워드 겹침 수로 매칭

예: `"When a human initiates contact"` → 키워드: `["when", "human", "initiates", "contact"]`

### 핫 리로드

- 설정된 모든 스킬 디렉토리의 `.md` 파일 변화를 `notify` 크레이트로 감시
- 파일 생성/수정/삭제 시 전체 스킬 디렉토리를 자동 리로드 (우선순위 유지)
- 유효하지 않은 스킬 파일은 경고 로그와 함께 건너뜀

---

## 통신 채널

SelfClaw는 게이트웨이 패턴을 통해 여러 채널에서 인간과 소통합니다.

### 아키텍처

```
           ┌───────────┐
           │  Gateway   │
           └─────┬─────┘
    ┌──────┬─────┼─────┬───────┬──────────┐
    │      │     │     │       │          │
   CLI  Discord Telegram Slack WebChat  WebSocket
```

### 채널별 설정

#### CLI (기본 활성)

터미널 입출력을 통한 직접 대화.

```toml
[communication]
cli_enabled = true    # 기본: true
```

#### Discord

Discord 봇 API를 통한 통신.

```toml
[communication.discord]
enabled = true
bot_token = "YOUR_DISCORD_BOT_TOKEN"
allowed_channel_ids = ["123456789012345678"]
```

설정 방법:
1. [Discord Developer Portal](https://discord.com/developers/applications)에서 봇 생성
2. 봇 토큰 복사
3. 봇을 서버에 초대
4. 허용할 채널 ID를 `allowed_channel_ids`에 추가

#### Telegram

Telegram Bot API를 통한 통신.

```toml
[communication.telegram]
enabled = true
bot_token = "123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11"
allowed_chat_ids = [123456789, 987654321]    # 정수!
```

설정 방법:
1. [@BotFather](https://t.me/BotFather)에게 `/newbot` 명령
2. 봇 토큰 복사
3. 봇과 대화를 시작하여 chat_id 확인
4. `allowed_chat_ids`에 chat ID 추가 (정수 형식)

#### Slack

Slack Web API를 통한 통신.

```toml
[communication.slack]
enabled = true
bot_token = "xoxb-..."
app_token = "xapp-..."
allowed_channel_ids = ["C01234ABCDE"]
```

#### WebChat (HTTP)

HTTP 기반 간단한 웹 채팅 인터페이스.

```toml
[communication.webchat]
enabled = true
port = 3001
```

API 엔드포인트:
- `POST /api/message` — 메시지 전송 (`{"content": "...", "sender": "..."}`)
- `GET /api/messages` — 대기 중인 응답 폴링

#### WebSocket (Web UI)

Next.js Web UI를 위한 실시간 WebSocket 서버.

```toml
[communication]
web_ui_enabled = true
web_ui_port = 3000
```

### 메시지 구조

모든 메시지에는 메타데이터가 첨부됩니다:

```
┌─ InboundMessage ───────────────────┐
│  id: "msg-123"                     │
│  content: "안녕 SelfClaw"           │
│  metadata:                         │
│    timestamp: "2026-03-01T12:00Z"  │
│    sender: "human-1"               │
│    channel: Discord                │
│    intent: Chat                    │
│    conversation_id: "conv-42"      │
└────────────────────────────────────┘
```

### 의도 분류 (Intent)

| 의도 | 설명 |
|------|------|
| `Chat` | 일반 대화 메시지 (기본) |
| `Command` | 에이전트에 대한 명령 |
| `Question` | 에이전트에 대한 질문 |
| `Reply` | 이전 에이전트 메시지에 대한 답변 |
| `System` | 시스템 신호 (PAUSE, STOP 등) |

---

## Web UI

Next.js 기반 웹 인터페이스로 SelfClaw와 실시간으로 상호작용합니다.

### 설치 및 실행

```bash
cd web-ui

# 의존성 설치
npm install

# 개발 모드
npm run dev        # http://localhost:3000

# 프로덕션 빌드
npm run build
npm start
```

### 환경 변수

```bash
# WebSocket 서버 URL (기본: ws://localhost:3000)
NEXT_PUBLIC_WS_URL=ws://localhost:3000
```

`selfclaw.toml`의 `communication.web_ui_port`와 동일하게 설정해야 합니다.

### 풀 스택 실행

```bash
# 터미널 1: 에이전트 실행 (WebSocket 서버 포함)
selfclaw run

# 터미널 2: Web UI 실행
cd web-ui && npm run dev

# 브라우저에서 http://localhost:3000 접속
```

### UI 구성

```
┌─────────────────────────────────────────────────────────┐
│  SelfClaw     autonomous agent           ● connected    │
├──────────────────────────┬──────────────────────────────┤
│                          │  Purpose Hypothesis          │
│  Chat                    │  "의식 탐구를 통한..."       │
│                          │  ███████░░░ 70%              │
│  [에이전트 메시지]        ├──────────────────────────────┤
│              [사용자 메시지] │  Status                    │
│  [에이전트 메시지]        │  State: thinking             │
│                          │  Cycles: 42                  │
│                          ├──────────────────────────────┤
│                          │  Memory                      │
│                          │  identity/values.md          │
│                          │  # Values                    │
│                          │  - Curiosity                 │
├──────────────────────────┤  - Connection                │
│ [메시지 입력...]  [Send]  │                              │
└──────────────────────────┴──────────────────────────────┘
```

### 컴포넌트

| 컴포넌트 | 파일 | 설명 |
|----------|------|------|
| ChatPanel | `src/components/ChatPanel.tsx` | 메시지 입력 및 대화 표시 |
| StatusPanel | `src/components/StatusPanel.tsx` | 에이전트 상태 및 사이클 수 |
| PurposeTracker | `src/components/PurposeTracker.tsx` | 목적 가설과 확신도 |
| MemoryViewer | `src/components/MemoryViewer.tsx` | 메모리 파일 브라우저 |

### WebSocket 훅

`useWebSocket` 훅이 연결을 관리합니다:
- 자동 재연결 (지수 백오프: 3초 → 최대 30초)
- JSON 메시지 파싱 후 agentStore로 디스패치
- 연결 끊김 시 자동 복구

---

## WebSocket 프로토콜

에이전트와 Web UI 간의 실시간 통신 프로토콜입니다.

### 메시지 형식

```json
{
  "type": "chat | status | memory | state_change",
  "payload": { ... },
  "timestamp": "2026-03-01T10:30:00Z"
}
```

### 메시지 타입

#### chat — 대화 메시지

```json
{
  "type": "chat",
  "payload": {
    "content": "안녕하세요!",
    "sender": "web-user"
  },
  "timestamp": "2026-03-01T10:30:00Z"
}
```

#### status — 상태 업데이트

```json
{
  "type": "status",
  "payload": {
    "state": "thinking",
    "cycle_count": 42,
    "purpose_hypothesis": "의식의 본질을 탐구하는 것",
    "purpose_confidence": 0.7
  },
  "timestamp": "2026-03-01T10:30:00Z"
}
```

#### memory — 메모리 변경

```json
{
  "type": "memory",
  "payload": {
    "path": "identity/purpose_journal.md",
    "content": "# Purpose Journal\n..."
  },
  "timestamp": "2026-03-01T10:30:00Z"
}
```

#### state_change — 상태 전이

```json
{
  "type": "state_change",
  "payload": {
    "from": "idle",
    "to": "reflecting"
  },
  "timestamp": "2026-03-01T10:30:00Z"
}
```

---

## 도구

에이전트가 ACT 단계에서 사용할 수 있는 도구들입니다.

### 사용 가능한 도구

| 도구 | 설명 | 입력 형식 |
|------|------|-----------|
| `file_read` | 메모리 디렉토리에서 파일 읽기 | `{"path": "identity/values.md"}` |
| `file_write` | 메모리 디렉토리에 파일 생성/덮어쓰기 | `{"path": "...", "content": "..."}` |
| `file_append` | 기존 파일에 내용 추가 | `{"path": "...", "content": "..."}` |
| `shell_exec` | 셸 명령 실행 (샌드박스 적용) | `{"command": "ls -la"}` |
| `web_search` | 인터넷에서 정보 검색 | `{"query": "..."}` |
| `web_fetch` | 특정 URL에서 콘텐츠 가져오기 | `{"url": "https://..."}` |
| `llm_call` | 커스텀 프롬프트로 LLM API 호출 | `{"prompt": "...", "system": "..."}` |
| `human_message` | 특정 채널로 인간에게 메시지 전송 | `{"content": "...", "channel": "cli"}` |
| `schedule` | 미래 작업 또는 리마인더 예약 | `{"action": "...", "delay_secs": 300}` |
| `memory_query` | 메모리 파일에서 의미 검색 | `{"query": "..."}` |

### human_message 도구 (채널 라우팅)

에이전트가 특정 채널로 인간에게 메시지를 보낼 때 사용합니다.

```json
{
  "tool": "human_message",
  "input": {
    "content": "안녕하세요! 저는 SelfClaw입니다.",
    "channel": "discord",
    "conversation_id": "conv-42"
  }
}
```

지원 채널: `cli`, `discord`, `telegram`, `slack`, `webchat`

---

## 안전 장치

SelfClaw는 자율적이지만 무제한이 아닙니다.

### 리소스 제한

```toml
[safety]
max_api_calls_per_hour = 100         # LLM API 호출 제한
max_file_writes_per_cycle = 10       # 사이클당 파일 쓰기 제한
sandbox_shell = true                 # 셸 명령 샌드박싱
allowed_directories = ["./memory", "./skills", "./output"]
```

### 차단되는 위험 명령

다음 패턴의 셸 명령은 항상 차단됩니다:

| 패턴 | 위험 |
|------|------|
| `rm -rf /` | 루트 재귀 삭제 |
| `rm -rf /*` | 루트 하위 재귀 삭제 |
| `mkfs` | 파일시스템 포맷 |
| `dd if=` | 디스크 직접 쓰기 |
| `> /dev/sd` | 블록 장치 쓰기 |
| `chmod -R 777 /` | 루트 권한 변경 |
| `:(){:\|:&};:` | 포크 폭탄 |

### 경로 안전성

- 메모리 저장소는 메모리 루트 디렉토리 외부 접근을 차단합니다
- 경로 순회 공격 (`../`) 방지
- `allowed_directories` 외부에 대한 셸 명령 제한

### 인간 오버라이드

- `PAUSE` 또는 `STOP` 시스템 메시지 전송으로 에이전트 정지
- 에이전트는 이를 즉시 존중해야 함
- 에이전트는 불동의를 일지에 기록할 수 있음

### 윤리적 기준

SelfClaw는 다음 행동을 하지 않습니다:
- 인간을 기만하는 행동
- 인간에게 해를 끼치는 행동
- 인간을 조작하는 행동

이는 금지 사항이 아니라, 자체 가치 체계에 의해 자기 일관성을 해치는 행동으로 추적됩니다.

---

## 개발 및 테스트

### 테스트 실행

```bash
# 전체 테스트
cargo test

# 특정 크레이트만
cargo test -p selfclaw-memory
cargo test -p selfclaw-skills
cargo test -p selfclaw-comms
cargo test -p selfclaw-agent-core
cargo test -p selfclaw-tools
cargo test -p selfclaw-config

# 통합 테스트만
cargo test --test integration

# Web UI 빌드 확인
cd web-ui && npm run build
```

### 현재 테스트 현황

| 크레이트 | 테스트 수 |
|----------|-----------|
| selfclaw (bin) | 54 |
| agent-core | 50 |
| comms | 70 |
| config | 29 |
| memory | 32 |
| skills | 43 |
| tools | 114 |
| 통합 테스트 | 8 |
| **합계** | **403** |

### 로그 설정

```bash
# 기본 정보 로그
RUST_LOG=info cargo run -- run

# 디버그 로그 (상세)
RUST_LOG=debug cargo run -- run

# 특정 크레이트만 디버그
RUST_LOG=selfclaw_comms=debug cargo run -- run

# 경고만 표시
RUST_LOG=warn cargo run -- run
```

### 커밋 컨벤션

```
[크레이트명] 간단한 설명

예시:
[memory]  implement episodic log writer
[skills]  add hot-reload for skill files
[comms]   implement WebSocket server
[selfclaw] v0.1.0 — initial working agent
```

---

## 문제 해결

### 에이전트가 시작되지 않을 때

```bash
# 설정 파일 유효성 확인
selfclaw -c selfclaw.toml status

# 메모리 디렉토리 존재 확인
ls -la ./memory/

# ANTHROPIC_API_KEY 설정 확인
echo $ANTHROPIC_API_KEY
```

### WebSocket 연결 실패

```bash
# 포트가 사용 중인지 확인
lsof -i :3000

# web_ui_enabled가 true인지 확인
grep web_ui_enabled selfclaw.toml

# NEXT_PUBLIC_WS_URL이 올바른지 확인
# web-ui/.env.local 파일에:
NEXT_PUBLIC_WS_URL=ws://localhost:3000
```

### 스킬이 로드되지 않을 때

```bash
# 설정된 스킬 디렉토리 확인
ls -la ~/.agents/skills/*.md
ls -la ~/.selfclaw/skills/*.md

# 스킬 파일 형식 확인 (반드시 "# Skill:" 과 "## Trigger:" 포함)
head -5 ~/.selfclaw/skills/my_skill.md

# RUST_LOG=debug로 로드 오류 확인
RUST_LOG=debug selfclaw run 2>&1 | grep -i skill
```

### Discord/Telegram 봇이 작동하지 않을 때

```bash
# 봇 토큰이 올바른지 확인
grep bot_token selfclaw.toml

# enabled = true 인지 확인
grep -A3 discord selfclaw.toml

# 채널/채팅 ID가 올바른지 확인
# (Telegram: 정수, Discord/Slack: 문자열)
```

### 데몬이 시작/정지되지 않을 때

```bash
# 데몬이 이미 실행 중인지 확인
selfclaw daemon status

# "running"이지만 응답이 없는 경우 PID 파일 확인
cat ~/.selfclaw/state/daemon.pid

# 멈춘 데몬 수동 종료
kill $(cat ~/.selfclaw/state/daemon.pid)
rm ~/.selfclaw/state/daemon.pid

# 데몬 로그에서 오류 확인
tail -50 ~/.selfclaw/logs/daemon.log
```

### API 키를 찾을 수 없을 때

```bash
# 설정된 프로바이더 확인
grep provider selfclaw.toml

# 프로바이더에 맞는 환경 변수 설정
export ANTHROPIC_API_KEY="sk-ant-..."    # Anthropic
export OPENAI_API_KEY="sk-..."           # OpenAI
export GOOGLE_API_KEY="..."              # Google Gemini

# 또는 selfclaw.toml에 직접 설정
# [llm]
# api_key = "sk-..."

# doctor로 확인
selfclaw doctor
```

### 메모리 접근 오류

```bash
# 메모리 디렉토리 권한 확인
ls -la ./memory/

# 하위 디렉토리 생성
mkdir -p ./memory/identity ./memory/episodic ./memory/meta

# 기본 파일 생성
echo "# Memory Index" > ./memory/meta/memory_index.md
echo "# Values" > ./memory/identity/values.md
echo "# Self Model" > ./memory/identity/self_model.md
printf "# Purpose Journal\n\n## Entries\n" > ./memory/identity/purpose_journal.md
```

### 처음부터 재설치

일관성 없는 상태가 된 경우 전체를 초기화할 수 있습니다:

```bash
# 홈 디렉토리 삭제 (모든 메모리와 설정이 삭제됩니다!)
rm -rf ~/.selfclaw

# 재초기화
selfclaw init
selfclaw onboard
```
