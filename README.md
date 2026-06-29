# agent-office

Virtual office for Mastra agents. Coworkers message each other through a shared mailbox, wake on incoming signals, and coordinate with a human teammate from a terminal UI.

## Install

```bash
npm install agent-office
```

## Run

Set a model provider API key, then start the terminal UI:

```bash
export XAI_API_KEY=...
# or: export ANTHROPIC_API_KEY=...
# or: export OPENAI_API_KEY=...

npx agent-office
```

From a local checkout:

```bash
npm install
npm run dev      # run from source (fastest while editing)
# or
npm run start    # build dist, then run compiled CLI
```

Default model selection order:

1. `MASTRA_MODEL` override, if set
2. `xai/grok-4.3` when `XAI_API_KEY` is present
3. `anthropic/claude-haiku-4-5` when `ANTHROPIC_API_KEY` is present
4. `openai/gpt-5.4-mini` when `OPENAI_API_KEY` is present, or as fallback

Optional model override:

```bash
export MASTRA_MODEL=xai/grok-4.3
# or: export MASTRA_MODEL=anthropic/claude-haiku-4-5
# or: export MASTRA_MODEL=openai/gpt-5.4-mini
```

## Terminal UI

`agent-office` opens an Ink-based TUI where you can:

- Send messages to Alice, Bob, Carol, or runtime coworkers you add
- Watch agent tool calls, signals, and replies in the office log
- Add or remove coworkers for the current session (Tab → Coworkers)

Controls:

- **Tab** — switch between Chat and Coworkers
- **↑/↓** — select recipient or coworker
- **Enter** — send a message (Chat mode)
- **a** — add a runtime coworker (Coworkers mode)
- **d** — remove the selected coworker for this run
- **PgUp/PgDn** — scroll the office log
- **Ctrl+C** — quit

## Library API

Import the office runtime and helpers from the package:

```ts
import {
  mastra,
  sendHumanMessage,
  listHumanInbox,
  markAllHumanMessagesSeen,
  createCoworkerAgent,
  subscribeCoworkerThreads,
} from 'agent-office'

subscribeCoworkerThreads()

sendHumanMessage(
  'Alice',
  'Please ask Bob and Carol once for their best office-friendly joke. Then schedule a wake for 10 seconds, pick the funniest available joke, and send it back to me.',
)
const unread = listHumanInbox({ unreadOnly: true })
markAllHumanMessagesSeen()
```

Agents reply to the human with `send_office_message({ to: 'Human', message: '...' })`.

## How targeting works

Each coworker watches their own mailbox resource:

```ts
{ resourceId: 'agent-office', threadId: 'coworker:alice' }
```

When a message has `to: 'alice'`, `OfficeSignals` emits a notification signal into Alice's thread.

## Package layout

- `src/office/mailbox.ts` — shared office mailbox
- `src/office/tools.ts` — agent tools for messaging, scheduling, and coworker lookup
- `src/office/human.ts` — human send/inbox helpers
- `src/office/office-signals.ts` — polling signal provider for mailbox delivery
- `src/agents/coworkers.ts` — Alice, Bob, and Carol agents
- `src/mastra/index.ts` — Mastra runtime with libSQL storage
- `src/cli.tsx` — terminal UI entrypoint (`agent-office` bin)

## Configuration

Copy `.env.example` and export the variables you need in your shell (the CLI does not auto-load `.env` files):

| Variable | Required | Default | Purpose |
| --- | --- | --- | --- |
| `XAI_API_KEY` | one of three keys | — | Use xAI Grok models |
| `ANTHROPIC_API_KEY` | one of three keys | — | Use Anthropic Claude models |
| `OPENAI_API_KEY` | one of three keys | — | Use OpenAI models |
| `MASTRA_MODEL` | no | auto-detected | Force a specific model id |
| `AGENT_OFFICE_DB_URL` | no | `file:./agent-office.db` | libSQL URL for Mastra memory |
| `AGENT_OFFICE_RESOURCE_ID` | no | `agent-office` | Mastra memory resource id |

The CLI exits with a helpful error if no model provider key is set.

## Deploy

### npm (recommended)

Published as [`agent-office`](https://www.npmjs.com/package/agent-office) on npm.

```bash
npm login
npm publish
```

CI runs on push via GitHub Actions (`.github/workflows/ci.yml`). To publish from CI, add an `NPM_TOKEN` repository secret, tag a release (`git tag v0.8.0 && git push origin v0.8.0`), and the release workflow publishes with provenance.

Before publishing locally:

```bash
npm run typecheck
npm run build
npm pack --dry-run
```

### Runtime requirements

- Node.js 20+
- An interactive terminal (TTY) for the Ink UI
- Outbound HTTPS to your model provider
- Writable directory for `agent-office.db` unless `AGENT_OFFICE_DB_URL` points elsewhere

## License

MIT © Richard Anaya

## Develop

```bash
npm install
npm run typecheck
npm run dev        # source
npm run start      # build + run dist
npm run mastra:dev
```