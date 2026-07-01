export type MessageTarget = {
  to: string
  body: string
  // True when an @mention prefix picked the target (vs the pinned default).
  mentioned: boolean
}

// "@bob hey there" addresses bob directly when bob is a known target;
// anything else goes to the pinned default target unchanged.
export function parseMessageTarget(input: string, targets: string[], defaultTarget: string): MessageTarget {
  const match = /^@(\S+)\s*(.*)$/s.exec(input)
  if (match) {
    const mention = match[1]!.toLowerCase()
    const target = targets.find(name => name.toLowerCase() === mention)
    if (target) return { to: target, body: match[2]!.trim(), mentioned: true }
  }
  return { to: defaultTarget, body: input.trim(), mentioned: false }
}
