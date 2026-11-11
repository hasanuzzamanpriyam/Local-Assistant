<script lang="ts">
  import { overlayStore } from '../../stores/overlay';

  let cursorX = $state(0);
  let cursorY = $state(0);
  let step = $state(0);
  let maxSteps = $state(10);
  let thought = $state('');
  let action = $state('');
  let elapsedMs = $state(0);
  let config = overlayStore.config;

  overlayStore.subscribeCursor((pos) => { cursorX = pos.x; cursorY = pos.y; })();
  overlayStore.subscribeStep((s) => { step = s; })();
  overlayStore.subscribeMaxSteps((m) => { maxSteps = m; })();
  overlayStore.subscribeThought((t) => { thought = t; })();
  overlayStore.subscribeAction((a) => { action = a; })();
  overlayStore.subscribeElapsed((e) => { elapsedMs = e; })();

  let auraCanvas: HTMLCanvasElement | null = null;
  let cometCanvas: HTMLCanvasElement | null = null;

  function drawAura() {
    if (!auraCanvas) return;
    const ctx = auraCanvas.getContext('2d');
    if (!ctx) return;
    const rect = auraCanvas.getBoundingClientRect();
    auraCanvas.width = rect.width;
    auraCanvas.height = rect.height;
    
    const gradient = ctx.createRadialGradient(
      cursorX, cursorY, 0,
      cursorX, cursorY, 300
    );
    gradient.addColorStop(0, config.auraColor + '40');
    gradient.addColorStop(1, 'transparent');
    
    ctx.fillStyle = gradient;
    ctx.fillRect(0, 0, auraCanvas.width, auraCanvas.height);
    requestAnimationFrame(drawAura);
  }

  function drawComet() {
    if (!cometCanvas) return;
    const ctx = cometCanvas.getContext('2d');
    if (!ctx) return;
    const rect = cometCanvas.getBoundingClientRect();
    cometCanvas.width = rect.width;
    cometCanvas.height = rect.height;
    
    ctx.clearRect(0, 0, cometCanvas.width, cometCanvas.height);
    ctx.strokeStyle = config.cometColor;
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(cursorX - 50, cursorY - 50);
    ctx.lineTo(cursorX, cursorY);
    ctx.stroke();
    requestAnimationFrame(drawComet);
  }

  $effect(() => {
    drawAura();
    drawComet();
  });
</script>

<div class="overlay-window">
  <canvas bind:this={auraCanvas} class="aura-canvas"></canvas>
  <canvas bind:this={cometCanvas} class="comet-canvas"></canvas>
  
  <div class="hud">
    <div class="hud-step">Step {step} / {maxSteps}</div>
    {#if thought}
      <div class="hud-thought">💭 {thought}</div>
    {/if}
    {#if action}
      <div class="hud-action">⚡ {action}</div>
    {/if}
    <div class="hud-time">{elapsedMs}ms</div>
  </div>
</div>

<style>
  .overlay-window {
    position: fixed;
    inset: 0;
    pointer-events: none;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: flex-start;
    padding: 20px;
  }
  
  .aura-canvas,
  .comet-canvas {
    position: fixed;
    inset: 0;
    width: 100vw;
    height: 100vh;
    z-index: 0;
  }
  
  .hud {
    position: fixed;
    top: 20px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 10;
    background: rgba(20, 20, 20, 0.9);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 12px 20px;
    color: var(--fg);
    font-size: 13px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 200px;
  }
  
  .hud-step {
    font-weight: 600;
    color: var(--accent);
  }
  
  .hud-thought {
    color: var(--fg-muted);
    font-style: italic;
  }
  
  .hud-action {
    color: var(--fg);
    font-family: var(--font-mono);
  }
  
  .hud-time {
    color: var(--fg-muted);
    font-size: 11px;
  }
</style>
