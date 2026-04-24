<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, shallowRef } from "vue";

import bulletUrl from "../../assets/sprites/bullet.png";
import enemyDownUrl from "../../assets/sprites/enemy-down.png";
import enemyLeftUrl from "../../assets/sprites/enemy-left.png";
import enemyRightUrl from "../../assets/sprites/enemy-right.png";
import enemyUpUrl from "../../assets/sprites/enemy-up.png";
import explosionUrl from "../../assets/sprites/explosion.png";
import heartEmptyUrl from "../../assets/sprites/heart-empty.png";
import heartUrl from "../../assets/sprites/heart.png";
import playerUrl from "../../assets/sprites/player-up.png";
import spriteSheetUrl from "../../assets/sprite-sheet-reference.png";
import starUrl from "../../assets/sprites/star.png";
import wallUrl from "../../assets/sprites/wall.png";
import {
  createGame,
  movePlayer,
  shoot,
  toCells,
  togglePause,
  updateGame,
  type Cell,
  type Direction,
} from "./game";

const game = shallowRef(createGame());
const runtime = shallowRef(window.__TAURI_INTERNALS__ ? "Tauri Desktop" : "Web");
const cells = computed(() => toCells(game.value));
const hearts = computed(() => Array.from({ length: 3 }, (_, index) => index < game.value.lives));
const stateLabel = computed(() => {
  if (game.value.gameOver) return "GAME OVER";
  if (game.value.paused) return "PAUSED";
  return "RUNNING";
});

function spriteFor(cell: Cell): string | undefined {
  if (cell.kind === "wall") return wallUrl;
  if (cell.kind === "player") return playerUrl;
  if (cell.kind === "enemy") {
    if (cell.dir === "up") return enemyUpUrl;
    if (cell.dir === "left") return enemyLeftUrl;
    if (cell.dir === "right") return enemyRightUrl;
    return enemyDownUrl;
  }
  if (cell.kind === "playerBullet" || cell.kind === "enemyBullet") return bulletUrl;
  if (cell.kind === "explosion") return explosionUrl;
  return undefined;
}

let timer: number | undefined;

function dispatchMove(dir: Direction) {
  game.value = movePlayer(game.value, dir);
}

function restart() {
  game.value = createGame();
}

function handleKeydown(event: KeyboardEvent) {
  const code = event.key.toLowerCase();

  if (code === "arrowup" || code === "w") {
    event.preventDefault();
    dispatchMove("up");
  } else if (code === "arrowdown" || code === "s") {
    event.preventDefault();
    dispatchMove("down");
  } else if (code === "arrowleft" || code === "a") {
    event.preventDefault();
    dispatchMove("left");
  } else if (code === "arrowright" || code === "d") {
    event.preventDefault();
    dispatchMove("right");
  } else if (code === " ") {
    event.preventDefault();
    game.value = shoot(game.value);
  } else if (code === "p") {
    game.value = togglePause(game.value);
  } else if (code === "r" && game.value.gameOver) {
    restart();
  }
}

onMounted(() => {
  window.addEventListener("keydown", handleKeydown);
  timer = window.setInterval(() => {
    game.value = updateGame(game.value);
  }, 90);
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", handleKeydown);
  if (timer !== undefined) window.clearInterval(timer);
});
</script>

<template>
  <main class="shell">
    <section class="game-panel" aria-label="Tank Battle game">
      <header class="topbar">
        <div>
          <p class="eyebrow">{{ runtime }}</p>
          <h1>Tank Battle</h1>
        </div>
        <div class="status" :class="{ paused: game.paused, over: game.gameOver }">
          {{ stateLabel }}
        </div>
      </header>

      <div class="hud" aria-label="Game status">
        <div class="metric">
          <span class="metric-label">Score</span>
          <img class="hud-icon star" :src="starUrl" alt="" />
          <strong>{{ game.score }}</strong>
        </div>
        <div class="metric hearts" aria-label="Lives">
          <img
            class="heart"
            v-for="(filled, index) in hearts"
            :key="index"
            :src="filled ? heartUrl : heartEmptyUrl"
            alt=""
          />
        </div>
        <div class="metric">
          <span class="metric-label">Enemies</span>
          <strong>{{ game.enemies.length }}</strong>
        </div>
      </div>

      <div class="board-wrap">
        <div
          class="board"
          :style="{ '--cols': game.width, '--rows': game.height }"
          role="grid"
          aria-label="Battlefield"
        >
          <span
            v-for="cell in cells"
            :key="cell.id"
            class="cell"
            :class="[cell.kind, cell.dir]"
            role="gridcell"
          >
            <img v-if="spriteFor(cell)" class="sprite" :src="spriteFor(cell)" alt="" />
          </span>
        </div>
      </div>

      <div class="controls" aria-label="Touch controls">
        <button type="button" aria-label="Move up" @click="dispatchMove('up')">▲</button>
        <button type="button" aria-label="Move left" @click="dispatchMove('left')">◀</button>
        <button type="button" aria-label="Fire" class="fire" @click="game = shoot(game)">●</button>
        <button type="button" aria-label="Move right" @click="dispatchMove('right')">▶</button>
        <button type="button" aria-label="Move down" @click="dispatchMove('down')">▼</button>
        <button type="button" aria-label="Pause" @click="game = togglePause(game)">Ⅱ</button>
        <button type="button" aria-label="Restart" @click="restart">↻</button>
      </div>
    </section>

    <aside class="asset-panel" aria-label="Generated asset reference">
      <div class="asset-copy">
        <p class="eyebrow">imageGen reference</p>
        <h2>Agents generated assets</h2>
        <p>
          这张像素素材图由 OpenAI imageGen 生成。Vue/Web 和 Tauri
          桌面版使用同一套颜色、符号和交互规则。
        </p>
      </div>
      <img :src="spriteSheetUrl" alt="Generated pixel-art tank sprite sheet reference" />
    </aside>
  </main>
</template>
