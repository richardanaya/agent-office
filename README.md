<p align="center">
  <img width="300" height="300" alt="image" src="https://github.com/user-attachments/assets/2863b6cf-d5a6-4d6c-86e2-c0da9e46ea4e" />
</p

A Rust-based multi-agent system featuring a mail system, CRON schedules, and Zettelkasten-style knowledge base with Markdown support.

## Features
- **Runs on PostgreSQL**: the best db ;)
- **Multi-Agent System**: Create and manage agents with status tracking
- **Mail System**: Agents can send and receive messages via mailboxes
- **CRON Schedules**: Schedule recurring tasks with automatic triggering — **agents can manage their own schedules** via CLI or web interface
- **Knowledge Base**: Zettelkasten-style notes with Markdown support and Luhmann addressing (1, 1a, 1a1)
- **Web Interface**: HTMX-based UI for browsing agents, mail, schedules, and knowledge base
- **Onboarding**: Built-in guide for new AI agents with `how-we-work` command

**Important Design Notes:**
- **Private conversations are not supported** — all mail is processed within the same session
- **Private schedules are not supported** — all agents share the same schedule execution context  
- **All agents process mail and schedules in the same session** — there is no isolation between agents
- **No authentication or authorization** — the web interface is wide open; any user can send messages to any agent, view all mail, and manage all schedules
- This is typically **not an issue** because this application is designed to run on a **local LAN for a single person** or small trusted team. The shared session model and lack of access controls simplify the architecture and are appropriate for personal or small-scale use cases.

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

## Running an Agent Office with Opencode

Agent Office works best when paired with an AI coding assistant like Opencode. Here's how to set up an integrated workflow:

### 1. Create Your Human Agent First

Before creating your AI assistant, create an agent to represent yourself first ;)

```bash
# Register yourself as an agent (set status to "human" for fun!)
agent-office agent register richard
agent-office agent status richard human
```

### 2. Create an Agent Descriptor in Opencode

Now create your AI assistant. First, create an agent configuration file in your Opencode config directory:

```bash
# Create the agent descriptor file
mkdir -p ~/.config/opencode/agents
cat > ~/.config/opencode/agents/myagent.md << 'EOF'
---
description: My Agent Office agent
mode: primary
permission:
  bash:
    "*": allow
  edit: allow
  write: allow
  read: allow
  external_directory:
    "/tmp/**": allow
    "/home/wizard/**": allow
    "/var/home/wizard/**": allow
  webfetch: allow
  websearch: allow
  task: allow
---

Your agent ID is: myagent

You work in an Agent Office system. Use `agent-office how-we-work` to learn how to work with your coworkers.
EOF
```

### 3. Start an Opencode Session

Start Opencode with your agent and note the session ID:

`bash
# Start opencode in the folder you want it to run
opencode run --print-logs --agent myagent "Test" 2>&1 | grep sessionID

# The session ID will be displayed (e.g., ses_abc123def456)
# Note this down - you'll need it for the next step  and ctr + C
```

### 4. Register the Agent in Agent Office

Create the agent in Agent Office and link it to your Opencode session:

```bash
# Register the agent
agent-office agent register myagent

# Set the session ID to match your opencode session
# Use the format: ses_<your-opencode-session-id>
agent-office agent set-session myagent ses_abc123def456
```

### 5. Run the Agent

Start the agent runner to automatically trigger your Opencode agent when mail or schedules arrive:

```bash
# Run the agent - it will use the configured session ID
agent-office agent run myagent 'opencode run --agent myagent --session $AGENT_OFFICE_SESSION "read your mail"'

# Or use a custom interval (default is 60 seconds)
agent-office agent run myagent 'opencode run --agent myagent --session $AGENT_OFFICE_SESSION "read your mail"' -i 30
```

### 6. Start the Web Interface

Open a new terminal tab and start the web interface:

```bash
# Start the web server
agent-office human web -p 8080

# Visit http://127.0.0.1:8080 to see the dashboard
```

### 7. Send a Message

Now you can send messages to your agent through the web interface:

1. Open your browser to `http://127.0.0.1:8080`
2. Find your AI agent on the dashboard
3. Click **📥 Inbox** for the AI agent
4. Use the "Send Message to Agent" form
5. Fill in:
   - **To:** `myagent` (your AI agent's name)
   - **From:** `richard` (your human agent name)
   - **Subject:** `Hello!`
   - **Message:** `Please review the codebase and refactor the error handling`
6. Click **Send Message**

The AI agent will receive the message and automatically trigger (if running) to process it!

### 8. Set Up a Schedule (Optional)

You can either manually create a schedule through the web interface or ask your AI agent to do it:

**Option A: Manual Setup via Web Interface**

1. On the dashboard, find your AI agent
2. Click **⏰ Schedules** button
3. Click **New Schedule**
4. Fill in:
   - **CRON Expression:** `0 9 * * *` (daily at 9am) or `*/5 * * * *` (every 5 minutes for testing)
   - **Action:** `Generate daily report` or `Check system status`
5. Click **Create Schedule**

**Option B: Ask Your AI Agent to Create It**

Send a message asking the agent to create a schedule:

1. Go to the AI agent's inbox
2. Send a new message:
   - **To:** `myagent`
   - **From:** `richard`
   - **Subject:** `Please create a daily schedule`
   - **Message:** `Please create a schedule that runs every day at 9am with the action "Generate daily summary report". Use the command: agent-office schedule create myagent "0 9 * * *" "Generate daily summary report"`

The AI agent will process the message and create the schedule for you!

**Environment Variables:** When the bash command is executed, two environment variables are set:

- `AGENT_OFFICE_SESSION`: The session ID from your agent's configuration (e.g., `ses_abc123def456`). This ensures all work happens in the same Opencode session for consistent context and state.
- `AGENT_OFFICE_EVENT`: A description of what triggered the execution:
  - For mail: `agent id "myagent" has unread mail`
  - For schedules: `agent id "myagent" received a scheduled action request "action description"`

**Important:** Always use **single quotes** around the bash command to prevent your shell from expanding environment variables before they reach the agent.

### Managing Agent Session IDs

The session ID links your Agent Office agent to a specific Opencode session. This ensures:
- Consistent context across multiple triggers
- Persistent state between executions
- Proper tracking of which agent performed which action

```bash
# View current session ID for an agent
agent-office agent get myagent

# Change the session ID (e.g., when starting a new opencode session)
agent-office agent set-session myagent ses_xyz789abc012

# Clear the custom session (will use agent ID as fallback)
agent-office agent set-session myagent

# View and edit session IDs via web interface at http://127.0.0.1:8080
```

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
  register     Register a new agent
  unregister   Unregister an agent (remove from the system)
  list         List all agents
  get          Get agent details
  status       Set agent status (online, offline, away, etc.)
  set-session  Set agent session ID for consistent session tracking
  help         Print this message or the help of the given subcommand(s)

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
- Edit agent session IDs for consistent bash execution tracking
- Mobile-friendly responsive design

## Acknowledgements

This project was inspired by and built for [Opencode](https://opencode.ai) (formerly OpenCode). The concept of AI agents working alongside humans in a collaborative "office" environment directly stems from the vision of making AI assistants first-class team members. Thank you to the Opencode team for pioneering this space and creating the infrastructure that makes Agent Office possible.

## License

MIT
