# agent-office

A virtual office in your terminal. Alice, Bob, and Carol are AI coworkers — message them, watch them think and reply, and see the office buzz in a live log.

## Setup

**Node.js 20+** required.

Install:

```bash
npm install -g agent-office
```

Set an API key for your model provider:

```bash
export ANTHROPIC_API_KEY=your-key-here
# or: export OPENAI_API_KEY=...
# or: export XAI_API_KEY=...
```

From source:

```bash
git clone git@github.com:richardanaya/agent-office.git
cd agent-office
npm install
```

## Run

```bash
agent-office
```

From a local checkout:

```bash
npm run dev
```

## The terminal experience

You land in a split-screen office:

- **Office log** (top) — live feed of what’s happening: your messages, agents thinking, replies, tool calls, coworker chatter.
- **Chat panel** (bottom) — pick a coworker, type, hit Enter.

It feels like dropping into a small team Slack channel, except your coworkers are agents who actually respond. Say hello to Alice. Ask Bob to check something with Carol. Watch the log fill in as they work.

**Tab** switches to Coworkers mode — add or remove agents for this session.

| Key | What it does |
| --- | --- |
| **↑ / ↓** | Pick who to message |
| **Enter** | Send |
| **Tab** | Chat ↔ Coworkers |
| **PgUp / PgDn** | Scroll the log |
| **Ctrl+C** | Quit |

## License

MIT © Richard Anaya