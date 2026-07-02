import * as api from './api.js'
import { agentBubble, initScene, setThinking, syncVillagers, updateBoard, updateWiki, villagerColor } from './scene.js'

const STATE_POLL_MS = 2_000
const LOG_LIMIT = 200

const $ = id => document.getElementById(id)

let chatTarget = 'All'
let coworkers = []
let questions = []
let tasks = []
let wiki = []
let selectedWikiSlug = null
let activeQuestion = null
let selectedChoices = []
const thinkingCounts = new Map()

// ── Log panel ────────────────────────────────────────────────

function log(text, kind = '') {
  const lines = $('log-lines')
  const line = document.createElement('div')
  line.className = `log-line ${kind}`.trim()
  line.textContent = text
  lines.appendChild(line)
  while (lines.children.length > LOG_LIMIT) lines.removeChild(lines.firstChild)
  lines.scrollTop = lines.scrollHeight
}

// navigator.clipboard needs a secure context (https or localhost); fall back
// to a scratch textarea for plain-http remote servers.
function copyText(text) {
  if (navigator.clipboard?.writeText) return navigator.clipboard.writeText(text)
  const scratch = document.createElement('textarea')
  scratch.value = text
  scratch.style.position = 'fixed'
  scratch.style.opacity = '0'
  document.body.appendChild(scratch)
  scratch.select()
  try {
    document.execCommand('copy')
  } finally {
    scratch.remove()
  }
  return Promise.resolve()
}

$('copy-log').addEventListener('click', () => {
  const text = [...$('log-lines').children].map(line => line.textContent).join('\n')
  copyText(text)
    .then(() => toast('Log copied! 📋'))
    .catch(() => toast('Could not copy the log 😿'))
})

let toastTimer
function toast(text) {
  const element = $('toast')
  element.textContent = text
  element.hidden = false
  clearTimeout(toastTimer)
  toastTimer = setTimeout(() => {
    element.hidden = true
  }, 3200)
}

// ── Chat ─────────────────────────────────────────────────────

function setChatTarget(name) {
  chatTarget = name
  $('chat-target').textContent = name === 'All' ? 'To ✦ All' : `To ✦ ${name}`
  $('chat-input').placeholder = name === 'All'
    ? 'Say something to the office… (click a villager to DM them)'
    : `Message ${name} privately… (click the chip to talk to everyone)`
}

$('chat-target').addEventListener('click', () => setChatTarget('All'))

$('chat-bar').addEventListener('submit', event => {
  event.preventDefault()
  const input = $('chat-input')
  const body = input.value.trim()
  if (!body) return
  if (coworkers.length === 0) {
    toast('The office is empty — hire a coworker first! 🍃')
    return
  }
  input.value = ''
  api.sendMessage(chatTarget, body).catch(error => {
    log(`Could not deliver: ${error.message}`, 'error')
    toast('Message failed 😿')
  })
})

// ── Team panel ───────────────────────────────────────────────

function renderTeam() {
  const list = $('team-list')
  list.replaceChildren()
  if (coworkers.length === 0) {
    const empty = document.createElement('li')
    empty.textContent = 'Nobody here yet — hire your first coworker below!'
    list.appendChild(empty)
    return
  }
  for (const coworker of coworkers) {
    const item = document.createElement('li')
    const swatch = document.createElement('span')
    swatch.className = 'swatch'
    swatch.style.background = villagerColor(coworker.name, coworker.appearance)
    const who = document.createElement('div')
    who.className = 'who'
    const name = document.createElement('div')
    name.className = 'name'
    name.textContent = coworker.name
    const role = document.createElement('div')
    role.className = 'role'
    role.textContent = coworker.role
    who.append(name, role)
    const rerollButton = document.createElement('button')
    rerollButton.className = 'row-action'
    rerollButton.title = `Reroll ${coworker.name}'s look`
    rerollButton.textContent = '🎲'
    rerollButton.addEventListener('click', () => {
      api.rerollAppearance(coworker.name)
        .then(() => {
          toast(`${coworker.name} got a new look! 🎲`)
          refreshState()
        })
        .catch(error => toast(error.message))
    })
    const fireButton = document.createElement('button')
    fireButton.className = 'row-action'
    fireButton.title = `Fire ${coworker.name}`
    fireButton.textContent = '❌'
    fireButton.addEventListener('click', () => {
      api.fire(coworker.name)
        .then(() => {
          log(`You fired ${coworker.name}.`, 'muted')
          refreshState()
        })
        .catch(error => toast(error.message))
    })
    item.append(swatch, who, rerollButton, fireButton)
    list.appendChild(item)
  }
}

$('hire-form').addEventListener('submit', event => {
  event.preventDefault()
  const name = $('hire-name').value.trim()
  const role = $('hire-role').value.trim()
  if (!name || !role) return
  api.hire(name, role)
    .then(({ profile }) => {
      log(`${profile.name} joined the office! 🎉`, 'chat')
      $('hire-name').value = ''
      $('hire-role').value = ''
      refreshState()
    })
    .catch(error => toast(error.message))
})

$('save-team').addEventListener('click', () => {
  const path = window.prompt('Save team to (path on the server):', 'team.json')
  if (!path) return
  api.saveTeam(path)
    .then(result => toast(`Saved ${result.coworkers.length} coworker(s) to ${result.path} 💾`))
    .catch(error => toast(error.message))
})

$('load-team').addEventListener('click', () => {
  const path = window.prompt('Load team from (path on the server):', 'team.json')
  if (!path) return
  api.loadTeam(path)
    .then(result => {
      toast(`Loaded ${result.coworkers.length} coworker(s) from ${result.path} 📂`)
      for (const skipped of result.skipped) log(`Skipped ${skipped}`, 'error')
      refreshState()
    })
    .catch(error => toast(error.message))
})

function togglePanel(id) {
  const panel = $(id)
  panel.hidden = !panel.hidden
}

$('toggle-log').addEventListener('click', () => togglePanel('log-panel'))
$('toggle-team').addEventListener('click', () => togglePanel('team-panel'))

// ── Questions ────────────────────────────────────────────────

function openQuestion(question) {
  activeQuestion = question
  selectedChoices = []
  $('question-from').textContent = `✦ ${question.from} asks`
  $('question-prompt').textContent = question.prompt
  $('question-custom').value = ''
  $('question-custom').hidden = !question.allowCustomAnswer
  const choicesBox = $('question-choices')
  choicesBox.replaceChildren()
  for (const choice of question.choices) {
    const button = document.createElement('button')
    button.type = 'button'
    button.className = 'choice'
    button.textContent = choice
    button.addEventListener('click', () => {
      if (question.mode === 'single') {
        selectedChoices = [choice]
        for (const other of choicesBox.children) other.classList.remove('selected')
        button.classList.add('selected')
      } else {
        if (selectedChoices.includes(choice)) {
          selectedChoices = selectedChoices.filter(item => item !== choice)
          button.classList.remove('selected')
        } else {
          selectedChoices.push(choice)
          button.classList.add('selected')
        }
      }
    })
    choicesBox.appendChild(button)
  }
  $('question-modal').hidden = false
}

$('question-later').addEventListener('click', () => {
  $('question-modal').hidden = true
  activeQuestion = null
})

$('question-submit').addEventListener('click', () => {
  if (!activeQuestion) return
  const customAnswer = $('question-custom').value.trim() || undefined
  if (selectedChoices.length === 0 && !customAnswer) {
    toast('Pick a choice or write an answer first!')
    return
  }
  api.answerQuestion(activeQuestion.id, { selectedChoices, customAnswer })
    .then(() => {
      log(`You answered ${activeQuestion.from}'s question ✅`, 'you')
      $('question-modal').hidden = true
      activeQuestion = null
      refreshState()
    })
    .catch(error => toast(error.message))
})

$('questions-button').addEventListener('click', () => {
  if (questions[0]) openQuestion(questions[0])
})

// ── Office board dialog ──────────────────────────────────────

const BOARD_COLUMNS = ['backlog', 'todo', 'doing', 'blocked', 'review', 'done', 'canceled']

function renderBoardModal() {
  const box = $('board-columns')
  box.replaceChildren()
  for (const column of BOARD_COLUMNS) {
    const items = tasks.filter(task => task.status === column)
    // Keep the core columns visible; edge columns only appear when used.
    if (items.length === 0 && (column === 'backlog' || column === 'canceled')) continue
    const columnBox = document.createElement('div')
    columnBox.className = 'board-column'
    const header = document.createElement('header')
    header.textContent = `${column} · ${items.length}`
    columnBox.appendChild(header)
    if (items.length === 0) {
      const empty = document.createElement('div')
      empty.className = 'empty'
      empty.textContent = 'nothing here'
      columnBox.appendChild(empty)
    }
    for (const task of items) {
      const card = document.createElement('div')
      card.className = `task-card priority-${task.priority}`
      const title = document.createElement('div')
      title.className = 'task-title'
      title.textContent = task.title
      const meta = document.createElement('div')
      meta.className = 'task-meta'
      meta.textContent = [task.assignee ? `@${task.assignee}` : null, task.priority !== 'normal' ? task.priority : null, `by ${task.createdBy}`]
        .filter(Boolean)
        .join(' · ')
      card.append(title, meta)
      if (task.description) card.title = task.description
      columnBox.appendChild(card)
    }
    box.appendChild(columnBox)
  }
}

function openBoard() {
  renderBoardModal()
  $('board-modal').hidden = false
}

$('board-close').addEventListener('click', () => {
  $('board-modal').hidden = true
})

// ── Office wiki dialog ───────────────────────────────────────

// Mirrors wikiSlug in src/office/wiki.ts so [[links]] resolve the same way.
function wikiSlugify(title) {
  return title.trim().toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-+|-+$/g, '')
}

const WIKI_LINK_PATTERN = /\[\[([^\[\]]+)\]\]/g

function wikiLinkSlugs(content) {
  return [...content.matchAll(WIKI_LINK_PATTERN)].map(match => wikiSlugify(match[1]))
}

// 'view' renders [[links]] as clickable navigation; 'edit' shows raw text.
let wikiMode = 'view'
let editingWikiSlug = null

function makeWikiLink(title) {
  const slug = wikiSlugify(title)
  const target = wiki.find(page => page.slug === slug)
  const link = document.createElement('button')
  link.type = 'button'
  link.className = `wiki-link${target ? '' : ' missing'}`
  link.title = target ? `Open "${target.title}"` : `"${title}" does not exist yet — click to create it`
  link.textContent = title
  link.addEventListener('click', () => {
    if (target) {
      selectedWikiSlug = target.slug
      renderWikiModal()
    } else {
      openWikiEditor(null, title)
    }
  })
  return link
}

function renderWikiBody(container, content) {
  container.replaceChildren()
  let last = 0
  for (const match of content.matchAll(WIKI_LINK_PATTERN)) {
    if (match.index > last) container.appendChild(document.createTextNode(content.slice(last, match.index)))
    container.appendChild(makeWikiLink(match[1].trim()))
    last = match.index + match[0].length
  }
  if (last < content.length) container.appendChild(document.createTextNode(content.slice(last)))
}

function renderWikiModal() {
  const editing = wikiMode === 'edit'
  $('wiki-editor').hidden = !editing
  $('wiki-pages').hidden = editing
  $('wiki-view-actions').hidden = editing
  if (editing) {
    $('wiki-content').hidden = true
    $('wiki-edit-delete').hidden = editingWikiSlug === null
    return
  }

  const chips = $('wiki-pages')
  chips.replaceChildren()
  if (wiki.length === 0) {
    const empty = document.createElement('div')
    empty.className = 'wiki-empty'
    empty.textContent = 'No pages yet. Coworkers write pages with their wiki tools, or press ➕ New page to start one yourself.'
    chips.appendChild(empty)
    $('wiki-content').hidden = true
    $('wiki-edit').hidden = true
    return
  }
  $('wiki-edit').hidden = false
  if (!wiki.some(page => page.slug === selectedWikiSlug)) selectedWikiSlug = wiki[0].slug
  for (const page of wiki) {
    const chip = document.createElement('button')
    chip.type = 'button'
    chip.className = `wiki-page-chip${page.slug === selectedWikiSlug ? ' selected' : ''}`
    chip.textContent = page.title
    chip.addEventListener('click', () => {
      selectedWikiSlug = page.slug
      renderWikiModal()
    })
    chips.appendChild(chip)
  }

  const selected = wiki.find(page => page.slug === selectedWikiSlug)
  $('wiki-content').hidden = false
  $('wiki-page-title').textContent = selected.title
  $('wiki-page-meta').textContent = `last edited by ${selected.updatedBy} · ${new Date(selected.updatedAt).toLocaleString()}`
  renderWikiBody($('wiki-page-body'), selected.content)

  const backlinksBox = $('wiki-backlinks')
  backlinksBox.replaceChildren()
  const backlinks = wiki.filter(page => page.slug !== selected.slug && wikiLinkSlugs(page.content).includes(selected.slug))
  if (backlinks.length > 0) {
    backlinksBox.appendChild(document.createTextNode('Linked from:'))
    for (const page of backlinks) backlinksBox.appendChild(makeWikiLink(page.title))
  }
}

function openWikiEditor(slug, prefillTitle = '') {
  const page = slug ? wiki.find(item => item.slug === slug) : undefined
  wikiMode = 'edit'
  editingWikiSlug = page ? page.slug : null
  $('wiki-edit-title').value = page ? page.title : prefillTitle
  $('wiki-edit-content').value = page ? page.content : ''
  renderWikiModal()
  ;(page || !prefillTitle ? $(page ? 'wiki-edit-content' : 'wiki-edit-title') : $('wiki-edit-content')).focus()
}

function closeWikiEditor() {
  wikiMode = 'view'
  editingWikiSlug = null
  renderWikiModal()
}

function openWiki() {
  wikiMode = 'view'
  renderWikiModal()
  $('wiki-modal').hidden = false
}

$('wiki-new').addEventListener('click', () => openWikiEditor(null))
$('wiki-edit').addEventListener('click', () => openWikiEditor(selectedWikiSlug))
$('wiki-edit-cancel').addEventListener('click', closeWikiEditor)

$('wiki-edit-save').addEventListener('click', () => {
  const title = $('wiki-edit-title').value.trim()
  const content = $('wiki-edit-content').value.trim()
  if (!title || !content) {
    toast('A page needs a title and some content!')
    return
  }
  const previousSlug = editingWikiSlug
  api.writeWikiPage(title, content)
    .then(async ({ page }) => {
      // Changing the title is a rename: drop the page under the old slug.
      if (previousSlug && previousSlug !== page.slug) await api.deleteWikiPage(previousSlug).catch(() => {})
      wiki = [...wiki.filter(item => item.slug !== page.slug && item.slug !== previousSlug), page]
        .sort((a, b) => a.title.localeCompare(b.title))
      selectedWikiSlug = page.slug
      toast(`Saved "${page.title}" 📖`)
      closeWikiEditor()
      void refreshState()
    })
    .catch(error => toast(error.message))
})

$('wiki-edit-delete').addEventListener('click', () => {
  if (!editingWikiSlug) return
  const page = wiki.find(item => item.slug === editingWikiSlug)
  if (!window.confirm(`Delete wiki page "${page?.title ?? editingWikiSlug}"?`)) return
  api.deleteWikiPage(editingWikiSlug)
    .then(({ page: deleted }) => {
      wiki = wiki.filter(item => item.slug !== deleted.slug)
      toast(`Deleted "${deleted.title}" 🗑️`)
      closeWikiEditor()
      void refreshState()
    })
    .catch(error => toast(error.message))
})

$('wiki-close').addEventListener('click', () => {
  closeWikiEditor()
  $('wiki-modal').hidden = true
})

document.addEventListener('keydown', event => {
  if (event.key !== 'Escape') return
  if (!$('wiki-modal').hidden) {
    if (wikiMode === 'edit') closeWikiEditor()
    else $('wiki-modal').hidden = true
  } else if (!$('board-modal').hidden) $('board-modal').hidden = true
  else if (!$('question-modal').hidden) {
    $('question-modal').hidden = true
    activeQuestion = null
  }
})

function renderQuestionsBadge() {
  const button = $('questions-button')
  button.hidden = questions.length === 0
  $('questions-count').textContent = String(questions.length)
  if (questions.length === 0) {
    $('question-modal').hidden = true
    activeQuestion = null
  } else if (!activeQuestion && $('question-modal').hidden) {
    openQuestion(questions[0])
  }
}

// ── State polling ────────────────────────────────────────────

async function refreshState() {
  try {
    const state = await api.getState()
    coworkers = state.coworkers
    questions = state.questions
    tasks = state.tasks
    wiki = state.wiki
    syncVillagers(state.coworkers, state.statuses)
    updateBoard(state.tasks)
    updateWiki(state.wiki)
    renderTeam()
    renderQuestionsBadge()
    if (!$('board-modal').hidden) renderBoardModal()
    if (!$('wiki-modal').hidden) renderWikiModal()
    if (coworkers.length === 0) $('team-panel').hidden = false
  } catch {
    // Connection issues are surfaced by the event stream status.
  }
}

// ── Event stream ─────────────────────────────────────────────

function bumpThinking(agent, delta) {
  const next = Math.max(0, (thinkingCounts.get(agent) ?? 0) + delta)
  thinkingCounts.set(agent, next)
  setThinking(agent, next > 0)
}

function handleAgentChunk(agent, chunk) {
  switch (chunk?.type) {
    case 'reasoning-start':
      bumpThinking(agent, 1)
      return
    case 'reasoning-end':
      bumpThinking(agent, -1)
      return
    case 'tool-call': {
      const { toolName, args = {} } = chunk.payload ?? {}
      if (toolName === 'send_office_message') {
        agentBubble(agent, args.message)
        log(`${agent} → ${args.to}: ${args.message}`, 'chat')
      } else if (toolName === 'ask_human_question') {
        log(`${agent} has a question for you: ${args.prompt}`, 'question')
        setTimeout(refreshState, 300)
      } else if (toolName === 'set_status') {
        setTimeout(refreshState, 300)
      } else if (toolName === 'create_office_task' || toolName === 'update_office_task') {
        log(`${agent} updated the office board 📌`, 'muted')
        setTimeout(refreshState, 300)
      } else if (toolName === 'write_wiki_page') {
        log(`${agent} wrote wiki page: ${args.title} 📖`, 'muted')
        setTimeout(refreshState, 300)
      } else if (toolName === 'delete_wiki_page') {
        log(`${agent} deleted wiki page: ${args.title} 📖`, 'muted')
        setTimeout(refreshState, 300)
      }
      return
    }
    case 'error': {
      const error = chunk.payload?.error
      log(`${agent} hit a snag: ${error?.responseBody ?? error?.message ?? 'unknown error'}`, 'error')
      return
    }
    default:
  }
}

function handleOfficeEvent(event) {
  switch (event.type) {
    case 'agent-event':
      handleAgentChunk(event.agent, event.chunk)
      return
    case 'human-message':
      log(`📬 ${event.message.from} → you: ${event.message.body}`, 'chat')
      return
    case 'human-message-sent':
      log(`You → ${event.to}: ${event.body}`, 'you')
      return
    case 'roster-changed':
      refreshState()
      return
    default:
  }
}

function setConnection(status) {
  $('connection-dot').className = `dot ${status}`
  $('connection-label').textContent = status === 'connected' ? 'connected' : 'reconnecting…'
}

// ── Boot ─────────────────────────────────────────────────────

// The panels, chat, and questions all work without the 3D scene, so a WebGL
// failure must not take the rest of the app down with it.
try {
  initScene({
    container: $('scene-container'),
    onVillagerClick: name => {
      setChatTarget(name)
      $('chat-input').focus()
    },
    onBoardClick: openBoard,
    onWikiClick: openWiki,
  })
} catch (error) {
  console.error('Could not start the 3D office scene:', error)
  log('3D view unavailable (WebGL failed to start) — panels still work!', 'error')
  $('log-panel').hidden = false
}

api.subscribeEvents(handleOfficeEvent, setConnection)
refreshState()
setInterval(refreshState, STATE_POLL_MS)
log('Welcome to your agent office! 🏝️', 'you')
