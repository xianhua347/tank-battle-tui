export type Direction = "up" | "down" | "left" | "right";
export type Owner = "player" | "enemy";
export type CellKind =
  | "empty"
  | "wall"
  | "player"
  | "enemy"
  | "playerBullet"
  | "enemyBullet"
  | "explosion";

export interface Pos {
  x: number;
  y: number;
}

export interface Tank {
  pos: Pos;
  dir: Direction;
  cooldown: number;
}

export interface Bullet {
  pos: Pos;
  dir: Direction;
  owner: Owner;
}

export interface Explosion {
  pos: Pos;
  age: number;
}

export interface Cell {
  id: string;
  kind: CellKind;
  dir?: Direction;
  symbol: string;
}

export interface GameState {
  width: number;
  height: number;
  player: Tank;
  enemies: Tank[];
  bullets: Bullet[];
  explosions: Explosion[];
  walls: Set<string>;
  score: number;
  lives: number;
  paused: boolean;
  gameOver: boolean;
  tick: number;
}

export const width = 42;
export const height = 22;

const enemyCount = 6;
const dirs: Direction[] = ["up", "down", "left", "right"];

export function createGame(): GameState {
  const state: GameState = {
    width,
    height,
    player: {
      pos: { x: Math.floor(width / 2), y: height - 3 },
      dir: "up",
      cooldown: 0,
    },
    enemies: [],
    bullets: [],
    explosions: [],
    walls: new Set<string>(),
    score: 0,
    lives: 3,
    paused: false,
    gameOver: false,
    tick: 0,
  };

  seedMap(state);
  return state;
}

export function cloneGame(state: GameState): GameState {
  return {
    ...state,
    player: cloneTank(state.player),
    enemies: state.enemies.map(cloneTank),
    bullets: state.bullets.map((bullet) => ({ ...bullet, pos: { ...bullet.pos } })),
    explosions: state.explosions.map((explosion) => ({ ...explosion, pos: { ...explosion.pos } })),
    walls: new Set(state.walls),
  };
}

export function movePlayer(state: GameState, dir: Direction): GameState {
  const next = cloneGame(state);
  if (next.paused || next.gameOver) return next;

  next.player.dir = dir;
  const pos = step(next.player.pos, dir);
  if (canEnter(next, pos, "player")) {
    next.player.pos = pos;
  }
  return next;
}

export function shoot(state: GameState): GameState {
  const next = cloneGame(state);
  if (next.paused || next.gameOver || next.player.cooldown > 0) return next;

  next.bullets.push({
    pos: bulletSpawnPos(next.player),
    dir: next.player.dir,
    owner: "player",
  });
  next.player.cooldown = 5;
  return next;
}

export function togglePause(state: GameState): GameState {
  const next = cloneGame(state);
  if (!next.gameOver) next.paused = !next.paused;
  return next;
}

export function updateGame(state: GameState): GameState {
  const next = cloneGame(state);
  if (next.paused || next.gameOver) return next;

  next.tick += 1;
  next.player.cooldown = Math.max(0, next.player.cooldown - 1);
  for (const enemy of next.enemies) {
    enemy.cooldown = Math.max(0, enemy.cooldown - 1);
  }

  updateEnemies(next);
  updateBullets(next);
  updateExplosions(next);

  if (next.enemies.length === 0) {
    spawnWave(next);
  }

  return next;
}

export function toCells(state: GameState): Cell[] {
  const cells: Cell[] = [];
  const byPos = new Map<string, Pick<Cell, "kind" | "dir">>();

  for (const wall of state.walls) byPos.set(wall, { kind: "wall" });
  for (const bullet of state.bullets) {
    byPos.set(key(bullet.pos), {
      kind: bullet.owner === "player" ? "playerBullet" : "enemyBullet",
      dir: bullet.dir,
    });
  }
  for (const explosion of state.explosions) byPos.set(key(explosion.pos), { kind: "explosion" });
  for (const enemy of state.enemies) byPos.set(key(enemy.pos), { kind: "enemy", dir: enemy.dir });
  byPos.set(key(state.player.pos), { kind: "player", dir: state.player.dir });

  for (let y = 0; y < state.height; y += 1) {
    for (let x = 0; x < state.width; x += 1) {
      const pos = { x, y };
      const cell = byPos.get(key(pos)) ?? { kind: "empty" as const };
      cells.push({
        id: `${x}-${y}`,
        kind: cell.kind,
        dir: cell.dir,
        symbol: symbolFor(cell.kind, pos, state),
      });
    }
  }

  return cells;
}

function seedMap(state: GameState): void {
  for (let i = 0; i < 40; i += 1) {
    const pos = {
      x: randomInt(2, width - 3),
      y: randomInt(3, height - 5),
    };

    if (
      !tankFootprint(state.player.pos).some((playerPos) => samePos(playerPos, pos)) &&
      (Math.abs(pos.x - state.player.pos.x) > 3 || Math.abs(pos.y - state.player.pos.y) > 3)
    ) {
      state.walls.add(key(pos));
    }
  }

  for (let i = 0; i < enemyCount; i += 1) {
    const pos = {
      x: 4 + ((i * 7) % (width - 8)),
      y: 2 + Math.floor(i / 4) * 3,
    };

    if (canTankEnter(state, pos, "enemy")) {
      state.enemies.push({
        pos,
        dir: "down",
        cooldown: randomInt(8, 20),
      });
    }
  }
}

function updateEnemies(state: GameState): void {
  for (const [index, enemy] of state.enemies.entries()) {
    let dir = enemy.dir;
    if (Math.random() < 0.18) dir = dirs[randomInt(0, dirs.length - 1)];

    const next = step(enemy.pos, dir);
    enemy.dir = dir;

    if (canTankEnter(state, next, "enemy", index)) {
      enemy.pos = next;
    }

    if (enemy.cooldown === 0 && Math.random() < 0.42) {
      state.bullets.push({
        pos: bulletSpawnPos(enemy),
        dir: enemy.dir,
        owner: "enemy",
      });
      enemy.cooldown = 14;
    }
  }
}

function updateBullets(state: GameState): void {
  const nextBullets: Bullet[] = [];
  const killedEnemies = new Set<number>();
  let damagedPlayer = false;

  for (const bullet of state.bullets) {
    if (!inBounds(bullet.pos)) continue;

    const currentHit = resolveHit(state, bullet.pos, bullet.owner, killedEnemies);
    if (currentHit === "player") damagedPlayer = true;
    if (currentHit !== "none") continue;

    const next = step(bullet.pos, bullet.dir);
    if (!inBounds(next)) continue;
    const nextHit = resolveHit(state, next, bullet.owner, killedEnemies);
    if (nextHit === "player") damagedPlayer = true;
    if (nextHit !== "none") continue;

    nextBullets.push({ ...bullet, pos: next });
  }

  state.enemies = state.enemies.filter((_, index) => !killedEnemies.has(index));
  state.bullets = nextBullets;

  if (damagedPlayer) {
    state.lives = Math.max(0, state.lives - 1);
    state.player.pos = { x: Math.floor(width / 2), y: height - 3 };
    state.player.dir = "up";
    clearTankSpace(state, state.player.pos);
    if (state.lives === 0) state.gameOver = true;
  }
}

type HitResult = "none" | "blocked" | "player";

function resolveHit(
  state: GameState,
  pos: Pos,
  owner: Owner,
  killedEnemies: Set<number>,
): HitResult {
  const posKey = key(pos);
  if (state.walls.delete(posKey)) {
    state.explosions.push({ pos, age: 0 });
    return "blocked";
  }

  if (owner === "player") {
    const enemyIndex = state.enemies.findIndex(
      (enemy, index) =>
        !killedEnemies.has(index) &&
        tankFootprint(enemy.pos).some((enemyPos) => samePos(enemyPos, pos)),
    );
    if (enemyIndex >= 0) {
      killedEnemies.add(enemyIndex);
      state.score += 100;
      state.explosions.push({ pos, age: 0 });
      return "blocked";
    }
  }

  if (
    owner === "enemy" &&
    tankFootprint(state.player.pos).some((playerPos) => samePos(playerPos, pos))
  ) {
    state.explosions.push({ pos, age: 0 });
    return "player";
  }

  return "none";
}

function updateExplosions(state: GameState): void {
  for (const explosion of state.explosions) explosion.age += 1;
  state.explosions = state.explosions.filter((explosion) => explosion.age < 5);
}

function spawnWave(state: GameState): void {
  const count = Math.min(10, enemyCount + Math.floor(state.score / 800));

  for (let i = 0; i < count; i += 1) {
    for (let attempt = 0; attempt < 30; attempt += 1) {
      const pos = {
        x: randomInt(2, width - 3),
        y: randomInt(1, Math.floor(height / 2)),
      };

      if (canEnter(state, pos, "enemy")) {
        state.enemies.push({
          pos,
          dir: "down",
          cooldown: randomInt(5, 22),
        });
        break;
      }
    }
  }
}

function canEnter(state: GameState, pos: Pos, mover: Owner): boolean {
  return canTankEnter(state, pos, mover);
}

function symbolFor(kind: CellKind, pos: Pos, state: GameState): string {
  if (kind === "player") return directionSymbol(state.player.dir);
  if (kind === "enemy")
    return directionSymbol(state.enemies.find((enemy) => samePos(enemy.pos, pos))?.dir ?? "down");
  if (kind === "wall") return "#";
  if (kind === "playerBullet") return "*";
  if (kind === "enemyBullet") return "o";
  if (kind === "explosion") return "@";
  return "";
}

function directionSymbol(dir: Direction): string {
  if (dir === "up") return "^";
  if (dir === "down") return "v";
  if (dir === "left") return "<";
  return ">";
}

function cloneTank(tank: Tank): Tank {
  return {
    ...tank,
    pos: { ...tank.pos },
  };
}

function step(pos: Pos, dir: Direction): Pos {
  if (dir === "up") return { x: pos.x, y: pos.y - 1 };
  if (dir === "down") return { x: pos.x, y: pos.y + 1 };
  if (dir === "left") return { x: pos.x - 1, y: pos.y };
  return { x: pos.x + 1, y: pos.y };
}

function bulletSpawnPos(tank: Tank): Pos {
  return step(step(tank.pos, tank.dir), tank.dir);
}

function tankFootprint(pos: Pos): Pos[] {
  return [
    pos,
    { x: pos.x, y: pos.y - 1 },
    { x: pos.x, y: pos.y + 1 },
    { x: pos.x - 1, y: pos.y },
    { x: pos.x + 1, y: pos.y },
  ];
}

function canTankEnter(
  state: GameState,
  pos: Pos,
  mover: Owner,
  ignoreEnemyIndex?: number,
): boolean {
  const footprint = tankFootprint(pos);
  if (
    footprint.some((footprintPos) => !inBounds(footprintPos) || state.walls.has(key(footprintPos)))
  ) {
    return false;
  }

  if (
    mover === "enemy" &&
    tankFootprint(state.player.pos).some((playerPos) =>
      footprint.some((footprintPos) => samePos(footprintPos, playerPos)),
    )
  ) {
    return false;
  }

  return !state.enemies.some((enemy, index) => {
    if (index === ignoreEnemyIndex) return false;
    const enemyFootprint = tankFootprint(enemy.pos);
    return footprint.some((footprintPos) =>
      enemyFootprint.some((enemyPos) => samePos(footprintPos, enemyPos)),
    );
  });
}

function clearTankSpace(state: GameState, pos: Pos): void {
  for (const footprintPos of tankFootprint(pos)) {
    state.walls.delete(key(footprintPos));
  }
}

function samePos(a: Pos, b: Pos): boolean {
  return a.x === b.x && a.y === b.y;
}

function inBounds(pos: Pos): boolean {
  return pos.x >= 0 && pos.x < width && pos.y >= 0 && pos.y < height;
}

function key(pos: Pos): string {
  return `${pos.x},${pos.y}`;
}

function randomInt(min: number, max: number): number {
  return Math.floor(Math.random() * (max - min + 1)) + min;
}
