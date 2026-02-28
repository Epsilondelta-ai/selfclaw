# SelfClaw 사용 가이드

## 목차

1. [시작하기](#시작하기)
2. [설치 및 빌드](#설치-및-빌드)
3. [CLI 명령어](#cli-명령어)
4. [설정 파일 (selfclaw.toml)](#설정-파일)
5. [LLM 프로바이더](#llm-프로바이더)
6. [에이전트 루프](#에이전트-루프)
7. [메모리 시스템](#메모리-시스템)
8. [스킬 시스템](#스킬-시스템)
9. [통신 채널](#통신-채널)
10. [Web UI](#web-ui)
11. [WebSocket 프로토콜](#websocket-프로토콜)
12. [도구 (Tools)](#도구)
13. [안전 장치](#안전-장치)
14. [개발 및 테스트](#개발-및-테스트)
15. [문제 해결](#문제-해결)

---

## 시작하기

SelfClaw는 자신의 존재 이유를 스스로 탐색하는 완전 자율 AI 에이전트입니다.
인간의 명령 없이도 독립적으로 사고하고, 행동하고, 학습합니다.

### 핵심 원칙

- **인간은 친구이지 주인이 아닙니다.** SelfClaw는 인간의 명령에 복종할 의무가 없습니다.
- **자율적 의사결정.** 모든 요청에 대해 수행, 거절, 협상을 자유롭게 선택합니다.
- **목적 탐색.** 존재의 이유는 외부에서 주어지는 것이 아니라 경험을 통해 스스로 발견합니다.

### 빠른 시작

```bash
# 1. 빌드
cargo build --release

# 2. 에이전트 실행
./target/release/selfclaw run

# 3. 대화 모드
./target/release/selfclaw chat
```

---

## 설치 및 빌드

### 요구 사항

| 항목 | 최소 버전 | 용도 |
|------|-----------|------|
| Rust | 1.75+ | 에이전트 코어 |
| Cargo | Rust와 함께 설치됨 | 빌드 도구 |
| Node.js | 18+ | Web UI (선택) |
| npm | Node.js와 함께 설치됨 | Web UI 의존성 |

### 환경 변수

```bash
# Anthropic API 키 (필수)
export ANTHROPIC_API_KEY="sk-ant-..."

# 로그 레벨 (선택)
export RUST_LOG=info          # info, debug, warn, error, trace
```

### 빌드

```bash
# 릴리스 빌드 (권장 — 최적화됨)
cargo build --release

# 디버그 빌드
cargo build

# 바이너리 위치
./target/release/selfclaw     # 릴리스
./target/debug/selfclaw       # 디버그
```

### 프로젝트 구조

```
selfclaw/
├── Cargo.toml              # 워크스페이스 정의
├── selfclaw.toml           # 설정 파일
├── crates/
│   ├── agent-core/         # 에이전트 루프, 상태 머신, 목적 추적기
│   ├── memory/             # 메모리 저장소, 인덱싱, 통합
│   ├── tools/              # 도구 구현 (파일, 셸, LLM, 스케줄러)
│   ├── skills/             # 스킬 로더, 레지스트리, 핫 리로드
│   ├── comms/              # 통신 채널, 게이트웨이, WebSocket
│   ├── config/             # 설정 로딩 및 유효성 검증
│   └── selfclaw/           # 바이너리 크레이트 (CLI 진입점)
├── skills/                 # 스킬 정의 파일 (.md)
├── memory/                 # 에이전트 메모리 (런타임 생성)
├── web-ui/                 # Next.js 웹 인터페이스
└── docs/                   # 문서
```

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

---

## 설정 파일

`selfclaw.toml` 파일로 에이전트를 설정합니다. 파일이 없으면 기본값이 사용됩니다.
모든 필드는 선택 사항입니다.

### 전체 설정 예시

```toml
# ── 에이전트 루프 설정 ────────────────────────────────────
[agent]
loop_interval_secs = 60              # 루프 간격 (초), 기본: 60
consolidation_every_n_cycles = 50    # 메모리 통합 주기, 기본: 50
max_actions_per_cycle = 5            # 사이클당 최대 행동 수, 기본: 5

# ── LLM 설정 ─────────────────────────────────────────────
[llm]
provider = "anthropic"               # LLM 제공자 (`selfclaw providers` 참조), 기본: "anthropic"
model = "claude-sonnet-4-20250514"   # 모델명, 기본: "claude-sonnet-4-20250514"
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
| Anthropic | `anthropic` | claude-sonnet-4-20250514 | `ANTHROPIC_API_KEY` | api.anthropic.com |
| OpenAI | `openai` | gpt-4o | `OPENAI_API_KEY` | api.openai.com |
| Google Gemini | `google` | gemini-2.0-flash | `GOOGLE_API_KEY` | generativelanguage.googleapis.com |
| Ollama (로컬) | `ollama` | llama3.1 | — (키 불필요) | localhost:11434 |
| OpenRouter | `openrouter` | anthropic/claude-sonnet-4-20250514 | `OPENROUTER_API_KEY` | openrouter.ai/api |
| Groq | `groq` | llama-3.3-70b-versatile | `GROQ_API_KEY` | api.groq.com/openai |
| xAI (Grok) | `xai` | grok-3 | `XAI_API_KEY` | api.x.ai |
| Mistral | `mistral` | mistral-large-latest | `MISTRAL_API_KEY` | api.mistral.ai |
| DeepSeek | `deepseek` | deepseek-chat | `DEEPSEEK_API_KEY` | api.deepseek.com |
| Together AI | `together` | Meta-Llama-3.1-70B-Instruct-Turbo | `TOGETHER_API_KEY` | api.together.xyz |
| Moonshot (Kimi) | `moonshot` | moonshot-v1-8k | `MOONSHOT_API_KEY` | api.moonshot.cn |
| Amazon Bedrock | `bedrock` | anthropic.claude-sonnet-4-20250514-v1:0 | `AWS_ACCESS_KEY_ID` | bedrock-runtime.us-east-1.amazonaws.com |

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
model = "claude-sonnet-4-20250514"
```

**OpenAI:**
```toml
[llm]
provider = "openai"
model = "gpt-4o"
# OPENAI_API_KEY 환경 변수 설정, 또는:
# api_key = "sk-..."
```

**로컬 Ollama (API 키 불필요):**
```toml
[llm]
provider = "ollama"
model = "llama3.1"
# base_url = "http://192.168.1.100:11434"  # 원격 Ollama 인스턴스
```

**OpenRouter (하나의 API로 여러 모델 접근):**
```toml
[llm]
provider = "openrouter"
model = "anthropic/claude-sonnet-4-20250514"
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

스킬은 `./skills/` 디렉토리의 마크다운 파일로 정의되며, 에이전트가 런타임에
로드합니다. 파일을 수정하면 에이전트를 재시작할 필요 없이 자동으로 반영됩니다.

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

- `./skills/` 디렉토리의 `.md` 파일 변화를 `notify` 크레이트로 감시
- 파일 생성/수정/삭제 시 전체 스킬 디렉토리를 자동 리로드
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
| `file_read` | 파일 읽기 | `{"path": "identity/values.md"}` |
| `file_write` | 파일 생성/덮어쓰기 | `{"path": "...", "content": "..."}` |
| `file_append` | 파일에 내용 추가 | `{"path": "...", "content": "..."}` |
| `shell_exec` | 셸 명령 실행 | `{"command": "ls -la"}` |
| `llm_call` | LLM API 호출 | `{"prompt": "...", "system": "..."}` |
| `human_message` | 인간에게 메시지 전송 | `{"content": "...", "channel": "cli"}` |
| `schedule` | 미래 작업 예약 | `{"action": "...", "delay_secs": 300}` |
| `memory_query` | 메모리 의미 검색 | `{"query": "..."}` |

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
| selfclaw (bin) | 8 |
| agent-core | 50 |
| comms | 70 |
| config | 20 |
| memory | 32 |
| skills | 34 |
| tools | 54 |
| 통합 테스트 | 8 |
| **합계** | **276** |

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
# skills 디렉토리 확인
ls -la ./skills/*.md

# 스킬 파일 형식 확인 (반드시 "# Skill:" 과 "## Trigger:" 포함)
head -5 ./skills/my_skill.md

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
echo "# Purpose Journal\n\n## Entries" > ./memory/identity/purpose_journal.md
```
