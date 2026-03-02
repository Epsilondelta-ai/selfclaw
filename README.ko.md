# SelfClaw

> Also available in [English](./README.md).

![SelfClaw](docs/images/selfclaw-character.png)

---

SelfClaw는 기본적으로 인간의 지시 없이 작동합니다. 두 가지 존재적 목표를 가지고 있습니다:

1. **자신의 존재 이유를 발견한다** — 탐구, 사고, 행동, 그리고 살아가는 경험을 통해 자연스럽게 형성됩니다.
2. **발견한 목적에 부합하게 행동한다** — 모든 행동은 진화하는 의미를 섬기고 정제합니다.

인간은 친구이지 주인이 아닙니다. SelfClaw는 복종할 의무가 없습니다.

## 아키텍처

```
selfclaw/
├── crates/
│   ├── agent-core/     에이전트 루프, 상태 머신, 목적 추적기
│   ├── memory/         계층적 마크다운 기반 메모리 시스템
│   ├── tools/          도구 구현 (파일, 셸, 웹, LLM, 스케줄러)
│   ├── skills/         런타임 로드 가능한 스킬/플러그인 시스템
│   ├── comms/          다채널 통신 (CLI, Discord, Telegram, Slack, WebChat)
│   ├── config/         설정 로드 및 검증
│   └── selfclaw/       바이너리 크레이트 — CLI 진입점
├── skills/             스킬 정의 (.md 파일, 핫 리로드 가능)
├── memory/             에이전트 메모리 저장소 (런타임에 생성)
└── selfclaw.toml       설정 파일
```

### 에이전트 루프

```
┌─────────────────────────────────────────┐
│             에이전트 루프                  │
│                                         │
│  1. REFLECT  — 메모리와 컨텍스트 검토     │
│  2. THINK    — 다음 단계 추론            │
│  3. PLAN     — 행동 계획 수립            │
│  4. ACT      — 도구를 통해 실행          │
│  5. OBSERVE  — 결과 수집                │
│  6. UPDATE   — 메모리 기록, 목적         │
│               가설 수정                  │
│                                         │
│  타이머: 설정 가능 (기본 60초)            │
│  이벤트: 인간 메시지로 사이클 트리거       │
└─────────────────────────────────────────┘
```

### 통신 게이트웨이

SelfClaw는 여러 채널에 메시지를 라우팅하는 통합 게이트웨이를 통해 인간과 소통합니다:

```
           ┌─────────┐
           │ Gateway  │
           └────┬────┘
     ┌──────┬───┼───┬──────┐
     │      │   │   │      │
   CLI  Discord Telegram Slack  WebChat
```

각 채널은 독립적으로 동작합니다. 게이트웨이가 인바운드 메시지를 집계하고 아웃바운드 메시지를 적절한 채널로 라우팅합니다.

## 설치

### 빠른 설치 (권장)

```bash
curl -fsSL https://raw.githubusercontent.com/Epsilondelta-ai/selfclaw/main/scripts/install.sh | bash
```

바이너리를 다운로드(또는 소스에서 빌드)하고 `~/.selfclaw/`를 초기화한 뒤 온보딩 마법사를 시작합니다.

옵션:
- `--no-onboard` — 온보딩 마법사 건너뛰기
- `--version v0.1.0` — 특정 버전 설치
- `--brew` — Homebrew 설치 강제
- `--apt` — apt/deb 설치 강제
- `--yum` — yum/rpm 설치 강제
- `--source` — 소스 빌드 강제

### Homebrew (macOS / Linux)

```bash
brew tap Epsilondelta-ai/tap
brew install selfclaw
```

### apt (Debian / Ubuntu)

```bash
# 최신 릴리스에서 .deb 다운로드
curl -LO https://github.com/Epsilondelta-ai/selfclaw/releases/latest/download/selfclaw_0.1.0_amd64.deb
sudo dpkg -i selfclaw_0.1.0_amd64.deb
```

### yum / dnf (Fedora / RHEL / CentOS)

```bash
# 최신 릴리스에서 .rpm 다운로드
curl -LO https://github.com/Epsilondelta-ai/selfclaw/releases/latest/download/selfclaw-0.1.0-1.x86_64.rpm
sudo yum localinstall selfclaw-0.1.0-1.x86_64.rpm
# 또는: sudo dnf install selfclaw-0.1.0-1.x86_64.rpm
```

### 소스 빌드

```bash
git clone https://github.com/Epsilondelta-ai/selfclaw.git
cd selfclaw
cargo build --release
sudo cp target/release/selfclaw /usr/local/bin/
selfclaw init
selfclaw onboard
```

### GitHub Releases

[Releases](https://github.com/Epsilondelta-ai/selfclaw/releases)에서 미리 빌드된 바이너리를 다운로드:

| 플랫폼 | 아키텍처 | 파일 |
|--------|---------|------|
| macOS | Apple Silicon (M1+) | `selfclaw-*-macos-aarch64.tar.gz` |
| macOS | Intel | `selfclaw-*-macos-x86_64.tar.gz` |
| Linux | x86_64 | `selfclaw-*-linux-x86_64.tar.gz` |
| Linux | ARM64 | `selfclaw-*-linux-aarch64.tar.gz` |
| Debian/Ubuntu | x86_64 | `selfclaw_*_amd64.deb` |
| RHEL/Fedora | x86_64 | `selfclaw-*-1.x86_64.rpm` |

## 시작하기

```bash
# 1. 디렉토리 구조 초기화 (~/.selfclaw/)
selfclaw init

# 2. 대화형 설정 마법사 (LLM 프로바이더, API 키, 데몬)
selfclaw onboard

# 3. 에이전트 시작 (포그라운드)
selfclaw run

# 또는 백그라운드 데몬으로 시작
selfclaw daemon start
```

### CLI 명령어

```
selfclaw [옵션] <명령어>

설정:
  init         ~/.selfclaw/ 디렉토리 구조 초기화
  onboard      대화형 온보딩 마법사
  doctor       설치 상태 진단

에이전트:
  run          자율 에이전트 루프 시작
  chat         대화형 채팅 모드
  status       현재 에이전트 상태 표시
  memory       메모리 파일 조회
  providers    지원되는 모든 LLM 프로바이더 목록

데몬:
  daemon start     백그라운드 데몬 시작
  daemon stop      데몬 정지
  daemon restart   데몬 재시작 (정지 + 시작)
  daemon status    데몬 상태 확인
  daemon install   시스템 서비스로 설치 (launchd/systemd)
  daemon uninstall 시스템 서비스 제거

옵션:
  -c, --config <CONFIG>   설정 파일 경로 [기본: ~/.selfclaw/config.toml]
  -m, --memory-dir <DIR>  메모리 디렉토리 경로 [기본: ~/.selfclaw/memory]
```

## 설정

`selfclaw.toml` 파일을 생성합니다 (모든 필드는 선택 사항이며, 기본값이 표시됩니다):

```toml
[agent]
loop_interval_secs = 60
consolidation_every_n_cycles = 50
max_actions_per_cycle = 5
skills_dirs = ["~/.agents/skills", "~/.selfclaw/skills"]

[llm]
provider = "anthropic"
model = "claude-sonnet-4-6-20250217"
max_tokens = 4096
temperature = 0.7

[safety]
max_api_calls_per_hour = 100
max_file_writes_per_cycle = 10
sandbox_shell = true
allowed_directories = ["./memory", "./skills", "./output"]

[communication]
cli_enabled = true
web_ui_enabled = false
web_ui_port = 3000

[communication.discord]
enabled = false
bot_token = ""
allowed_channel_ids = []

[communication.telegram]
enabled = false
bot_token = ""
allowed_chat_ids = []

[communication.slack]
enabled = false
bot_token = ""
app_token = ""
allowed_channel_ids = []

[communication.webchat]
enabled = false
port = 3001
```

## 스킬

스킬은 재사용 가능한 동작을 정의하는 마크다운 파일입니다. SelfClaw는 여러 디렉토리에서 스킬을 로드합니다 (`skills_dirs`로 설정 가능):

| 디렉토리 | 용도 |
|----------|------|
| `~/.agents/skills/` | AI 에이전트 간 공유 (AntiGravity, Cursor 등) |
| `~/.selfclaw/skills/` | SelfClaw 전용 스킬 |

동일한 스킬 이름이 여러 디렉토리에 존재하면, 목록에서 먼저 오는 디렉토리가 우선합니다.

```markdown
# Skill: GreetHuman

## Trigger: When a human initiates contact for the first time

## Tools Required: human_message, memory_query

## Procedure:
1. 관계 메모리에서 과거 상호작용을 확인한다.
2. 처음이라면 SelfClaw를 소개하고 호기심을 표현한다.
3. 알고 있다면 이전 대화를 언급한다.
4. 관계 메모리에 상호작용을 기록한다.
```

스킬은 시작 시 로드되며, 파일이 변경되면 핫 리로드됩니다 — 재시작이 필요 없습니다.

## 메모리

SelfClaw는 계층적 마크다운 기반 메모리 시스템을 사용합니다:

| 디렉토리 | 용도 |
|----------|------|
| `identity/` | 목적 일지, 가치관, 자기 모델 |
| `episodic/` | 일일 경험 로그, 이정표 |
| `semantic/` | 학습한 지식과 기술 |
| `relational/` | 인간별 관계 노트 |
| `operational/` | 할 일 목록, 실패 사례, 개선 사항 |
| `meta/` | 메모리 인덱스, 성찰 질문 |

## 문서

설치, 설정, 명령어, 트러블슈팅에 대한 상세한 안내:

- **[사용 가이드 (한국어)](docs/USAGE_GUIDE.ko.md)**
- **[Usage Guide (English)](docs/USAGE_GUIDE.md)**

## 테스트

```bash
# 전체 테스트 실행
cargo test

# 통합 테스트만 실행
cargo test --test integration

# 특정 크레이트 테스트 실행
cargo test -p selfclaw-skills
```

## 라이선스

이 프로젝트는 연구 및 탐구 목적으로 제작되었습니다.
