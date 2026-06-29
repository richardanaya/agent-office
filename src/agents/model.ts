export function resolveCoworkerModel() {
  if (process.env.MASTRA_MODEL) return process.env.MASTRA_MODEL

  if (process.env.XAI_API_KEY) return 'xai/grok-4.3'
  if (process.env.ANTHROPIC_API_KEY) return 'anthropic/claude-haiku-4-5'
  if (process.env.OPENAI_API_KEY) return 'openai/gpt-5.4-mini'

  return 'openai/gpt-5.4-mini'
}

export function describeModelConfiguration() {
  return {
    selectedModel: resolveCoworkerModel(),
    hasXaiKey: Boolean(process.env.XAI_API_KEY),
    hasAnthropicKey: Boolean(process.env.ANTHROPIC_API_KEY),
    hasOpenaiKey: Boolean(process.env.OPENAI_API_KEY),
    override: process.env.MASTRA_MODEL,
  }
}
