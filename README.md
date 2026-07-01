# agent-office

A virtual office in your terminal. Build a team of AI coworkers — message them, watch them think and reply, and see the office buzz in a live log.

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
agent-office              # starts empty, or auto-loads ./team.json if present
agent-office my-team.json # start with a specific team
```

From a local checkout:

```bash
npm run dev
```

## Build a team

The office starts empty. Press **Tab** to open the Team panel, then:

- **a** — hire a coworker (you give them a name and a role)
- **d** — fire the selected coworker
- **s** — save the current team to a JSON file
- **l** — load a team from a JSON file (replaces the current team)

A team file is plain JSON you can edit by hand:

```json
{
  "name": "Product Crew",
  "coworkers": [
    { "name": "Alice", "role": "product lead who coordinates priorities and decisions" },
    { "name": "Bob", "role": "engineer who thinks through implementation details" }
  ]
}
```

If a `team.json` exists in the directory you launch from, it loads automatically.

## The terminal experience

You land in a split-screen office:

- **Office log** (top) — live feed of what's happening: your messages, agents thinking, replies, tool calls, coworker chatter.
- **Office chat** (bottom) — type and hit Enter.

Messages go to **All** (the shared office channel) by default. Type `@name message` to DM a coworker, or pin a target with **←/→**.

When a coworker needs a decision from you, a question badge appears — press **?** to answer with arrow keys and Space, or type a custom answer on the Custom row.

### Keys

| Key | Chat | Team | Questions |
| --- | --- | --- | --- |
| **Enter** | Send | — | Submit answer |
| **@name …** | DM a coworker | — | — |
| **←/→** | Pin target | — | Switch question |
| **↑/↓** | Message history | Select coworker | Move highlight |
| **Space** | — | — | Select choice |
| **a / d** | — | Hire / fire | — |
| **s / l** | — | Save / load team | — |
| **?** | Open questions | Open questions | — |
| **Tab** | Team | Chat | Chat |
| **Esc** | Clear input | Chat | Chat |
| **PgUp / PgDn** | Scroll the log | ← same | ← same |
| **Ctrl+C** | Quit | Quit | Quit |

## License

MIT © Richard Anaya
