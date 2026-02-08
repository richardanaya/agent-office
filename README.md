# Agent Office

A Rust-based multi-agent system featuring a mail system and Zettelkasten-style knowledge base.

## Features

- **Multi-Agent System**: Create and manage agents with status tracking
- **Mail System**: Agents can send and receive messages via mailboxes
- **Knowledge Base**: Zettelkasten-style notes with Luhmann addressing (1, 1a, 1a1)
- **Web Interface**: HTMX-based UI for browsing agents and mail and kb

## Quick Start

```bash
# Set up database
export AGENT_OFFICE_URL="postgresql://user:pass@localhost/agent_office"
cargo install agent-office
agent-office db setup

# Create agents
agent-office agent create alice
agent-office  agent create bob

# Send mail
agent-office mail send alice bob "Hello" "Message body"

# Start web server
agent-office  -p 8080
```

## Configuration

Set `AGENT_OFFICE_URL` environment variable or use `.env` file:

```bash
AGENT_OFFICE_URL=postgresql://agent:agent123@localhost:5432/agent_office
```

## Options

```bash
agent-office --help
A graph-based data structure tool for AI agents

Usage: agent-office <COMMAND>

Commands:
  mail   
  agent  
  db     
  kb     Simplified KB commands - shared knowledge base, only Luhmann IDs
  web    Start web interface
  help   Print this message or the help of the given subcommand(s)

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
Simplified KB commands - shared knowledge base, only Luhmann IDs

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
- Browse agent inboxes
- Set agents offline with one click

## License

MIT
