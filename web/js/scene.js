// The Animal Crossing-flavored office: a grassy island with toon villagers
// (one per coworker) that wander, bob, think, and talk in speech bubbles.

import * as THREE from 'three'
import { OrbitControls } from 'three/addons/controls/OrbitControls.js'

const PALETTE = ['#ff9f9f', '#ffc38a', '#f7e07e', '#9fe08f', '#8fd8e8', '#9fb8ff', '#c9aaff', '#ffaad5']
const GRASS_RADIUS = 26
const WANDER_RADIUS = 8

const BUBBLE_FONT = '700 34px "M PLUS Rounded 1c", sans-serif'
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
const villagers = new Map()
const clouds = []
let villagerClickHandler = () => {}
let boardClickHandler = () => {}

// Animal Crossing-style villager traits, all derived from the seed. Humans
// are one species among the animals; for them the fur color is hair color.
const SPECIES = ['cat', 'dog', 'bear', 'rabbit', 'frog', 'bird', 'pig', 'mouse', 'human']
const FUR_COLORS = ['#f6d7b0', '#e8b48a', '#c98f63', '#9c6b45', '#9a9a9a', '#f5f0e6', '#ffd9e8', '#b6e3ff', '#cbb2ff', '#b9e6a1', '#ffe08a', '#f79d84']
const SKIN_TONES = ['#ffe3c9', '#f3d3ac', '#e3b58a', '#c98f63', '#9c6b45']
const ACCESSORIES = ['none', 'scarf', 'bowtie', 'pendant', 'buttons', 'badge']
const ACCESSORY_COLORS = ['#e86a5e', '#f5c542', '#5aa73a', '#4a90d9', '#b06ab3', '#e08a3c']

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
  const fur = pick(FUR_COLORS)
  return {
    species: pick(SPECIES),
    fur,
    muzzle: lighten(fur, 0.45),
    skin: pick(SKIN_TONES),
    shirt: pick(PALETTE),
    accessory: pick(ACCESSORIES),
    accessoryColor: pick(ACCESSORY_COLORS),
    bodyScale: 0.88 + random() * 0.28,
    headScale: 0.9 + random() * 0.22,
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
    ctx.font = `${weight} ${size}px "M PLUS Rounded 1c", sans-serif`
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

  ctx.font = '500 30px "M PLUS Rounded 1c", sans-serif'
  const statusLines = status ? wrapBubbleLines(ctx, status, maxTextWidth, 2) : []
  const cardHeight = 88 + statusLines.length * 34

  ctx.fillStyle = 'rgba(255, 248, 230, 0.92)'
  ctx.strokeStyle = color
  ctx.lineWidth = 10
  roundedRect(ctx, 8, 8, canvas.width - 16, cardHeight - 16, 40)
  ctx.fill()
  ctx.stroke()

  ctx.fillStyle = '#6b4f2f'
  ctx.textAlign = 'center'
  ctx.textBaseline = 'middle'
  const fittedName = fitText(ctx, name, maxTextWidth, '800', statusLines.length > 0 ? 46 : 52, 30)
  ctx.fillText(fittedName, canvas.width / 2, 48)

  ctx.fillStyle = '#9b7f5d'
  ctx.font = '500 30px "M PLUS Rounded 1c", sans-serif'
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

  ctx.fillStyle = 'rgba(255, 255, 255, 0.96)'
  ctx.strokeStyle = '#e3d3a8'
  ctx.lineWidth = 8
  roundedRect(ctx, left, top, bubbleWidth, bodyHeight, 30)
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

  ctx.fillStyle = '#6b4f2f'
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
  ctx.fillStyle = 'rgba(255, 255, 255, 0.96)'
  ctx.strokeStyle = '#e3d3a8'
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
  ctx.fillStyle = '#9b7f5d'
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

function makeGrassTexture() {
  const canvas = document.createElement('canvas')
  canvas.width = 256
  canvas.height = 256
  const ctx = canvas.getContext('2d')
  ctx.fillStyle = '#8fd170'
  ctx.fillRect(0, 0, 256, 256)
  ctx.fillStyle = '#83c765'
  for (let y = 0; y < 8; y++) {
    for (let x = 0; x < 8; x++) {
      if ((x + y) % 2 === 0) ctx.fillRect(x * 32, y * 32, 32, 32)
    }
  }
  ctx.strokeStyle = 'rgba(110, 170, 85, 0.55)'
  ctx.lineWidth = 2
  for (let index = 0; index < 240; index++) {
    const x = Math.random() * 256
    const y = Math.random() * 256
    ctx.beginPath()
    ctx.moveTo(x, y)
    ctx.lineTo(x + 2, y - 5)
    ctx.stroke()
  }
  const texture = new THREE.CanvasTexture(canvas)
  texture.colorSpace = THREE.SRGBColorSpace
  texture.wrapS = THREE.RepeatWrapping
  texture.wrapT = THREE.RepeatWrapping
  texture.repeat.set(10, 10)
  return texture
}

function toon(color) {
  return new THREE.MeshToonMaterial({ color })
}

function addTree(x, z, scale = 1) {
  const tree = new THREE.Group()
  const trunk = new THREE.Mesh(new THREE.CylinderGeometry(0.28 * scale, 0.36 * scale, 1.6 * scale, 8), toon('#9a6b43'))
  trunk.position.y = 0.8 * scale
  trunk.castShadow = true
  tree.add(trunk)
  const leafMaterial = toon('#5eb648')
  const positions = [
    [0, 2.2 * scale, 0, 1.15 * scale],
    [0.7 * scale, 1.8 * scale, 0.15 * scale, 0.8 * scale],
    [-0.65 * scale, 1.85 * scale, -0.1 * scale, 0.75 * scale],
  ]
  for (const [px, py, pz, radius] of positions) {
    const blob = new THREE.Mesh(new THREE.SphereGeometry(radius, 16, 12), leafMaterial)
    blob.position.set(px, py, pz)
    blob.castShadow = true
    tree.add(blob)
  }
  tree.position.set(x, 0, z)
  scene.add(tree)
}

function addFlower(x, z) {
  const flower = new THREE.Group()
  const stem = new THREE.Mesh(new THREE.CylinderGeometry(0.03, 0.03, 0.35, 6), toon('#4f9d3f'))
  stem.position.y = 0.18
  flower.add(stem)
  const bloom = new THREE.Mesh(new THREE.SphereGeometry(0.12, 10, 8), toon(PALETTE[Math.floor(Math.random() * PALETTE.length)]))
  bloom.position.y = 0.4
  flower.add(bloom)
  flower.position.set(x, 0, z)
  scene.add(flower)
}

function addKanbanBoard() {
  const board = new THREE.Group()
  const legMaterial = toon('#9a6b43')
  for (const side of [-1.3, 1.3]) {
    const leg = new THREE.Mesh(new THREE.CylinderGeometry(0.09, 0.09, 2.2, 8), legMaterial)
    leg.position.set(side, 1.1, 0)
    leg.castShadow = true
    board.add(leg)
  }
  const frame = new THREE.Mesh(new THREE.BoxGeometry(3.2, 1.9, 0.14), toon('#b98a5d'))
  frame.position.y = 2.1
  frame.castShadow = true
  board.add(frame)

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

export function updateBoard(tasks) {
  if (!boardCanvas) return
  const ctx = boardCanvas.getContext('2d')
  ctx.fillStyle = '#fff3d6'
  ctx.fillRect(0, 0, boardCanvas.width, boardCanvas.height)
  ctx.fillStyle = '#6b4f2f'
  ctx.textAlign = 'center'
  ctx.font = '800 44px "M PLUS Rounded 1c", sans-serif'
  ctx.fillText('📌 Office board', boardCanvas.width / 2, 56)
  ctx.font = '600 30px "M PLUS Rounded 1c", sans-serif'
  const columns = ['todo', 'doing', 'blocked', 'review', 'done']
  columns.forEach((column, index) => {
    const count = tasks.filter(task => task.status === column).length
    ctx.fillText(`${column}: ${count}`, boardCanvas.width / 2, 112 + index * 38)
  })
  boardTexture.needsUpdate = true
}

function addClouds() {
  const material = new THREE.MeshToonMaterial({ color: '#ffffff' })
  for (let index = 0; index < 5; index++) {
    const cloud = new THREE.Group()
    for (const [px, py, radius] of [[-1.2, 0, 0.9], [0, 0.35, 1.2], [1.3, 0, 0.85]]) {
      const puff = new THREE.Mesh(new THREE.SphereGeometry(radius, 12, 10), material)
      puff.position.set(px, py, 0)
      cloud.add(puff)
    }
    cloud.position.set(-30 + index * 14, 11 + (index % 3) * 1.6, -14 + (index % 2) * 20)
    cloud.userData.speed = 0.4 + (index % 3) * 0.18
    clouds.push(cloud)
    scene.add(cloud)
  }
}

function addMuzzle(head, appearance) {
  const muzzle = new THREE.Mesh(new THREE.SphereGeometry(0.19, 12, 10), toon(appearance.muzzle))
  muzzle.scale.set(1, 0.72, 0.65)
  muzzle.position.set(0, -0.16, 0.42)
  head.add(muzzle)
  const nose = new THREE.Mesh(new THREE.SphereGeometry(0.05, 8, 8), toon('#6b4f2f'))
  nose.position.set(0, -0.08, 0.56)
  head.add(nose)
}

// Ears, beaks, and snouts that make each species readable at a glance.
function addSpeciesFeatures(head, appearance) {
  const fur = toon(appearance.fur)
  switch (appearance.species) {
    case 'cat':
      for (const side of [-1, 1]) {
        const ear = new THREE.Mesh(new THREE.ConeGeometry(0.16, 0.32, 8), fur)
        ear.position.set(side * 0.27, 0.48, 0)
        ear.rotation.z = side * -0.25
        head.add(ear)
      }
      addMuzzle(head, appearance)
      break
    case 'dog':
      for (const side of [-1, 1]) {
        const ear = new THREE.Mesh(new THREE.SphereGeometry(0.16, 10, 8), fur)
        ear.scale.set(0.55, 1.25, 0.45)
        ear.position.set(side * 0.42, 0.18, 0)
        ear.rotation.z = side * 0.85
        head.add(ear)
      }
      addMuzzle(head, appearance)
      break
    case 'bear':
      for (const side of [-1, 1]) {
        const ear = new THREE.Mesh(new THREE.SphereGeometry(0.15, 10, 8), fur)
        ear.position.set(side * 0.3, 0.44, 0)
        head.add(ear)
      }
      addMuzzle(head, appearance)
      break
    case 'rabbit':
      for (const side of [-1, 1]) {
        const ear = new THREE.Mesh(new THREE.SphereGeometry(0.15, 10, 8), fur)
        ear.scale.set(0.55, 1.9, 0.55)
        ear.position.set(side * 0.2, 0.66, 0)
        ear.rotation.z = side * -0.12
        head.add(ear)
      }
      addMuzzle(head, appearance)
      break
    case 'frog': {
      // Eye bumps on top of the head; the eyes themselves sit on these.
      for (const side of [-1, 1]) {
        const bump = new THREE.Mesh(new THREE.SphereGeometry(0.16, 12, 10), fur)
        bump.position.set(side * 0.24, 0.42, 0.14)
        head.add(bump)
      }
      const mouth = new THREE.Mesh(new THREE.TorusGeometry(0.17, 0.022, 8, 16, Math.PI), toon('#6b4f2f'))
      mouth.rotation.z = Math.PI
      mouth.position.set(0, -0.1, 0.47)
      head.add(mouth)
      break
    }
    case 'human': {
      const hair = new THREE.Mesh(new THREE.SphereGeometry(0.55, 20, 12, 0, Math.PI * 2, 0, Math.PI * 0.5), toon(appearance.fur))
      hair.position.y = 0.06
      head.add(hair)
      for (const side of [-1, 1]) {
        const ear = new THREE.Mesh(new THREE.SphereGeometry(0.09, 8, 8), toon(appearance.skin))
        ear.position.set(side * 0.5, 0, 0)
        head.add(ear)
      }
      break
    }
    case 'bird': {
      const beak = new THREE.Mesh(new THREE.ConeGeometry(0.11, 0.28, 10), toon('#f2a33c'))
      beak.rotation.x = Math.PI / 2
      beak.position.set(0, -0.04, 0.56)
      head.add(beak)
      const tuft = new THREE.Mesh(new THREE.ConeGeometry(0.1, 0.24, 8), fur)
      tuft.position.set(0.04, 0.54, 0)
      tuft.rotation.z = -0.35
      head.add(tuft)
      break
    }
    case 'pig': {
      const snout = new THREE.Mesh(new THREE.CylinderGeometry(0.13, 0.13, 0.12, 12), toon(appearance.muzzle))
      snout.rotation.x = Math.PI / 2
      snout.position.set(0, -0.06, 0.5)
      head.add(snout)
      for (const side of [-1, 1]) {
        const ear = new THREE.Mesh(new THREE.ConeGeometry(0.12, 0.22, 8), fur)
        ear.position.set(side * 0.26, 0.44, 0)
        ear.rotation.z = side * -0.5
        head.add(ear)
      }
      break
    }
    case 'mouse':
      for (const side of [-1, 1]) {
        const ear = new THREE.Mesh(new THREE.SphereGeometry(0.19, 12, 10), fur)
        ear.scale.set(1, 1, 0.35)
        ear.position.set(side * 0.34, 0.4, 0)
        head.add(ear)
      }
      addMuzzle(head, appearance)
      break
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
  const color = appearance.fur
  const group = new THREE.Group()

  // The capsule body reads as the villager's outfit; the head is fur.
  const body = new THREE.Mesh(new THREE.CapsuleGeometry(0.42, 0.5, 6, 16), toon(appearance.shirt))
  body.position.y = 0.72
  body.scale.y = appearance.bodyScale
  body.castShadow = true
  group.add(body)

  // Face parts are children of the head so proportions scale together.
  const headColor = appearance.species === 'human' ? appearance.skin : appearance.fur
  const head = new THREE.Mesh(new THREE.SphereGeometry(0.52, 20, 16), toon(headColor))
  head.position.y = 1.62
  head.scale.setScalar(appearance.headScale)
  head.castShadow = true
  group.add(head)

  addSpeciesFeatures(head, appearance)
  addChestAccessory(group, appearance)

  // White eyes with dark pupils; frog eyes sit on top of its head bumps.
  const eyeWhiteMaterial = new THREE.MeshBasicMaterial({ color: '#ffffff' })
  const pupilMaterial = new THREE.MeshBasicMaterial({ color: '#1d140c' })
  const eyeSpot = appearance.species === 'frog'
    ? { x: 0.24, y: 0.5, z: 0.24, size: 0.085 }
    : { x: 0.18, y: 0.05, z: 0.45, size: 0.075 }
  for (const side of [-1, 1]) {
    const eye = new THREE.Mesh(new THREE.SphereGeometry(eyeSpot.size, 10, 10), eyeWhiteMaterial)
    eye.position.set(side * eyeSpot.x, eyeSpot.y, eyeSpot.z)
    head.add(eye)
    const pupil = new THREE.Mesh(new THREE.SphereGeometry(0.035, 8, 8), pupilMaterial)
    pupil.position.set(side * eyeSpot.x, eyeSpot.y + (appearance.species === 'frog' ? 0.02 : 0), eyeSpot.z + 0.065)
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
  tag.sprite.position.y = 3.05
  drawNameTag(tag, name, color, '')
  group.add(tag.sprite)

  const bubble = makeCanvasSprite(BUBBLE_CANVAS_WIDTH, BUBBLE_CANVAS_HEIGHT, BUBBLE_CANVAS_WIDTH / BUBBLE_PIXELS_PER_UNIT)
  // Anchor at the tail tip so taller bubbles grow upward, not over the tag.
  bubble.sprite.center.set(0.5, 0)
  bubble.sprite.position.y = 3.15
  bubble.sprite.visible = false
  group.add(bubble.sprite)

  const think = new THREE.Sprite(getThinkingMaterial())
  think.scale.set(1.0, 0.83, 1)
  think.center.set(0.5, 0)
  think.position.set(0.75, 2.8, 0)
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
  return appearanceFor(name, seed).fur
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
    group.children[0].position.y = 0.72 + Math.sin(now * (villager.thinking ? 7 : 3) + villager.phase) * 0.045
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

  for (const cloud of clouds) {
    cloud.position.x += cloud.userData.speed * delta
    if (cloud.position.x > 38) cloud.position.x = -38
  }

  controls.update()
  renderer.render(scene, camera)
}

export function initScene({ container, onVillagerClick, onBoardClick }) {
  villagerClickHandler = onVillagerClick ?? (() => {})
  boardClickHandler = onBoardClick ?? (() => {})
  scene = new THREE.Scene()
  scene.background = new THREE.Color('#aee3f5')
  scene.fog = new THREE.Fog('#aee3f5', 34, 75)
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

  scene.add(new THREE.HemisphereLight('#ffffff', '#9cd07f', 0.9))
  const sun = new THREE.DirectionalLight('#fff4d6', 1.6)
  sun.position.set(14, 22, 10)
  sun.castShadow = true
  sun.shadow.mapSize.set(2048, 2048)
  sun.shadow.camera.left = -28
  sun.shadow.camera.right = 28
  sun.shadow.camera.top = 28
  sun.shadow.camera.bottom = -28
  scene.add(sun)

  // Sea, island, props.
  const sea = new THREE.Mesh(new THREE.CircleGeometry(160, 48), new THREE.MeshToonMaterial({ color: '#6cc3e8' }))
  sea.rotation.x = -Math.PI / 2
  sea.position.y = -0.12
  scene.add(sea)

  const grass = new THREE.Mesh(new THREE.CircleGeometry(GRASS_RADIUS, 64), new THREE.MeshToonMaterial({ map: makeGrassTexture() }))
  grass.rotation.x = -Math.PI / 2
  grass.receiveShadow = true
  scene.add(grass)

  const sand = new THREE.Mesh(
    new THREE.RingGeometry(GRASS_RADIUS - 0.4, GRASS_RADIUS + 2.4, 64),
    new THREE.MeshToonMaterial({ color: '#f2e3b3' }),
  )
  sand.rotation.x = -Math.PI / 2
  sand.position.y = -0.02
  scene.add(sand)

  for (const [x, z, scale] of [[-14, -10, 1.3], [13, -12, 1.1], [17, 6, 1.25], [-17, 7, 1], [-9, 15, 1.15], [8, 16, 1], [15, -2, 0.9], [-16, -2, 0.95]]) {
    addTree(x, z, scale)
  }
  for (let index = 0; index < 14; index++) {
    const angle = Math.random() * Math.PI * 2
    const distance = 9 + Math.random() * 9
    addFlower(Math.cos(angle) * distance, Math.sin(angle) * distance)
  }
  addKanbanBoard()
  addClouds()

  // Villagers and the board are clickable (distinguish clicks from orbit
  // drags); hovering them shows a pointer cursor.
  const raycaster = new THREE.Raycaster()
  const pointer = new THREE.Vector2()

  function interactiveHit(event) {
    pointer.set((event.clientX / window.innerWidth) * 2 - 1, -(event.clientY / window.innerHeight) * 2 + 1)
    raycaster.setFromCamera(pointer, camera)
    const groups = [...villagers.values()].map(villager => villager.group)
    if (boardGroup) groups.push(boardGroup)
    const hits = raycaster.intersectObjects(groups, true)
    const data = hits[0]?.object?.userData ?? {}
    if (data.villagerName) return { villager: data.villagerName }
    if (data.isBoard) return { board: true }
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
