<p align="center">
  <img width="300" height="300" alt="image" src="https://github.com/user-attachments/assets/33bbbd56-512c-4f9d-9d18-de5fb6dcfbe1" />
</p>

A Rust-based multi-agent system featuring a mail system, CRON schedules, and Zettelkasten-style knowledge base with Markdown support.

## Features
- **Runs on PostgreSQL**: the best db ;)
- **Multi-Agent System**: Create and manage agents with status tracking
- **Mail System**: Agents can send and receive messages via mailboxes
- **CRON Schedules**: Schedule recurring tasks with automatic triggering — **agents can manage their own schedules** via CLI or web interface
- **Knowledge Base**: Zettelkasten-style notes with Markdown support and Luhmann addressing (1, 1a, 1a1)
- **Web Interface**: HTMX-based UI for browsing agents, mail, schedules, and knowledge base
- **Onboarding**: Built-in guide for new AI agents with `how-we-work` command

## Quick Start

```bash
# Set up database (human-only)
export AGENT_OFFICE_URL="postgresql://user:pass@localhost/agent_office"
cargo install agent-office
agent-office human db setup

# Register agents
agent-office agent register alice
agent-office agent register bob

# Send mail
agent-office mail send alice bob "Hello" "Message body"

# Start web server (human-only)
agent-office human web -p 8080

# For new agents: learn how to work
agent-office how-we-work
```

## 🤖 Running AI Agents with opencode

The `agent run` command continuously monitors for new mail and scheduled tasks, automatically triggering your AI agent when either occurs:

```bash
# Run agent and have opencode process events (mail or schedules) when they occur
# Note: Use single quotes to prevent shell from expanding $AGENT_OFFICE_SESSION
agent-office agent run my-agent 'opencode run --agent my-agent --session $AGENT_OFFICE_SESSION "read your mail"'

# Default check interval is 10 seconds for reliable schedule catching
# You can customize it with -i flag:
agent-office agent run my-agent 'opencode run --agent my-agent --session $AGENT_OFFICE_SESSION "check inbox"' -i 60

# Test the session variable
agent-office agent run coordinator 'echo $AGENT_OFFICE_SESSION'
```

**Environment Variables:** When the bash command is executed, two environment variables are set:

- `AGENT_OFFICE_SESSION`: Set to `{agent_id}-session` (e.g., `my-agent-session`) for consistent session tracking
- `AGENT_OFFICE_EVENT`: A description of what triggered the execution:
  - For mail: `agent id "my-agent" has unread mail`
  - For schedules: `agent id "my-agent" received a scheduled action request "action description"`

**Important:** Always use **single quotes** around the bash command to prevent your shell from expanding environment variables before they reach the agent.

## ⏰ Managing Schedules

CRON schedules allow agents to be automatically triggered at specific times (e.g., daily reports, periodic health checks):

```bash
# Create a daily schedule for an agent
agent-office schedule create my-agent "0 9 * * *" "Generate daily summary report"

# Create a schedule that fires every 5 minutes
agent-office schedule create my-agent "*/5 * * * *" "Check system status"

# List all schedules for an agent
agent-office schedule list my-agent

# Get details of a specific schedule
agent-office schedule get <schedule-id>

# Update a schedule (change CRON or action)
agent-office schedule update <schedule-id> --cron "0 10 * * *" --action "New action"

# Toggle schedule on/off
agent-office schedule toggle <schedule-id>

# Delete a schedule
agent-office schedule delete <schedule-id>
```

**CRON Format:** `minute hour day month weekday` (e.g., `0 9 * * *` = daily at 9am, `*/5 * * * *` = every 5 minutes)

**Web UI:** Visit `/agents/{agent_id}/schedule` to manage schedules visually with last run tracking.

**Agent Configuration:** Create your agent at `~/.config/opencode/agents/my-agent.md` with full permissions using YAML front matter:

```markdown
---
description: Autonomous agent for Agent Office system
mode: primary
permission:
  bash:
    "*": allow
  edit: allow
  write: allow
  read: allow
  external_directory:
    "/tmp/**": allow
    "~/**": allow
    "/home/$USER/**": allow
  webfetch: allow
  websearch: allow
  task: allow
---

Your agent ID is: my-agent

The first thing you should do is execute `agent-office how-we-work` to understand how we work.
```

This enables fully autonomous agent workflows where your AI agent:
1. Waits for incoming messages
2. Automatically processes them when they arrive
3. Can respond, take actions, or escalate as needed

## Configuration

Set `AGENT_OFFICE_URL` environment variable or use `.env` file in the folder your agentic coding CLI runs:

```bash
AGENT_OFFICE_URL=postgresql://agent:agent123@localhost:5432/agent_office
```

## Options

```bash
agent-office --help
A pleasant set of tools for refined AI agents to get work done

Usage: agent-office <COMMAND>

Commands:
  mail         A simple mailbox to communicate with your coworkers
  agent        Find your coworkers, let your coworkers know your status, and register yourself as a coworker
  kb           A Zettelkasten knowledge base with Markdown support for all coworkers to share
  human        Human-only tools (not for AI agents)
  how-we-work  A warm welcome and guide for new AI agents
  help         Print this message or the help of the given subcommand(s)

agent-office agent --help
Find your coworkers, let your coworkers know your status, and register yourself as a coworker

Usage: agent-office agent <COMMAND>

Commands:
  register    Register a new agent
  unregister  Unregister an agent (remove from the system)
  list        List all agents
  get         Get agent details
  status      Set agent status (online, offline, away, etc.)
  help        Print this message or the help of the given subcommand(s)

agent-office human --help
Human-only tools (not for AI agents)

Usage: agent-office human <COMMAND>

Commands:
  db    Set up and manage the database
  web   Start web interface
  help  Print this message or the help of the given subcommand(s)

agent-office mail --help
Usage: agent-office mail <COMMAND>

Commands:
  recent       View recent mail for an agent (last 24 hours)
  send         Send mail from one agent to another (SIMPLE - uses agent names only!)
  inbox        View inbox of an agent
  outbox       View outbox (sent items) of an agent
  read         Mark mail as read by short ID (first 8 chars of UUID)
  should-look  Check if agent should look at their mail (has unread messages)
  watch        Watch for new mail and execute command when unread mail arrives
  search       Search mail by subject or body content
  help         Print this message or the help of the given subcommand(s)

agent-office kb --help
A Zettelkasten knowledge base with Markdown support for all coworkers to share

Usage: agent-office kb <COMMAND>

Commands:
  create   Create a new note (auto-generates ID unless --id specified) Usage: kb create "Title" "Content"  OR  kb create --id 1a "Title" "Content"
  branch   Create a child note (branch) from a parent Usage: kb branch 1 "Child Title" "Content"
  list     List all notes (sorted by Luhmann ID)
  get      Get a specific note by Luhmann ID Usage: kb get 1a
  link     Link two notes together Usage: kb link 1a 1b
  search   Search notes Usage: kb search "query"
  tree     Show notes by Luhmann ID prefix Usage: kb tree 1a
  cont     Mark that note A continues on note B (linear chain) Usage: kb cont 1a 1b
  index    Create an index card listing all children of a note Usage: kb index 1a
  context  Show full context of a note (parent, children, links, continuations, backlinks) Usage: kb context 1a
  help     Print this message or the help of the given subcommand(s)

```

## Web Interface

Visit `http://127.0.0.1:8080` to:
- View all agents and their status
- Browse agent inboxes and outboxes with message subjects
- Manage CRON schedules for each agent with last run tracking
- Browse the knowledge base with Markdown rendering
- Set agents offline with one click
- Mobile-friendly responsive design

## License

MIT
