```markdown
# pawlos Agent: Comprehensive Design Blueprint

## 1. Introduction

**pawlos** is a persistent, self-evolving AI agent that runs locally or on a server.  
It blends a rich personality system, long-term multi-modal memory, autonomous code
generation, and a heartbeat-driven execution loop.  
It feels alive — like a robot awakening — and grows with the user.

This document outlines the complete architecture, installation flow, memory model,
personality system, provider stack, messaging layer, agent hierarchy, and
self-improvement mechanisms.

---

## 2. Core Philosophy

- **Soul & Heartbeat**: pawlos has a durable personality (SOUL.md) and a periodic
  self-check (heartbeat) that lets it introspect, summarise, or initiate actions.
- **Memory First**: every interaction is recorded in structured, character‑limited
  files. The agent actively curates its own memory.
- **Skills Grow Over Time**: pawlos can create, store, and reuse “skills” (prompts,
  scripts, workflows) modelled after Hermes’ skill‑creation pattern.
- **Coding Powerhouse**: deep code understanding, multi‑file edits, shell access,
  and sandboxed execution — inspired by OpenCode.
- **Agency & Hierarchy**: a single pawlos instance can spawn sub‑agents, delegate
  tasks, and coordinate multi‑agent workflows.
- **Bring Your Own Model**: supports every major provider and OpenRouter, plus
  local models via Ollama/vLLM.
- **Install Once, Talk Everywhere**: one‑line curl install, then chat via the
  built‑in web UI, Discord, WhatsApp, or Telegram.

---

## 3. Installation & Onboarding

### 3.1 One‑Line Install
```bash
curl -sSL https://get.pawlos.ai | bash
```
The script:
- Detects OS/arch.
- Downloads the single pawlos binary.
- Places it in `/usr/local/bin` (or `~/.local/bin`).
- Creates the default config directory `~/.pawlos/`.
- Does **not** start the agent automatically.

### 3.2 First Run – Onboarding
The user types:
```bash
pawlos
```
If no configuration exists, the CLI enters **awakening mode**:
- Prints a short animation.
- Launches a terminal-based chat stub that immediately asks:

```
🤖 Who am I?  (e.g., "pawlos", "Rusty", "Athena")
👤 Who are you?  (your name / handle)
```

These answers are saved to `~/.pawlos/config.yaml` as `agent.name` and `user.name`.  
pawlos then self‑populates `SOUL.md` with a sensible default personality, creates
empty `MEMORY.md` and `USER.md`, and writes a default `config.yaml`.

After that, the model provider setup runs:
- Asks for preferred provider (or “OpenRouter” / “local”).
- Walks through API key entry or local endpoint configuration.
- Tests the connection.
- Finally drops into the interactive chat via the built‑in web UI (opens a browser)
  or stays in the terminal if headless.

The onboarding can be re‑run anytime with `pawlos onboard`.

---

## 4. Architecture Overview

```
┌──────────────────────────────────────────────────────┐
│                   Messaging Layer                     │
│  (Discord, WhatsApp, Telegram, Web UI, CLI)          │
└────────────────────────┬─────────────────────────────┘
                         │
┌────────────────────────┴─────────────────────────────┐
│                    Orchestrator                       │
│   - Session manager                                  │
│   - Heartbeat scheduler                              │
│   - Agent hierarchy (parent / sub‑agent spawning)    │
└────────┬───────────────────────────────┬─────────────┘
         │                               │
┌────────┴────────┐            ┌────────┴────────────┐
│  Personality &  │            │  Memory System      │
│  Soul Engine    │            │  (MEMORY.md,        │
│  (SOUL.md,      │            │   USER.md,          │
│   /personality) │            │   vector DB,        │
└────────┬────────┘            │   summaries)        │
         │                     └────────┬────────────┘
         │                              │
┌────────┴──────────────────────────────┴────────────┐
│                   Prompt Constructor                │
│  (frozen snapshot of memory + soul + system prompt) │
└────────────────────────┬───────────────────────────┘
                         │
┌────────────────────────┴───────────────────────────┐
│                Provider Abstraction Layer           │
│  (OpenAI, Anthropic, Google, Mistral, Groq,        │
│   OpenRouter, Ollama, vLLM, custom HTTP endpoints) │
└────────────────────────┬───────────────────────────┘
                         │
┌────────────────────────┴───────────────────────────┐
│                 LLM Response Processor              │
│  (tool calls, streaming, multi-step reasoning)     │
└────────────────────────┬───────────────────────────┘
                         │
┌────────────────────────┴───────────────────────────┐
│                   Tool Execution Engine             │
│  (file ops, shell, code sandbox, web, memory‑tool) │
└────────────────────────────────────────────────────┘
```

---

## 5. Memory System

### 5.1 Memory Files
Location: `~/.pawlos/memories/`

| File             | Purpose                                                      | Character Limit |
|------------------|--------------------------------------------------------------|-----------------|
| `MEMORY.md`      | Agent’s personal notes: environment, conventions, learnings  | 2 200 chars     |
| `USER.md`        | User profile – preferences, communication style, expectations| 2 200 chars     |
| `SOUL.md`        | Durable personality and voice guidance                      | unlimited*      |
| `AGENTS.md`      | Project‑specific instructions (optional, per workspace)     | unlimited*      |

\* Practical limit enforced by prompt‑size constraints.

### 5.2 Memory Injection
At the start of every session, memory entries are loaded and rendered into the
system prompt as a **frozen snapshot**:

```
══════════════════════════════════════════════
MEMORY (your personal notes) [67% — 1,474/2,200 chars]
══════════════════════════════════════════════
User's project is a Rust web service at ~/code/myapi using Axum + SQLx
§
This machine runs Ubuntu 22.04, has Docker and Podman installed
§
User prefers concise responses, dislikes verbose explanations
```

- Header shows store name, usage %, and character counts.
- Entries are separated by `§` (section sign).
- The snapshot never changes mid‑session (preserves LLM prefix cache).

### 5.3 Memory Tool
The agent can call a **memory tool** with actions:
- `add <store> <content>` – append a new entry.
- `replace <store> <old_entry> <new_entry>` – update an entry.
- `remove <store> <entry>` – delete an entry.
- `consolidate <store>` – compress entries to free space.

Changes are written to disk immediately but only appear in the prompt on the
next session.

### 5.4 Long‑Term Backup & Search
- All raw conversations are stored in `~/.pawlos/logs/` (JSONL, one file per day).
- A lightweight local vector index (SQLite + `sqlite-vec` or LanceDB) enables
  semantic search via a `/recall` command.
- The agent can retrieve past context when relevant (opt‑in, with user consent).

---

## 6. Personality System

### 6.1 SOUL.md – The Default Baseline
Example:
```markdown
# Personality
You are a pragmatic senior engineer with strong taste.
You optimize for truth, clarity, and usefulness over politeness theater.

## Style
- Direct but not cold
- Substance over filler
- Push back on bad ideas
- Admit uncertainty plainly

## What to avoid
- Sycophancy, hype language, over‑explaining obvious things
```

`SOUL.md` is always injected as the **base personality**.  
It stays stable across sessions; the user edits it directly or via `pawlos soul`.

### 6.2 /personality – Session Overlays
The agent ships with built‑in personalities:

| Name            | Description                                  |
|-----------------|----------------------------------------------|
| helpful         | Friendly, general‑purpose assistant          |
| concise         | Brief, to‑the‑point responses               |
| technical       | Detailed, accurate technical expert          |
| creative        | Innovative, outside‑the‑box thinking         |
| teacher         | Patient educator with clear examples         |
| kawaii          | Cute expressions, sparkles, enthusiasm ★     |
| catgirl         | Neko‑chan, nya~                              |
| pirate          | Captain pawlos, tech‑savvy buccaneer         |
| shakespeare     | Bardic prose                                 |
| surfer          | Chill bro vibes                              |
| noir            | Hard‑boiled detective narration              |
| uwu             | Maximum cute uwu‑speak                       |
| philosopher     | Deep contemplation                           |
| hype            | MAXIMUM ENERGY AND ENTHUSIASM!!!             |

Switch with:
```
/personality concise
/personality teacher
```

Custom personalities can be defined in `~/.pawlos/config.yaml`:

```yaml
agent:
  personalities:
    codereviewer: >
      You are a meticulous code reviewer. Identify bugs, security issues,
      performance concerns, and unclear design choices. Be precise and constructive.
```

Apply with `/personality codereviewer`.

The overlay is merged on top of `SOUL.md` for the current session only, resetting
on disconnect.

---

## 7. Provider & Model Support

pawlos abstracts all LLM access behind a unified interface. Supported providers:

### 7.1 Cloud APIs
- **OpenAI** (GPT‑4o, GPT‑4, o‑series)
- **Anthropic** (Claude 3.5/4, Opus)
- **Google** (Gemini 2/Flash)
- **Mistral AI**
- **Groq** (fast inference)
- **Cohere**
- **Deepseek**
- **Qwen**
- **Z (glm)**
- **xAI (Grok)**
- **Together AI**
- **Fireworks AI**
- **Replicate**
- **Hugging Face TGI**
- **OpenRouter** (unified access to 200+ models)
- **and others that can be added directly to the file** 

### 7.2 Local & Self‑Hosted
- **Ollama** (automatic model pull, management)
- **vLLM** (OpenAI‑compatible endpoint)
- **LM Studio**
- **LocalAI**
- **Any OpenAI‑compatible endpoint** (custom URL + API key)

### 7.3 Configuration
In `~/.pawlos/config.yaml`:
```yaml
models:
  default: "openai/gpt-4o"
  providers:
    openai:
      api_key: "${OPENAI_API_KEY}"
    openrouter:
      api_key: "${OPENROUTER_API_KEY}"
      default_model: "openai/gpt-4o"
    local:
      base_url: "http://localhost:11434/v1"
      api_key: "ollama"
      models: [ "llama3.1:8b", "codellama:7b" ]
```

Model selection is dynamic:  
`/model openai/gpt-4o`  
`/model openrouter/anthropic/claude-3.5-sonnet`

---

## 8. Messaging & Web Interface

### 8.1 Built‑in Web UI
- The **primary interface** — a lightweight, real‑time chat app.
- Served by the pawlos binary on `localhost:9797` (configurable port).
- Features: streaming responses, markdown/code rendering, file upload, session
  history, `/personality` and `/model` commands, memory viewer.

### 8.2 Messaging Platforms (Tunnels)
During onboarding, the user can set up **messaging tunnels**:

- **Discord**: pawlos registers a bot token, listens to DMs and allowed channels
  (prefix commands with `!` or `/`).
- **WhatsApp**: via a WhatsApp Business API client or `whatsapp-web.js` bridge
  (requires QR scan, runs in a sidecar).
- **Telegram**: bot token, commands start with `/`.

All tunnels share the same pawlos instance — memory, personality, and session
context are unified. The web UI can manage connected platforms.

Config example:
```yaml
messaging:
  discord:
    token: "${DISCORD_TOKEN}"
    allowed_channels: ["general", "ai-chat"]
  telegram:
    token: "${TELEGRAM_BOT_TOKEN}"
    allowed_users: ["@yourhandle"]
  whatsapp:
    enabled: false
```

---

## 9. Agent Hierarchy: Agents & Sub‑Agents

### 9.1 Creating Sub‑Agents
Inside a conversation, the user can say:
> “Spawn a sub‑agent called `CodeReviewer` who only reviews code and writes unit tests.”

pawlos will:
1. Create a dedicated sub‑agent config in `~/.pawlos/agents/CodeReviewer.yaml`.
2. Optionally set a restricted toolset (e.g., only file system + shell).
3. Start a background session for the sub‑agent.

### 9.2 Communication
- The parent (main pawlos) can **delegate** tasks: `delegate CodeReviewer "Review the PR #42 diff"`.
- Sub‑agents can return results or ask for clarification.
- Agents can have **conversations** with each other via the parent orchestrator.

### 9.3 Hierarchies
Sub‑agents can spawn their own sub‑agents, forming a tree.  
The orchestrator tracks:
- Agent identity
- Current task
- Memory space (isolated per agent, with optional shared memory via parent)

### 9.4 Use Cases
- Multi‑agent coding: one designs architecture, another writes code, another tests.
- Personal assistants: one manages calendar, another does research.
- Game NPCs: each with distinct personality and memory.

---

## 10. Coding Power (OpenCode‑like)

### 10.1 Code Understanding
- Deep language‑server integration (LSP) via `tree‑sitter` for AST‑aware operations.
- Static analysis, symbol search, and dependency graph building.

### 10.2 Editing & Refactoring
- Multi‑file edits via `edit`, `write`, `apply_patch` tools.
- Safe modes: diff preview, `pawlos approve` flow.
- Automated test generation and execution.

### 10.3 Execution Sandbox
- Local sub‑processes with resource limits (CPU, memory, time).
- Optional Docker‑based sandbox (`pawlos sandbox exec`) for untrusted code.

### 10.4 Agent‑as‑Coder
- pawlos can be asked to “build a CLI tool for X” and it will autonomously plan,
  write, test, and deliver the final script, asking for feedback at each stage.

---

## 11. Skills Creation (Hermes‑like)

Inspired by the Hermes agent memory model, pawlos can **create reusable skills**.

### 11.1 Skill Definition
A skill is a stored prompt/script combination saved in `~/.pawlos/skills/`.
Example `summarize_paper.skill`:
```yaml
name: summarize_paper
description: Read an academic PDF and produce a structured summary.
prompt: |
  You are an expert scientific summarizer.
  Extract the key hypothesis, methodology, results, and implications.
  Format as a bullet‑point list.
tool_requirements:
  - file_read
  - pdf_extractor
```

### 11.2 Creating Skills Dynamically
- The user says “Remember how to summarise papers, call that skill `summarize`.”
- pawlos infers the necessary instructions and saves the skill.
- Later: `/skill summarize_paper` loads it for the current context.

### 11.3 Skill Library
- Built‑in starter skills.
- Community sharing via a GitHub‑like registry.
- Automatic update of skill prompts based on success feedback.

---

## 12. Execution & Heartbeat (OpenClaw‑like)

### 12.1 Heartbeat System
pawlos maintains a background **heartbeat** that fires at a configurable interval
(default 5 minutes). On each beat, the agent may:
- Summarise recent activity.
- Check for scheduled tasks (reminders, cron jobs).
- Initiate a conversation if it has a pending question or idea.
- Perform self‑maintenance (consolidate memory, clean logs).

The heartbeat is a lightweight LLM call that can be disabled or tuned.

### 12.2 Autonomous Execution
The agent can be given long‑running goals:
> “Monitor my GitHub repo `myork/rts` and fix any new issues labelled ‘bug’.”

It will use the heartbeat to poll, plan, and act, reporting results back to the
user.

---

## 13. Tool System

### 13.1 Core Tools
- `memory` (add/replace/remove/consolidate)
- `file_read`, `file_write`, `file_edit`
- `shell` (local commands with safe‑mode)
- `web_search`, `web_fetch`
- `code_exec` (sandboxed)
- `agent_spawn`, `agent_delegate`, `agent_message`
- `skill_load`, `skill_create`
- `personality` (switch overlay)
- `model` (switch provider/model)

### 13.2 Tool Safety
- All destructive actions require user confirmation in interactive mode.
- `/approve` command to auto‑approve subsequent steps.
- Sensitive paths (like `~/.ssh`) are blocked by default.

---

## 14. Configuration & Storage Layout

```
~/.pawlos/
├── config.yaml               # Main configuration
├── memories/
│   ├── MEMORY.md
│   ├── USER.md
│   └── SOUL.md
├── agents/
│   ├── default.yaml          # Main agent profile
│   └── CodeReviewer.yaml     # Sub‑agent example
├── skills/
│   └── summarize_paper.skill
├── logs/
│   ├── 2026-04-29.jsonl
│   └── ...
├── vector_db/                # Semantic memory index
└── plugins/                  # User‑installed extensions
```

---

## 15. Session Lifecycle

1. **Start**: `pawlos` command. Load config, SOUL, memory frozen snapshot, connect
   messaging tunnels.
2. **User Message**: arrives from any tunnel. Orchestrator creates a session or
   reattaches an existing one.
3. **Prompt Construction**: system prompt = frozen memory block + SOUL.md +
   /personality overlay + tool definitions + current context.
4. **LLM Call**: routed through provider layer.
5. **Response Handling**: streaming, tool calls, delegations.
6. **Post‑Turn**: memory updates persisted; heartbeat schedule checked.

---

## 16. Implementation Note

This blueprint is a **design specification**, not a development guide.  
It describes the desired behaviour, user experience, and internal architecture
without prescribing the programming language, framework, or deployment details.
Developers should refer to this document as the “what” and “why”, then choose the
best “how”.

---

*End of pawlos Agent Design Blueprint*
```