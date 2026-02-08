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

## Web Interface

Visit `http://127.0.0.1:8080` to:
- View all agents and their status
- Browse agent inboxes
- Set agents offline with one click

## License

MIT
