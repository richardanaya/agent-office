// The office: an art deco underwater habitat — a glass half-dome with gold
// ribs over a polished marble floor, drowned deco towers and neon signs in
// the water outside, bubbles rising past — with Animal Crossing-style toon
// villagers (one per coworker) that wander, bob, think, and talk in bubbles.

import * as THREE from 'three'
import { OrbitControls } from 'three/addons/controls/OrbitControls.js'

const FLOOR_RADIUS = 26
const WANDER_RADIUS = 8

const BUBBLE_FONT = '700 34px "Josefin Sans", sans-serif'
const BUBBLE_MAX_TEXT_WIDTH = 520
const BUBBLE_MAX_LINES = 4
const BUBBLE_LINE_HEIGHT = 44
const BUBBLE_TAIL = 46
const BUBBLE_PADDING = 88
// 160 canvas pixels per world unit keeps bubble text a consistent size.
const BUBBLE_PIXELS_PER_UNIT = 160
// The bubble canvas and sprite never resize (resizing a live CanvasTexture
// stretches stale frames); messages draw at their measured size inside this
// fixed canvas, anchored bottom-center, with transparent margins around them.
const BUBBLE_CANVAS_WIDTH = 640
const BUBBLE_CANVAS_HEIGHT = 352

// Bubbles stay up long enough to read: a base plus reading time by length.
function bubbleSecondsFor(text) {
  return Math.min(24, 8 + String(text).length * 0.07)
}

let scene
let camera
let renderer
let controls
let lastFrameAt = 0
let boardTexture
let boardCanvas
let boardGroup
let wikiTexture
let wikiCanvas
let wikiGroup
const villagers = new Map()
let villagerClickHandler = () => {}
let boardClickHandler = () => {}
let wikiClickHandler = () => {}

// Noble little deco humans, all derived from the seed: skin, hair, nose,
// ears, headwear, evening attire, and a finishing accessory.
// Kept very bright: the dim underwater toon lighting darkens everything a
// step, so even the deepest tone here stays light.
const SKIN_TONES = ['#fff5ea', '#ffeedd', '#ffe7cf', '#fde0c0', '#f8d6b0', '#f2cca2', '#eec096']
const HAIR_COLORS = ['#1d1a17', '#3c2a1e', '#5b3c26', '#7a5230', '#b3843c', '#8f4c2e', '#b7b3ac', '#e6ddc8']
const HAIR_STYLES = ['sleek', 'waves', 'bob', 'bun', 'pompadour', 'crop']
const NOSE_SHAPES = ['button', 'pointed', 'long', 'wide']
const EAR_SHAPES = ['petite', 'round', 'tall']
const HEADWEAR = ['none', 'none', 'tophat', 'cloche', 'boater', 'headband']
const OUTFIT_COLORS = ['#22222c', '#2c3e63', '#1f5e4e', '#6e2f3d', '#5d4a78', '#b8912c', '#f0e6d2', '#3c6e71']
const ACCESSORIES = ['none', 'scarf', 'bowtie', 'pendant', 'buttons', 'badge']
const ACCESSORY_COLORS = ['#d4af37', '#f2ede2', '#2f8f6b', '#b03a48', '#3b5fa0', '#c9ccd4']

function lighten(hex, amount) {
  const value = Number.parseInt(hex.slice(1), 16)
  const mix = channel => Math.round(channel + (255 - channel) * amount)
  const r = mix((value >> 16) & 255)
  const g = mix((value >> 8) & 255)
  const b = mix(value & 255)
  return `#${((r << 16) | (g << 8) | b).toString(16).padStart(6, '0')}`
}

function hashString(value) {
  let hash = 0
  for (const char of value) hash = (hash * 31 + char.codePointAt(0)) >>> 0
  return hash
}

function mulberry32(seed) {
  return () => {
    seed = (seed + 0x6d2b79f5) | 0
    let t = Math.imul(seed ^ (seed >>> 15), 1 | seed)
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296
  }
}

// Appearance is deterministic from name + the profile's appearance seed (kept
// server-side and saved into team files), so a villager looks the same in
// every client and across sessions until the dice are rolled.
function appearanceFor(name, seed) {
  const random = mulberry32(hashString(`${name.toLowerCase()}:${seed ?? 0}`))
  const pick = list => list[Math.floor(random() * list.length)]
  return {
    skin: pick(SKIN_TONES),
    hair: pick(HAIR_COLORS),
    hairStyle: pick(HAIR_STYLES),
    nose: pick(NOSE_SHAPES),
    ears: pick(EAR_SHAPES),
    headwear: pick(HEADWEAR),
    outfit: pick(OUTFIT_COLORS),
    accessory: pick(ACCESSORIES),
    accessoryColor: pick(ACCESSORY_COLORS),
    bodyScale: 0.88 + random() * 0.28,
    headScale: 0.9 + random() * 0.22,
    // Overall height: some villagers are shorties, some tower a bit.
    height: 0.8 + random() * 0.45,
  }
}

function roundedRect(ctx, x, y, width, height, radius) {
  ctx.beginPath()
  ctx.moveTo(x + radius, y)
  ctx.arcTo(x + width, y, x + width, y + height, radius)
  ctx.arcTo(x + width, y + height, x, y + height, radius)
  ctx.arcTo(x, y + height, x, y, radius)
  ctx.arcTo(x, y, x + width, y, radius)
  ctx.closePath()
}

function makeCanvasSprite(width, height, worldWidth) {
  const canvas = document.createElement('canvas')
  canvas.width = width
  canvas.height = height
  const texture = new THREE.CanvasTexture(canvas)
  texture.colorSpace = THREE.SRGBColorSpace
  const sprite = new THREE.Sprite(new THREE.SpriteMaterial({ map: texture, transparent: true, depthTest: false }))
  sprite.scale.set(worldWidth, worldWidth * (height / width), 1)
  return { canvas, ctx: canvas.getContext('2d'), texture, sprite }
}

// Shrink the font until the text fits (down to a floor), then ellipsize.
// Sets ctx.font as a side effect so the caller can fillText directly.
function fitText(ctx, text, maxWidth, weight, baseSize, minSize) {
  for (let size = baseSize; size >= minSize; size -= 2) {
    ctx.font = `${weight} ${size}px "Josefin Sans", sans-serif`
    if (ctx.measureText(text).width <= maxWidth) return text
  }
  let trimmed = text
  while (trimmed && ctx.measureText(`${trimmed}…`).width > maxWidth) trimmed = trimmed.slice(0, -1)
  return `${trimmed.trimEnd()}…`
}

// The tag canvas is fixed-size; the card inside is drawn at content height
// from the top, so a wrapped status extends the card without any rescaling.
function drawNameTag(tag, name, color, status) {
  const { ctx, canvas, texture } = tag
  const maxTextWidth = canvas.width - 72
  ctx.clearRect(0, 0, canvas.width, canvas.height)

  ctx.font = '500 30px "Josefin Sans", sans-serif'
  const statusLines = status ? wrapBubbleLines(ctx, status, maxTextWidth, 2) : []
  const cardHeight = 88 + statusLines.length * 34

  ctx.fillStyle = 'rgba(22, 22, 30, 0.92)'
  ctx.strokeStyle = '#d4af37'
  ctx.lineWidth = 8
  roundedRect(ctx, 8, 8, canvas.width - 16, cardHeight - 16, 22)
  ctx.fill()
  ctx.stroke()

  // Villager-colored deco diamonds flank the name.
  ctx.fillStyle = color
  for (const x of [42, canvas.width - 42]) {
    ctx.save()
    ctx.translate(x, 48)
    ctx.rotate(Math.PI / 4)
    ctx.fillRect(-8, -8, 16, 16)
    ctx.restore()
  }

  ctx.fillStyle = '#f4e8d0'
  ctx.textAlign = 'center'
  ctx.textBaseline = 'middle'
  const fittedName = fitText(ctx, name, maxTextWidth - 60, '700', statusLines.length > 0 ? 44 : 48, 30)
  ctx.fillText(fittedName, canvas.width / 2, 48)

  ctx.fillStyle = '#c7b58c'
  ctx.font = '500 30px "Josefin Sans", sans-serif'
  statusLines.forEach((line, index) => {
    ctx.fillText(line, canvas.width / 2, 92 + index * 34)
  })
  texture.needsUpdate = true
}

function wrapBubbleLines(ctx, text, maxWidth, maxLines) {
  const words = String(text).trim().split(/\s+/).filter(Boolean)
  const lines = []
  let current = ''
  let truncated = false
  for (const word of words) {
    const attempt = current ? `${current} ${word}` : word
    if (ctx.measureText(attempt).width <= maxWidth || !current) {
      current = attempt
      continue
    }
    if (lines.length === maxLines - 1) {
      truncated = true
      break
    }
    lines.push(current)
    current = word
  }
  if (current) lines.push(current)
  if (lines.length > maxLines) {
    lines.length = maxLines
    truncated = true
  }
  // Ellipsize any line that still overflows (unbroken long words, last line).
  return lines.map((line, index) => {
    const needsEllipsis = (truncated && index === lines.length - 1) || ctx.measureText(line).width > maxWidth
    if (!needsEllipsis) return line
    let trimmed = line
    while (trimmed && ctx.measureText(`${trimmed}…`).width > maxWidth) trimmed = trimmed.slice(0, -1)
    return `${trimmed.trimEnd()}…`
  })
}

// Measure the text, then draw a bubble sized to it inside the fixed canvas:
// short quips get small bubbles, long messages get room to breathe, and the
// sprite itself never changes size.
function drawBubble(bubble, text) {
  const { canvas, ctx, texture } = bubble
  ctx.clearRect(0, 0, canvas.width, canvas.height)
  ctx.font = BUBBLE_FONT
  let lines = wrapBubbleLines(ctx, text, BUBBLE_MAX_TEXT_WIDTH, BUBBLE_MAX_LINES)
  if (lines.length === 0) lines = ['…']
  const widest = Math.max(120, ...lines.map(line => ctx.measureText(line).width))
  const bubbleWidth = Math.ceil(Math.min(widest, BUBBLE_MAX_TEXT_WIDTH) + BUBBLE_PADDING)
  const bodyHeight = lines.length * BUBBLE_LINE_HEIGHT + 56
  const centerX = canvas.width / 2
  const left = centerX - bubbleWidth / 2
  const bodyBottom = canvas.height - BUBBLE_TAIL
  const top = bodyBottom - bodyHeight

  ctx.fillStyle = 'rgba(244, 232, 208, 0.97)'
  ctx.strokeStyle = '#b8912c'
  ctx.lineWidth = 8
  roundedRect(ctx, left, top, bubbleWidth, bodyHeight, 24)
  ctx.fill()
  ctx.stroke()
  // Tail below the body: fill a wedge over the border to open the gap, then
  // outline only its two sides so it reads as one shape with the tip down.
  ctx.beginPath()
  ctx.moveTo(centerX - 24, bodyBottom - 12)
  ctx.lineTo(centerX, canvas.height - 6)
  ctx.lineTo(centerX + 24, bodyBottom - 12)
  ctx.closePath()
  ctx.fill()
  ctx.beginPath()
  ctx.moveTo(centerX - 24, bodyBottom - 5)
  ctx.lineTo(centerX, canvas.height - 6)
  ctx.lineTo(centerX + 24, bodyBottom - 5)
  ctx.stroke()

  ctx.fillStyle = '#1d1d28'
  ctx.textAlign = 'center'
  ctx.textBaseline = 'middle'
  const centerY = top + bodyHeight / 2
  lines.forEach((line, index) => {
    ctx.fillText(line, centerX, centerY + (index - (lines.length - 1) / 2) * BUBBLE_LINE_HEIGHT)
  })
  texture.needsUpdate = true
}

// One shared thought-cloud texture for every villager's thinking indicator:
// fixed size and drawn once, so it never depends on text measurement.
let thinkingMaterial = null
function getThinkingMaterial() {
  if (thinkingMaterial) return thinkingMaterial
  const canvas = document.createElement('canvas')
  canvas.width = 192
  canvas.height = 160
  const ctx = canvas.getContext('2d')
  ctx.fillStyle = 'rgba(244, 232, 208, 0.97)'
  ctx.strokeStyle = '#b8912c'
  ctx.lineWidth = 6
  roundedRect(ctx, 6, 6, 180, 96, 40)
  ctx.fill()
  ctx.stroke()
  for (const [x, y, radius] of [[58, 122, 11], [40, 146, 6]]) {
    ctx.beginPath()
    ctx.arc(x, y, radius, 0, Math.PI * 2)
    ctx.fill()
    ctx.stroke()
  }
  ctx.fillStyle = '#9c8127'
  for (const x of [64, 96, 128]) {
    ctx.beginPath()
    ctx.arc(x, 54, 8, 0, Math.PI * 2)
    ctx.fill()
  }
  const texture = new THREE.CanvasTexture(canvas)
  texture.colorSpace = THREE.SRGBColorSpace
  thinkingMaterial = new THREE.SpriteMaterial({ map: texture, transparent: true, depthTest: false })
  return thinkingMaterial
}

// Polished marble checker with thin gold gridlines and corner studs.
function makeFloorTexture() {
  const canvas = document.createElement('canvas')
  canvas.width = 256
  canvas.height = 256
  const ctx = canvas.getContext('2d')
  const tile = 128
  for (let y = 0; y < 2; y++) {
    for (let x = 0; x < 2; x++) {
      ctx.fillStyle = (x + y) % 2 === 0 ? '#25252f' : '#2d2d39'
      ctx.fillRect(x * tile, y * tile, tile, tile)
    }
  }
  ctx.strokeStyle = 'rgba(212, 175, 55, 0.4)'
  ctx.lineWidth = 3
  for (const offset of [0, tile, 256]) {
    ctx.beginPath()
    ctx.moveTo(offset, 0)
    ctx.lineTo(offset, 256)
    ctx.moveTo(0, offset)
    ctx.lineTo(256, offset)
    ctx.stroke()
  }
  ctx.fillStyle = 'rgba(212, 175, 55, 0.55)'
  for (const x of [0, tile, 256]) {
    for (const y of [0, tile, 256]) {
      ctx.save()
      ctx.translate(x, y)
      ctx.rotate(Math.PI / 4)
      ctx.fillRect(-6, -6, 12, 12)
      ctx.restore()
    }
  }
  const texture = new THREE.CanvasTexture(canvas)
  texture.colorSpace = THREE.SRGBColorSpace
  texture.wrapS = THREE.RepeatWrapping
  texture.wrapT = THREE.RepeatWrapping
  texture.repeat.set(6, 6)
  return texture
}

function toon(color) {
  return new THREE.MeshToonMaterial({ color })
}

// Potted palm in a gold-banded lacquer planter — the deco office plant.
function addPalm(x, z, scale = 1) {
  const palm = new THREE.Group()
  const planter = new THREE.Mesh(new THREE.CylinderGeometry(0.5 * scale, 0.62 * scale, 0.62 * scale, 10), toon('#1f1f2a'))
  planter.position.y = 0.31 * scale
  planter.castShadow = true
  palm.add(planter)
  const band = new THREE.Mesh(new THREE.CylinderGeometry(0.53 * scale, 0.53 * scale, 0.1 * scale, 10), toon('#d4af37'))
  band.position.y = 0.55 * scale
  palm.add(band)
  const trunk = new THREE.Mesh(new THREE.CylinderGeometry(0.1 * scale, 0.16 * scale, 1.7 * scale, 8), toon('#6e5136'))
  trunk.position.y = 1.4 * scale
  trunk.castShadow = true
  palm.add(trunk)
  const frondMaterial = toon('#2f6b4f')
  const fronds = 6
  for (let index = 0; index < fronds; index++) {
    const angle = (index / fronds) * Math.PI * 2
    const frond = new THREE.Mesh(new THREE.SphereGeometry(0.55 * scale, 10, 8), frondMaterial)
    frond.scale.set(1.6, 0.22, 0.4)
    frond.position.set(Math.cos(angle) * 0.62 * scale, 2.32 * scale, Math.sin(angle) * 0.62 * scale)
    frond.rotation.y = -angle
    frond.rotation.z = 0.42
    frond.castShadow = true
    palm.add(frond)
  }
  const crown = new THREE.Mesh(new THREE.SphereGeometry(0.2 * scale, 10, 8), frondMaterial)
  crown.position.y = 2.42 * scale
  palm.add(crown)
  palm.position.set(x, 0, z)
  scene.add(palm)
}

// Low gold-trimmed planter with an emerald hedge ball.
function addPlanter(x, z) {
  const planter = new THREE.Group()
  const pot = new THREE.Mesh(new THREE.CylinderGeometry(0.2, 0.26, 0.26, 8), toon('#1f1f2a'))
  pot.position.y = 0.13
  planter.add(pot)
  const rim = new THREE.Mesh(new THREE.CylinderGeometry(0.22, 0.22, 0.05, 8), toon('#d4af37'))
  rim.position.y = 0.24
  planter.add(rim)
  const bush = new THREE.Mesh(new THREE.SphereGeometry(0.2, 10, 8), toon('#2f6b4f'))
  bush.position.y = 0.42
  planter.add(bush)
  planter.position.set(x, 0, z)
  scene.add(planter)
}

// Gold trim, stepped skyscraper crown, and a finial — the deco treatment
// shared by the board and wiki stands.
function addDecoCrown(group, width, height, y) {
  const gold = toon('#d4af37')
  const trim = new THREE.Mesh(new THREE.BoxGeometry(width + 0.18, height + 0.18, 0.06), gold)
  trim.position.set(0, y, -0.05)
  group.add(trim)
  let stepY = y + height / 2 + 0.09
  for (const [stepWidth, stepHeight] of [[width * 0.52, 0.12], [width * 0.28, 0.1]]) {
    const step = new THREE.Mesh(new THREE.BoxGeometry(stepWidth, stepHeight, 0.16), gold)
    stepY += stepHeight / 2
    step.position.set(0, stepY, 0)
    group.add(step)
    stepY += stepHeight / 2
  }
  const finial = new THREE.Mesh(new THREE.SphereGeometry(0.08, 10, 10), gold)
  finial.position.set(0, stepY + 0.07, 0)
  group.add(finial)
}

function addKanbanBoard() {
  const board = new THREE.Group()
  const legMaterial = toon('#23232e')
  for (const side of [-1.3, 1.3]) {
    const leg = new THREE.Mesh(new THREE.CylinderGeometry(0.09, 0.09, 2.2, 8), legMaterial)
    leg.position.set(side, 1.1, 0)
    leg.castShadow = true
    board.add(leg)
  }
  const frame = new THREE.Mesh(new THREE.BoxGeometry(3.2, 1.9, 0.14), toon('#2a2a36'))
  frame.position.y = 2.1
  frame.castShadow = true
  board.add(frame)
  addDecoCrown(board, 3.2, 1.9, 2.1)

  boardCanvas = document.createElement('canvas')
  boardCanvas.width = 512
  boardCanvas.height = 304
  boardTexture = new THREE.CanvasTexture(boardCanvas)
  boardTexture.colorSpace = THREE.SRGBColorSpace
  const face = new THREE.Mesh(
    new THREE.PlaneGeometry(2.95, 1.68),
    new THREE.MeshBasicMaterial({ map: boardTexture }),
  )
  face.position.set(0, 2.1, 0.078)
  board.add(face)

  board.position.set(-6.5, 0, -6.5)
  board.rotation.y = Math.PI / 5
  board.traverse(object => {
    object.userData.isBoard = true
  })
  boardGroup = board
  scene.add(board)
  updateBoard([])
}

function addWikiStand() {
  const stand = new THREE.Group()
  const legMaterial = toon('#23232e')
  for (const side of [-1.1, 1.1]) {
    const leg = new THREE.Mesh(new THREE.CylinderGeometry(0.09, 0.09, 2.0, 8), legMaterial)
    leg.position.set(side, 1.0, 0)
    leg.castShadow = true
    stand.add(leg)
  }
  const frame = new THREE.Mesh(new THREE.BoxGeometry(2.8, 1.8, 0.14), toon('#2a2a36'))
  frame.position.y = 1.95
  frame.castShadow = true
  stand.add(frame)
  addDecoCrown(stand, 2.8, 1.8, 1.95)

  wikiCanvas = document.createElement('canvas')
  wikiCanvas.width = 512
  wikiCanvas.height = 320
  wikiTexture = new THREE.CanvasTexture(wikiCanvas)
  wikiTexture.colorSpace = THREE.SRGBColorSpace
  const face = new THREE.Mesh(
    new THREE.PlaneGeometry(2.56, 1.6),
    new THREE.MeshBasicMaterial({ map: wikiTexture }),
  )
  face.position.set(0, 1.95, 0.078)
  stand.add(face)

  stand.position.set(6.5, 0, -6.5)
  stand.rotation.y = -Math.PI / 5
  stand.traverse(object => {
    object.userData.isWiki = true
  })
  wikiGroup = stand
  scene.add(stand)
  updateWiki([])
}

export function updateWiki(pages) {
  if (!wikiCanvas) return
  const ctx = wikiCanvas.getContext('2d')
  ctx.fillStyle = '#1c1c26'
  ctx.fillRect(0, 0, wikiCanvas.width, wikiCanvas.height)
  ctx.strokeStyle = '#d4af37'
  ctx.lineWidth = 4
  ctx.strokeRect(10, 10, wikiCanvas.width - 20, wikiCanvas.height - 20)
  ctx.fillStyle = '#d4af37'
  ctx.textAlign = 'center'
  ctx.font = '400 42px "Poiret One", "Josefin Sans", sans-serif'
  ctx.fillText('✦ OFFICE WIKI ✦', wikiCanvas.width / 2, 58)
  ctx.fillStyle = '#f4e8d0'
  ctx.font = '600 28px "Josefin Sans", sans-serif'
  if (pages.length === 0) {
    ctx.fillText('no pages yet', wikiCanvas.width / 2, 120)
  } else {
    ctx.fillText(`${pages.length} page${pages.length === 1 ? '' : 's'}`, wikiCanvas.width / 2, 104)
    ctx.textAlign = 'left'
    for (const [index, page] of pages.slice(0, 4).entries()) {
      let title = `◆ ${page.title}`
      while (title.length > 3 && ctx.measureText(title).width > wikiCanvas.width - 72) title = title.slice(0, -1)
      ctx.fillText(title, 44, 152 + index * 38)
    }
  }
  wikiTexture.needsUpdate = true
}

export function updateBoard(tasks) {
  if (!boardCanvas) return
  const ctx = boardCanvas.getContext('2d')
  ctx.fillStyle = '#1c1c26'
  ctx.fillRect(0, 0, boardCanvas.width, boardCanvas.height)
  ctx.strokeStyle = '#d4af37'
  ctx.lineWidth = 4
  ctx.strokeRect(10, 10, boardCanvas.width - 20, boardCanvas.height - 20)
  ctx.fillStyle = '#d4af37'
  ctx.textAlign = 'center'
  ctx.font = '400 42px "Poiret One", "Josefin Sans", sans-serif'
  ctx.fillText('✦ OFFICE BOARD ✦', boardCanvas.width / 2, 58)
  ctx.fillStyle = '#f4e8d0'
  ctx.font = '600 30px "Josefin Sans", sans-serif'
  const columns = ['todo', 'doing', 'blocked', 'review', 'done']
  columns.forEach((column, index) => {
    const count = tasks.filter(task => task.status === column).length
    ctx.fillText(`${column} · ${count}`, boardCanvas.width / 2, 112 + index * 38)
  })
  boardTexture.needsUpdate = true
}

// A gold-and-champagne sunburst medallion at the center of the lawn.
function makePlazaTexture() {
  const canvas = document.createElement('canvas')
  canvas.width = 512
  canvas.height = 512
  const ctx = canvas.getContext('2d')
  ctx.fillStyle = '#e9dcba'
  ctx.beginPath()
  ctx.arc(256, 256, 256, 0, Math.PI * 2)
  ctx.fill()
  const rays = 24
  ctx.fillStyle = '#d2bd85'
  for (let index = 0; index < rays; index += 2) {
    ctx.beginPath()
    ctx.moveTo(256, 256)
    ctx.arc(256, 256, 256, (index / rays) * Math.PI * 2, ((index + 1) / rays) * Math.PI * 2)
    ctx.closePath()
    ctx.fill()
  }
  ctx.strokeStyle = '#a8842e'
  for (const [radius, width] of [[244, 8], [150, 3], [64, 3]]) {
    ctx.lineWidth = width
    ctx.beginPath()
    ctx.arc(256, 256, radius, 0, Math.PI * 2)
    ctx.stroke()
  }
  ctx.fillStyle = '#a8842e'
  ctx.beginPath()
  ctx.arc(256, 256, 26, 0, Math.PI * 2)
  ctx.fill()
  const texture = new THREE.CanvasTexture(canvas)
  texture.colorSpace = THREE.SRGBColorSpace
  return texture
}

function addLampPost(x, z) {
  const lamp = new THREE.Group()
  const pole = new THREE.Mesh(new THREE.CylinderGeometry(0.05, 0.08, 2.4, 8), toon('#23232e'))
  pole.position.y = 1.2
  pole.castShadow = true
  lamp.add(pole)
  const collar = new THREE.Mesh(new THREE.CylinderGeometry(0.1, 0.1, 0.08, 8), toon('#d4af37'))
  collar.position.y = 2.36
  lamp.add(collar)
  const globe = new THREE.Mesh(new THREE.SphereGeometry(0.17, 12, 10), new THREE.MeshBasicMaterial({ color: '#ffe9a8' }))
  globe.position.y = 2.55
  lamp.add(globe)
  const tip = new THREE.Mesh(new THREE.ConeGeometry(0.06, 0.14, 8), toon('#d4af37'))
  tip.position.y = 2.78
  lamp.add(tip)
  lamp.position.set(x, 0, z)
  scene.add(lamp)
}

// Neon signage that cuts through the water (fog-exempt so it glows at
// distance); materials collected for the flicker animation.
const neonMaterials = []
const NEON_SIGNS = [
  ['AGENT OFFICE', '#4ef0e0'],
  ['OPEN 24H', '#ffc44e'],
  ['BIG IDEAS', '#ff5ec4'],
  ['MAIL ROOM', '#4ef0e0'],
  ['WIKI & CO.', '#ff6a5e'],
  ['THINK!', '#7ea8ff'],
]

function makeNeonTexture(text, color) {
  const canvas = document.createElement('canvas')
  canvas.width = 512
  canvas.height = 128
  const ctx = canvas.getContext('2d')
  ctx.textAlign = 'center'
  ctx.textBaseline = 'middle'
  ctx.font = '400 64px "Poiret One", "Josefin Sans", sans-serif'
  ctx.shadowColor = color
  ctx.fillStyle = color
  for (const blur of [30, 18, 8]) {
    ctx.shadowBlur = blur
    ctx.fillText(text, 256, 66)
  }
  ctx.shadowBlur = 0
  ctx.fillStyle = '#f6ffff'
  ctx.fillText(text, 256, 66)
  const texture = new THREE.CanvasTexture(canvas)
  texture.colorSpace = THREE.SRGBColorSpace
  return texture
}

// A ring of dark deco towers with lit windows on the horizon.
function makeWindowTexture() {
  const canvas = document.createElement('canvas')
  canvas.width = 64
  canvas.height = 128
  const ctx = canvas.getContext('2d')
  ctx.fillStyle = '#161624'
  ctx.fillRect(0, 0, 64, 128)
  for (let y = 8; y < 120; y += 14) {
    for (let x = 8; x < 56; x += 12) {
      ctx.fillStyle = Math.random() < 0.55 ? '#f2cd7c' : '#232336'
      ctx.fillRect(x, y, 6, 8)
    }
  }
  const texture = new THREE.CanvasTexture(canvas)
  texture.colorSpace = THREE.SRGBColorSpace
  return texture
}

function addSkyline() {
  const count = 22
  let signIndex = 0
  for (let index = 0; index < count; index++) {
    const angle = (index / count) * Math.PI * 2 + 0.13
    const distance = 46 + (index % 4) * 5
    const width = 3.5 + (index % 3) * 1.6
    const height = 10 + ((index * 7) % 15)
    const tower = new THREE.Group()
    const body = new THREE.Mesh(
      new THREE.BoxGeometry(width, height, width * 0.8),
      new THREE.MeshBasicMaterial({ map: makeWindowTexture() }),
    )
    body.position.y = height / 2
    tower.add(body)
    if (index % 2 === 0) {
      const crown = new THREE.Mesh(new THREE.BoxGeometry(width * 0.55, 1.6, width * 0.45), new THREE.MeshBasicMaterial({ color: '#122029' }))
      crown.position.y = height + 0.8
      tower.add(crown)
      const spire = new THREE.Mesh(new THREE.CylinderGeometry(0.06, 0.06, 2.2, 6), new THREE.MeshBasicMaterial({ color: '#d4af37' }))
      spire.position.y = height + 2.6
      tower.add(spire)
    }
    // Every few towers wears a neon sign facing the dome.
    if (index % 4 === 1) {
      const [text, color] = NEON_SIGNS[signIndex % NEON_SIGNS.length]
      signIndex++
      const material = new THREE.MeshBasicMaterial({ map: makeNeonTexture(text, color), transparent: true, fog: false, depthWrite: false })
      material.userData.phase = signIndex * 2.3
      neonMaterials.push(material)
      const sign = new THREE.Mesh(new THREE.PlaneGeometry(width * 1.15, width * 0.29), material)
      sign.position.set(0, height * 0.62, width * 0.4 + 0.2)
      tower.add(sign)
    }
    // Towers rise from the seabed below the habitat floor.
    tower.position.set(Math.cos(angle) * distance, -1.6, Math.sin(angle) * distance)
    tower.lookAt(0, -1.6, 0)
    scene.add(tower)
  }
}

// Bubbles drift up through the water outside the dome.
let bubbleGeometry = null
let bubbleSpeeds = null

function addBubbles() {
  const count = 260
  const positions = new Float32Array(count * 3)
  bubbleSpeeds = new Float32Array(count)
  for (let index = 0; index < count; index++) {
    const theta = Math.random() * Math.PI * 2
    const distance = 31 + Math.random() * 38
    positions[index * 3] = Math.cos(theta) * distance
    positions[index * 3 + 1] = Math.random() * 34 - 1.5
    positions[index * 3 + 2] = Math.sin(theta) * distance
    bubbleSpeeds[index] = 0.7 + Math.random() * 1.4
  }
  bubbleGeometry = new THREE.BufferGeometry()
  bubbleGeometry.setAttribute('position', new THREE.Float32BufferAttribute(positions, 3))
  const bubbles = new THREE.Points(
    bubbleGeometry,
    new THREE.PointsMaterial({ color: '#cfeef8', size: 0.4, sizeAttenuation: true, transparent: true, opacity: 0.55 }),
  )
  scene.add(bubbles)
}

// The habitat itself: a barely-there glass hemisphere with gold deco ribs.
function addGlassDome() {
  const radius = FLOOR_RADIUS + 2
  const glass = new THREE.Mesh(
    new THREE.SphereGeometry(radius, 48, 24, 0, Math.PI * 2, 0, Math.PI / 2),
    new THREE.MeshPhongMaterial({ color: '#9fdcec', transparent: true, opacity: 0.1, side: THREE.DoubleSide, depthWrite: false, shininess: 80 }),
  )
  scene.add(glass)

  const gold = toon('#c9a227')
  for (let index = 0; index < 6; index++) {
    const rib = new THREE.Mesh(new THREE.TorusGeometry(radius, 0.14, 8, 64, Math.PI), gold)
    rib.rotation.y = (index / 6) * Math.PI
    scene.add(rib)
  }
  for (const latitude of [0.38, 0.72]) {
    const y = radius * latitude
    const ring = new THREE.Mesh(new THREE.TorusGeometry(Math.sqrt(radius * radius - y * y), 0.12, 8, 72), gold)
    ring.rotation.x = Math.PI / 2
    ring.position.y = y
    scene.add(ring)
  }
  const collar = new THREE.Mesh(new THREE.TorusGeometry(radius, 0.22, 10, 96), gold)
  collar.rotation.x = Math.PI / 2
  collar.position.y = 0.05
  scene.add(collar)
  const beacon = new THREE.Mesh(new THREE.SphereGeometry(0.5, 12, 10), new THREE.MeshBasicMaterial({ color: '#ffe9a8' }))
  beacon.position.y = radius + 0.3
  scene.add(beacon)
  const beaconBase = new THREE.Mesh(new THREE.CylinderGeometry(0.3, 0.42, 0.5, 10), gold)
  beaconBase.position.y = radius - 0.1
  scene.add(beaconBase)
}

// Deco hairdos: a base cap plus style-specific volumes.
function addHair(head, appearance) {
  const material = toon(appearance.hair)
  const cap = new THREE.Mesh(new THREE.SphereGeometry(0.55, 20, 12, 0, Math.PI * 2, 0, Math.PI * 0.5), material)
  cap.position.y = 0.06
  // Tilt back so the front rim sits above the brows (a hairline) while the
  // back drops to cover the nape — keeps hair out of the eyes on every style.
  cap.rotation.x = -0.32
  head.add(cap)
  switch (appearance.hairStyle) {
    case 'waves':
      // Finger waves hugging the head along the hairline, above the brows.
      for (const x of [-0.32, -0.16, 0, 0.16, 0.32]) {
        const wave = new THREE.Mesh(new THREE.SphereGeometry(0.09, 8, 8), material)
        wave.position.set(x, 0.26, Math.sqrt(Math.max(0.04, 0.24 - x * x)))
        head.add(wave)
      }
      break
    case 'bob':
      for (const side of [-1, 1]) {
        const curtain = new THREE.Mesh(new THREE.SphereGeometry(0.3, 12, 10), material)
        curtain.scale.set(0.5, 1.1, 0.75)
        curtain.position.set(side * 0.42, -0.05, 0.02)
        head.add(curtain)
      }
      break
    case 'bun': {
      const bun = new THREE.Mesh(new THREE.SphereGeometry(0.18, 12, 10), material)
      bun.position.set(0, 0.36, -0.42)
      head.add(bun)
      break
    }
    case 'pompadour': {
      const pomp = new THREE.Mesh(new THREE.SphereGeometry(0.28, 14, 12), material)
      pomp.scale.set(1, 0.75, 1)
      pomp.position.set(0, 0.5, 0.16)
      head.add(pomp)
      break
    }
    case 'crop':
      cap.scale.set(1, 0.72, 1)
      break
    default:
    // 'sleek' is the plain cap.
  }
}

function addNose(head, appearance) {
  const material = toon(lighten(appearance.skin, 0.1))
  switch (appearance.nose) {
    case 'pointed': {
      const nose = new THREE.Mesh(new THREE.ConeGeometry(0.06, 0.18, 8), material)
      nose.rotation.x = Math.PI / 2
      nose.position.set(0, -0.08, 0.54)
      head.add(nose)
      break
    }
    case 'long': {
      const nose = new THREE.Mesh(new THREE.SphereGeometry(0.07, 10, 8), material)
      nose.scale.set(0.7, 1, 1.6)
      nose.position.set(0, -0.1, 0.5)
      head.add(nose)
      break
    }
    case 'wide': {
      const nose = new THREE.Mesh(new THREE.SphereGeometry(0.07, 10, 8), material)
      nose.scale.set(1.5, 0.8, 0.9)
      nose.position.set(0, -0.1, 0.48)
      head.add(nose)
      break
    }
    default: {
      const nose = new THREE.Mesh(new THREE.SphereGeometry(0.07, 10, 8), material)
      nose.position.set(0, -0.08, 0.5)
      head.add(nose)
    }
  }
}

function addEars(head, appearance) {
  const material = toon(appearance.skin)
  for (const side of [-1, 1]) {
    const ear = new THREE.Mesh(new THREE.SphereGeometry(appearance.ears === 'petite' ? 0.07 : 0.1, 8, 8), material)
    if (appearance.ears === 'tall') ear.scale.y = 1.4
    ear.position.set(side * 0.5, 0, 0)
    head.add(ear)
  }
}

// Deco millinery: top hats, cloches, boaters, feathered headbands.
function addHeadwear(head, appearance) {
  const accent = toon(appearance.accessoryColor)
  const gold = toon('#d4af37')
  switch (appearance.headwear) {
    case 'tophat': {
      const felt = toon('#20202a')
      const brim = new THREE.Mesh(new THREE.CylinderGeometry(0.5, 0.5, 0.05, 16), felt)
      brim.position.y = 0.5
      head.add(brim)
      const crown = new THREE.Mesh(new THREE.CylinderGeometry(0.3, 0.32, 0.52, 16), felt)
      crown.position.y = 0.78
      head.add(crown)
      const band = new THREE.Mesh(new THREE.CylinderGeometry(0.325, 0.335, 0.1, 16), gold)
      band.position.y = 0.58
      head.add(band)
      break
    }
    case 'cloche': {
      // Tilted back like the hair cap so the low bell clears the eyes.
      const bell = new THREE.Mesh(new THREE.SphereGeometry(0.58, 20, 12, 0, Math.PI * 2, 0, Math.PI * 0.58), accent)
      bell.position.y = 0.06
      bell.rotation.x = -0.3
      head.add(bell)
      const band = new THREE.Mesh(new THREE.TorusGeometry(0.52, 0.035, 8, 24), gold)
      band.rotation.x = Math.PI / 2 - 0.3
      band.position.y = 0.08
      head.add(band)
      break
    }
    case 'boater': {
      const straw = toon('#e8dcbc')
      const brim = new THREE.Mesh(new THREE.CylinderGeometry(0.55, 0.55, 0.04, 16), straw)
      brim.position.y = 0.5
      head.add(brim)
      const crown = new THREE.Mesh(new THREE.CylinderGeometry(0.32, 0.32, 0.18, 16), straw)
      crown.position.y = 0.6
      head.add(crown)
      const band = new THREE.Mesh(new THREE.CylinderGeometry(0.325, 0.325, 0.08, 16), accent)
      band.position.y = 0.57
      head.add(band)
      break
    }
    case 'headband': {
      const band = new THREE.Mesh(new THREE.TorusGeometry(0.5, 0.045, 8, 24), accent)
      band.rotation.x = Math.PI / 2
      band.position.y = 0.2
      head.add(band)
      const feather = new THREE.Mesh(new THREE.ConeGeometry(0.06, 0.5, 6), toon('#f2ede2'))
      feather.position.set(0.3, 0.55, 0)
      feather.rotation.z = -0.4
      head.add(feather)
      const jewel = new THREE.Mesh(new THREE.SphereGeometry(0.05, 8, 8), gold)
      jewel.position.set(0.33, 0.26, 0.32)
      head.add(jewel)
      break
    }
    default:
  }
}

function addChestAccessory(group, appearance) {
  const material = toon(appearance.accessoryColor)
  const neckY = 0.72 + 0.58 * appearance.bodyScale
  switch (appearance.accessory) {
    case 'scarf': {
      const scarf = new THREE.Mesh(new THREE.TorusGeometry(0.32, 0.09, 10, 20), material)
      scarf.rotation.x = Math.PI / 2
      scarf.position.y = neckY - 0.06
      group.add(scarf)
      break
    }
    case 'bowtie': {
      for (const side of [-1, 1]) {
        const wing = new THREE.Mesh(new THREE.SphereGeometry(0.1, 10, 8), material)
        wing.scale.set(1, 0.6, 0.45)
        wing.position.set(side * 0.11, neckY - 0.12, 0.38)
        group.add(wing)
      }
      const knot = new THREE.Mesh(new THREE.SphereGeometry(0.05, 8, 8), material)
      knot.position.set(0, neckY - 0.12, 0.42)
      group.add(knot)
      break
    }
    case 'pendant': {
      const pendant = new THREE.Mesh(new THREE.SphereGeometry(0.07, 10, 10), material)
      pendant.position.set(0, neckY - 0.24, 0.4)
      group.add(pendant)
      break
    }
    case 'buttons':
      for (const [index, offset] of [0.16, 0.3, 0.44].entries()) {
        const button = new THREE.Mesh(new THREE.SphereGeometry(0.045, 8, 8), material)
        button.position.set(0, neckY - offset, 0.43 - index * 0.01)
        group.add(button)
      }
      break
    case 'badge': {
      const badge = new THREE.Mesh(new THREE.CylinderGeometry(0.08, 0.08, 0.04, 12), material)
      badge.rotation.x = Math.PI / 2
      badge.position.set(0.2, neckY - 0.18, 0.4)
      group.add(badge)
      break
    }
    default:
  }
}

function createVillager(name, role, seed) {
  const appearance = appearanceFor(name, seed)
  const color = appearance.outfit
  const group = new THREE.Group()

  // The whole character scales by height; sprites stay unscaled so tag and
  // bubble text is the same size on short and tall villagers.
  const figure = new THREE.Group()
  figure.scale.setScalar(appearance.height)
  group.add(figure)

  // The capsule body reads as the villager's evening attire.
  const body = new THREE.Mesh(new THREE.CapsuleGeometry(0.42, 0.5, 6, 16), toon(appearance.outfit))
  body.position.y = 0.72
  body.scale.y = appearance.bodyScale
  body.castShadow = true
  figure.add(body)

  // Face parts are children of the head so proportions scale together.
  const head = new THREE.Mesh(new THREE.SphereGeometry(0.52, 20, 16), toon(appearance.skin))
  head.position.y = 1.62
  head.scale.setScalar(appearance.headScale)
  head.castShadow = true
  figure.add(head)

  addHair(head, appearance)
  addNose(head, appearance)
  addEars(head, appearance)
  addHeadwear(head, appearance)
  addChestAccessory(figure, appearance)

  // White eyes with dark pupils.
  const eyeWhiteMaterial = new THREE.MeshBasicMaterial({ color: '#ffffff' })
  const pupilMaterial = new THREE.MeshBasicMaterial({ color: '#1d140c' })
  for (const side of [-1, 1]) {
    const eye = new THREE.Mesh(new THREE.SphereGeometry(0.075, 10, 10), eyeWhiteMaterial)
    eye.position.set(side * 0.18, 0.05, 0.45)
    head.add(eye)
    const pupil = new THREE.Mesh(new THREE.SphereGeometry(0.035, 8, 8), pupilMaterial)
    pupil.position.set(side * 0.18, 0.05, 0.515)
    head.add(pupil)
  }
  const blushMaterial = new THREE.MeshBasicMaterial({ color: '#ffb3a3' })
  for (const side of [-0.34, 0.34]) {
    const blush = new THREE.Mesh(new THREE.SphereGeometry(0.05, 8, 8), blushMaterial)
    blush.position.set(side, -0.08, 0.38)
    head.add(blush)
  }

  const tag = makeCanvasSprite(512, 176, 2.6)
  // Anchor the tag by its top edge so a two-line status grows downward.
  tag.sprite.center.set(0.5, 1)
  tag.sprite.position.y = 3.05 * appearance.height
  drawNameTag(tag, name, color, '')
  group.add(tag.sprite)

  const bubble = makeCanvasSprite(BUBBLE_CANVAS_WIDTH, BUBBLE_CANVAS_HEIGHT, BUBBLE_CANVAS_WIDTH / BUBBLE_PIXELS_PER_UNIT)
  // Anchor at the tail tip so taller bubbles grow upward, not over the tag.
  bubble.sprite.center.set(0.5, 0)
  bubble.sprite.position.y = 3.15 * appearance.height
  bubble.sprite.visible = false
  group.add(bubble.sprite)

  const think = new THREE.Sprite(getThinkingMaterial())
  think.scale.set(1.0, 0.83, 1)
  think.center.set(0.5, 0)
  think.position.set(0.75, 2.8 * appearance.height, 0)
  think.visible = false
  group.add(think)

  const angle = Math.random() * Math.PI * 2
  const distance = 2.5 + Math.random() * (WANDER_RADIUS - 2.5)
  group.position.set(Math.cos(angle) * distance, 0, Math.sin(angle) * distance)

  group.traverse(object => {
    object.userData.villagerName = name
  })
  scene.add(group)

  return {
    name,
    role,
    color,
    appearanceSeed: seed ?? 0,
    group,
    body,
    tag,
    bubble,
    think,
    bubbleHideAt: 0,
    thinking: false,
    thinkingSince: 0,
    thinkShownAt: 0,
    status: '',
    phase: Math.random() * Math.PI * 2,
    target: group.position.clone(),
    nextWanderAt: performance.now() / 1000 + 1 + Math.random() * 3,
  }
}

export function syncVillagers(coworkers, statuses) {
  const wanted = new Set(coworkers.map(coworker => coworker.name))
  for (const [name, villager] of villagers) {
    if (!wanted.has(name)) {
      scene.remove(villager.group)
      villagers.delete(name)
    }
  }
  for (const coworker of coworkers) {
    const seed = coworker.appearance ?? 0
    let villager = villagers.get(coworker.name)
    // A rerolled seed rebuilds the villager in place, keeping its spot.
    if (villager && villager.appearanceSeed !== seed) {
      const snapshot = {
        position: villager.group.position.clone(),
        rotationY: villager.group.rotation.y,
        target: villager.target.clone(),
        thinking: villager.thinking,
      }
      scene.remove(villager.group)
      villager = createVillager(coworker.name, coworker.role, seed)
      villager.group.position.copy(snapshot.position)
      villager.group.rotation.y = snapshot.rotationY
      villager.target.copy(snapshot.target)
      villager.thinking = snapshot.thinking
      villagers.set(coworker.name, villager)
    }
    if (!villager) {
      villager = createVillager(coworker.name, coworker.role, seed)
      villagers.set(coworker.name, villager)
    }
    const status = statuses.find(entry => entry.name.toLowerCase() === coworker.name.toLowerCase())
    const statusText = status ? `${status.status}${status.note ? ` — ${status.note}` : ''}` : ''
    if (statusText !== villager.status) {
      villager.status = statusText
      drawNameTag(villager.tag, villager.name, villager.color, statusText)
    }
  }
}

export function villagerColor(name, seed) {
  return appearanceFor(name, seed).outfit
}

export function agentBubble(name, text) {
  const villager = villagers.get(name)
  if (!villager || !text) return
  drawBubble(villager.bubble, text)
  villager.bubble.sprite.visible = true
  villager.bubbleHideAt = performance.now() / 1000 + bubbleSecondsFor(text)
}

// Thinking is its own small thought-cloud sprite, so it can never overwrite
// or resize a speech bubble that is still being read. Agents reason in many
// short bursts, so visibility is smoothed in the animation loop: the cloud
// appears only after ~0.3s of continuous thinking and stays at least ~1.2s.
export function setThinking(name, thinking) {
  const villager = villagers.get(name)
  if (!villager) return
  if (thinking && !villager.thinking) villager.thinkingSince = performance.now() / 1000
  villager.thinking = thinking
}

function animate() {
  const now = performance.now() / 1000
  const delta = Math.min(now - lastFrameAt, 0.1)
  lastFrameAt = now

  for (const villager of villagers.values()) {
    const { group } = villager
    // Bob and sway; think in place, wander otherwise.
    villager.body.position.y = 0.72 + Math.sin(now * (villager.thinking ? 7 : 3) + villager.phase) * 0.045
    if (!villager.thinking) {
      if (now >= villager.nextWanderAt) {
        const angle = Math.random() * Math.PI * 2
        const distance = Math.random() * WANDER_RADIUS
        villager.target.set(Math.cos(angle) * distance, 0, Math.sin(angle) * distance)
        villager.nextWanderAt = now + 4 + Math.random() * 6
      }
      const direction = villager.target.clone().sub(group.position)
      direction.y = 0
      if (direction.length() > 0.15) {
        direction.normalize()
        group.position.addScaledVector(direction, delta * 1.1)
        const desired = Math.atan2(direction.x, direction.z)
        let diff = desired - group.rotation.y
        diff = ((diff + Math.PI) % (Math.PI * 2)) - Math.PI
        group.rotation.y += diff * Math.min(1, delta * 6)
      }
    }
    if (villager.bubble.sprite.visible && now > villager.bubbleHideAt) {
      villager.bubble.sprite.visible = false
    }
    if (villager.thinking && !villager.think.visible && now - villager.thinkingSince > 0.3) {
      villager.think.visible = true
      villager.thinkShownAt = now
    } else if (!villager.thinking && villager.think.visible && now - villager.thinkShownAt > 1.2) {
      villager.think.visible = false
    }
  }

  if (bubbleGeometry) {
    const positions = bubbleGeometry.attributes.position
    for (let index = 0; index < bubbleSpeeds.length; index++) {
      let y = positions.getY(index) + bubbleSpeeds[index] * delta
      if (y > 34) y = -1.5
      positions.setY(index, y)
    }
    positions.needsUpdate = true
  }

  for (const material of neonMaterials) {
    const phase = material.userData.phase
    material.opacity = 0.78 + 0.22 * Math.sin(now * 1.4 + phase)
    // Occasional hard neon sputter.
    if (Math.sin(now * 11 + phase * 3) > 0.985) material.opacity *= 0.35
  }

  controls.update()
  renderer.render(scene, camera)
}

export function initScene({ container, onVillagerClick, onBoardClick, onWikiClick }) {
  villagerClickHandler = onVillagerClick ?? (() => {})
  boardClickHandler = onBoardClick ?? (() => {})
  wikiClickHandler = onWikiClick ?? (() => {})
  scene = new THREE.Scene()
  scene.background = new THREE.Color('#0b2836')
  scene.fog = new THREE.Fog('#0b2836', 30, 92)
  lastFrameAt = performance.now() / 1000

  camera = new THREE.PerspectiveCamera(45, window.innerWidth / window.innerHeight, 0.1, 200)
  camera.position.set(0, 13, 19)

  renderer = new THREE.WebGLRenderer({ antialias: true })
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2))
  renderer.setSize(window.innerWidth, window.innerHeight)
  renderer.shadowMap.enabled = true
  container.appendChild(renderer.domElement)

  controls = new OrbitControls(camera, renderer.domElement)
  controls.target.set(0, 1, 0)
  controls.enableDamping = true
  controls.enablePan = false
  controls.minDistance = 8
  controls.maxDistance = 40
  controls.maxPolarAngle = Math.PI * 0.46

  // Warm interior lamp light against the cool water outside.
  scene.add(new THREE.HemisphereLight('#7fb6c9', '#10222c', 0.9))
  const sun = new THREE.DirectionalLight('#ffd9a0', 1.5)
  sun.position.set(14, 22, 10)
  sun.castShadow = true
  sun.shadow.mapSize.set(2048, 2048)
  sun.shadow.camera.left = -28
  sun.shadow.camera.right = 28
  sun.shadow.camera.top = 28
  sun.shadow.camera.bottom = -28
  scene.add(sun)

  // The seabed far below, and the office floor as a raised dais.
  const seabed = new THREE.Mesh(new THREE.CircleGeometry(160, 48), new THREE.MeshBasicMaterial({ color: '#0a1d26' }))
  seabed.rotation.x = -Math.PI / 2
  seabed.position.y = -1.6
  scene.add(seabed)

  const daisWall = new THREE.Mesh(
    new THREE.CylinderGeometry(FLOOR_RADIUS + 0.2, FLOOR_RADIUS + 1.4, 1.4, 64, 1, true),
    new THREE.MeshToonMaterial({ color: '#181820', side: THREE.DoubleSide }),
  )
  daisWall.position.y = -0.7
  scene.add(daisWall)

  const floor = new THREE.Mesh(new THREE.CircleGeometry(FLOOR_RADIUS, 64), new THREE.MeshToonMaterial({ map: makeFloorTexture() }))
  floor.rotation.x = -Math.PI / 2
  floor.receiveShadow = true
  scene.add(floor)

  // Gold rim where the sand ring used to be.
  const rim = new THREE.Mesh(new THREE.TorusGeometry(FLOOR_RADIUS + 0.05, 0.14, 8, 96), toon('#d4af37'))
  rim.rotation.x = Math.PI / 2
  rim.position.y = 0.02
  scene.add(rim)

  for (const [x, z, scale] of [[-14, -10, 1.2], [13, -12, 1], [17, 6, 1.15], [-17, 7, 0.95], [-9, 15, 1.05], [8, 16, 0.95], [15, -2, 0.85], [-16, -2, 0.9]]) {
    addPalm(x, z, scale)
  }
  for (let index = 0; index < 12; index++) {
    const angle = (index / 12) * Math.PI * 2 + 0.26
    const distance = 11 + (index % 3) * 3
    addPlanter(Math.cos(angle) * distance, Math.sin(angle) * distance)
  }
  addSkyline()
  addBubbles()
  addGlassDome()
  const plaza = new THREE.Mesh(new THREE.CircleGeometry(4.6, 48), new THREE.MeshToonMaterial({ map: makePlazaTexture() }))
  plaza.rotation.x = -Math.PI / 2
  plaza.position.y = 0.01
  plaza.receiveShadow = true
  scene.add(plaza)

  for (const [x, z] of [[-5.5, 5.5], [5.5, 5.5], [-8.5, 0.5], [8.5, 0.5]]) addLampPost(x, z)

  addKanbanBoard()
  addWikiStand()

  // Villagers and the board are clickable (distinguish clicks from orbit
  // drags); hovering them shows a pointer cursor.
  const raycaster = new THREE.Raycaster()
  const pointer = new THREE.Vector2()

  function interactiveHit(event) {
    pointer.set((event.clientX / window.innerWidth) * 2 - 1, -(event.clientY / window.innerHeight) * 2 + 1)
    raycaster.setFromCamera(pointer, camera)
    const groups = [...villagers.values()].map(villager => villager.group)
    if (boardGroup) groups.push(boardGroup)
    if (wikiGroup) groups.push(wikiGroup)
    const hits = raycaster.intersectObjects(groups, true)
    const data = hits[0]?.object?.userData ?? {}
    if (data.villagerName) return { villager: data.villagerName }
    if (data.isBoard) return { board: true }
    if (data.isWiki) return { wiki: true }
    return null
  }

  let downAt = null
  renderer.domElement.addEventListener('pointerdown', event => {
    downAt = { x: event.clientX, y: event.clientY }
  })
  renderer.domElement.addEventListener('pointerup', event => {
    if (!downAt) return
    const moved = Math.hypot(event.clientX - downAt.x, event.clientY - downAt.y)
    downAt = null
    if (moved > 6) return
    const hit = interactiveHit(event)
    if (hit?.villager) villagerClickHandler(hit.villager)
    else if (hit?.board) boardClickHandler()
    else if (hit?.wiki) wikiClickHandler()
  })
  renderer.domElement.addEventListener('pointermove', event => {
    renderer.domElement.style.cursor = interactiveHit(event) ? 'pointer' : ''
  })

  window.addEventListener('resize', () => {
    camera.aspect = window.innerWidth / window.innerHeight
    camera.updateProjectionMatrix()
    renderer.setSize(window.innerWidth, window.innerHeight)
  })

  renderer.setAnimationLoop(animate)
}
