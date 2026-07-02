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

function pastelFor(name) {
  let hash = 0
  for (const char of name) hash = (hash * 31 + char.codePointAt(0)) >>> 0
  return PALETTE[hash % PALETTE.length]
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

function drawNameTag(tag, name, color, status) {
  const { ctx, canvas, texture } = tag
  ctx.clearRect(0, 0, canvas.width, canvas.height)
  ctx.fillStyle = 'rgba(255, 248, 230, 0.92)'
  ctx.strokeStyle = color
  ctx.lineWidth = 10
  roundedRect(ctx, 8, 8, canvas.width - 16, canvas.height - 16, 46)
  ctx.fill()
  ctx.stroke()
  ctx.fillStyle = '#6b4f2f'
  ctx.textAlign = 'center'
  ctx.textBaseline = 'middle'
  if (status) {
    ctx.font = '800 46px "M PLUS Rounded 1c", sans-serif'
    ctx.fillText(name, canvas.width / 2, 48)
    ctx.font = '500 32px "M PLUS Rounded 1c", sans-serif'
    ctx.fillStyle = '#9b7f5d'
    ctx.fillText(status, canvas.width / 2, 92)
  } else {
    ctx.font = '800 52px "M PLUS Rounded 1c", sans-serif'
    ctx.fillText(name, canvas.width / 2, canvas.height / 2)
  }
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

// Measure the text first, then size the canvas, bubble, and sprite to fit it:
// short quips get small bubbles, long messages get room to breathe.
function drawBubble(bubble, text) {
  const { canvas, texture, sprite } = bubble
  const ctx = canvas.getContext('2d')
  ctx.font = BUBBLE_FONT
  const lines = wrapBubbleLines(ctx, text, BUBBLE_MAX_TEXT_WIDTH, BUBBLE_MAX_LINES)
  const widest = Math.max(120, ...lines.map(line => ctx.measureText(line).width))
  const width = Math.ceil(Math.min(widest, BUBBLE_MAX_TEXT_WIDTH) + BUBBLE_PADDING)
  const height = lines.length * BUBBLE_LINE_HEIGHT + 56 + BUBBLE_TAIL
  canvas.width = width
  canvas.height = height

  // Resizing the canvas resets all context state.
  ctx.fillStyle = 'rgba(255, 255, 255, 0.96)'
  ctx.strokeStyle = '#e3d3a8'
  ctx.lineWidth = 8
  roundedRect(ctx, 10, 10, width - 20, height - 20 - BUBBLE_TAIL, 30)
  ctx.fill()
  ctx.stroke()
  // Tail.
  ctx.beginPath()
  ctx.moveTo(width / 2 - 24, height - BUBBLE_TAIL - 12)
  ctx.lineTo(width / 2, height - 8)
  ctx.lineTo(width / 2 + 24, height - BUBBLE_TAIL - 12)
  ctx.closePath()
  ctx.fill()
  ctx.stroke()

  ctx.fillStyle = '#6b4f2f'
  ctx.font = BUBBLE_FONT
  ctx.textAlign = 'center'
  ctx.textBaseline = 'middle'
  const centerY = 10 + (height - 20 - BUBBLE_TAIL) / 2
  lines.forEach((line, index) => {
    ctx.fillText(line, width / 2, centerY + (index - (lines.length - 1) / 2) * BUBBLE_LINE_HEIGHT)
  })
  texture.needsUpdate = true

  const worldWidth = width / BUBBLE_PIXELS_PER_UNIT
  sprite.scale.set(worldWidth, worldWidth * (height / width), 1)
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

function createVillager(name, role) {
  const color = pastelFor(name)
  const group = new THREE.Group()

  const body = new THREE.Mesh(new THREE.CapsuleGeometry(0.42, 0.5, 6, 16), toon(color))
  body.position.y = 0.72
  body.castShadow = true
  group.add(body)

  const head = new THREE.Mesh(new THREE.SphereGeometry(0.52, 20, 16), toon('#ffe8cf'))
  head.position.y = 1.62
  head.castShadow = true
  group.add(head)

  const hair = new THREE.Mesh(new THREE.SphereGeometry(0.54, 20, 12, 0, Math.PI * 2, 0, Math.PI * 0.55), toon(color))
  hair.position.y = 1.66
  group.add(hair)

  const eyeMaterial = new THREE.MeshBasicMaterial({ color: '#3c2c1e' })
  for (const side of [-0.18, 0.18]) {
    const eye = new THREE.Mesh(new THREE.SphereGeometry(0.06, 8, 8), eyeMaterial)
    eye.position.set(side, 1.64, 0.46)
    group.add(eye)
  }
  const blushMaterial = new THREE.MeshBasicMaterial({ color: '#ffb3a3' })
  for (const side of [-0.32, 0.32]) {
    const blush = new THREE.Mesh(new THREE.SphereGeometry(0.05, 8, 8), blushMaterial)
    blush.position.set(side, 1.52, 0.4)
    group.add(blush)
  }

  const tag = makeCanvasSprite(512, 128, 2.6)
  tag.sprite.position.y = 2.6
  drawNameTag(tag, name, color, '')
  group.add(tag.sprite)

  const bubble = makeCanvasSprite(512, 176, 3.1)
  // Anchor at the tail tip so taller bubbles grow upward, not over the tag.
  bubble.sprite.center.set(0.5, 0)
  bubble.sprite.position.y = 3.0
  bubble.sprite.visible = false
  group.add(bubble.sprite)

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
    group,
    tag,
    bubble,
    bubbleHideAt: 0,
    thinking: false,
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
    if (!villagers.has(coworker.name)) villagers.set(coworker.name, createVillager(coworker.name, coworker.role))
    const villager = villagers.get(coworker.name)
    const status = statuses.find(entry => entry.name.toLowerCase() === coworker.name.toLowerCase())
    const statusText = status ? `${status.status}${status.note ? ` — ${status.note}` : ''}` : ''
    if (statusText !== villager.status) {
      villager.status = statusText
      drawNameTag(villager.tag, villager.name, villager.color, statusText)
    }
  }
}

export function villagerColor(name) {
  return pastelFor(name)
}

export function agentBubble(name, text) {
  const villager = villagers.get(name)
  if (!villager || !text) return
  drawBubble(villager.bubble, text)
  villager.bubble.sprite.visible = true
  villager.bubbleHideAt = performance.now() / 1000 + bubbleSecondsFor(text)
}

export function setThinking(name, thinking) {
  const villager = villagers.get(name)
  if (!villager) return
  villager.thinking = thinking
  if (thinking) {
    drawBubble(villager.bubble, '💭 …')
    villager.bubble.sprite.visible = true
    villager.bubbleHideAt = Number.POSITIVE_INFINITY
  } else if (villager.bubbleHideAt === Number.POSITIVE_INFINITY) {
    villager.bubble.sprite.visible = false
    villager.bubbleHideAt = 0
  }
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
