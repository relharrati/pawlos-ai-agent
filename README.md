# pawlos

> Persistent, self-evolving AI agent — Rust core, TypeScript web UI, SQLite memory

## Install

### 🖥️ Windows (PowerShell)
```powershell
iwr -useb https://raw.githubusercontent.com/relharrati/pawlos-ai-agent/main/scripts/install.ps1 | iex
```

### 🍎 Linux/macOS (curl)
```bash
curl -sSL https://raw.githubusercontent.com/relharrati/pawlos-ai-agent/main/scripts/install.sh | bash
```

### 📦 npm/npx
```bash
npx pawlos-ai
```

### 🛠️ Build from source (requires Rust)
```bash
cargo build --release -p cli
./target/release/pawlos
```

### First Run
```bash
pawlos          # Start the agent
pawlos onboard  # Run first-time setup
```

## Directory Structure

```
pawlos/
├── crates/              # Rust source code
│   ├── core/            # Config, DB, types
│   ├── memory/          # Memory system + vector embeddings
│   ├── provider/       # LLM providers
│   ├── prompt/         # Prompt builder
│   ├── tools/          # Tool executors
│   ├── orchestrator/   # Session manager, web server
│   └── cli/            # Binary
│
├── context/             # ← Your memory files
│   ├── SOUL.md         # Your identity
│   ├── USER.md         # User profile
│   ├── MEMORY.md       # Short-term memory (2200 chars)
│   └── LONGTERM.md     # Long-term learnings
│
├── agents/              # ← Sub-agents (created by you)
│   └── [agent-name]/
│       ├── AGENT.md    # Agent identity & role
│       ├── skills/     # Agent-specific skills
│       ├── tasks/      # Task tracking
│       │   ├── done/
│       │   ├── in_progress/
│       │   └── pending/
│       └── logs/       # Agent activity logs
│
├── memory/              # ← Daily logs
│   └── YYYY-MM-DD.md   # Daily log entries
│
├── skills/              # ← Reusable skills (created during chat)
│   └── skills/
│       ├── make_pdf.skill
│       ├── create_skill.skill
│       └── ...
│
├── web/                 # TypeScript web UI
├── tunnels/            # Discord/Telegram adapters
└── scripts/            # Install script
```

## Memory System

| Location | File | Purpose |
|----------|------|---------|
| `context/` | SOUL.md | Your identity & personality |
| `context/` | USER.md | User profile & preferences |
| `context/` | MEMORY.md | Short-term working memory (2200 char) |
| `context/` | LONGTERM.md | Curated long-term learnings |
| `memory/` | YYYY-MM-DD.md | Daily activity logs |

### Dual Storage
- **Markdown**: Human-readable, editable directly
- **Vector embeddings**: SQLite (`memory/vector.db`) + JSON backup

## Creating Skills

During chat, say "remember how to [do X]" or use `/skill create`:

```bash
# Example: create a skill for making PDFs
pawlos: Remember how to make a PDF, call it "make_pdf"
```

Skills are saved to `skills/skills/[name].skill`

## Creating Agents

You can spawn sub-agents for specialized tasks:

```
pawlos: Spawn an agent called "CodeReviewer" who reviews code
```

This creates `agents/CodeReviewer/` with its own AGENT.md, skills, and tasks.

## Commands

```bash
pawlos                    # Start (onboards on first run)
pawlos onboard            # Re-run setup
pawlos memory read soul  # Read a memory file
/skill make_pdf          # Load a skill
/recall "query"          # Semantic search
```

## Slash Commands

```
/personality concise     # Switch tone
/model openai/gpt-4o   # Switch model
/skill <name>          # Load a skill
/recall <query>        # Search long-term memory
/agent spawn <name>   # Create sub-agent
/memory                # View all memory stores
```