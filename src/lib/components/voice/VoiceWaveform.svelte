<script lang="ts">
  import { voiceStore } from '../../stores/voice';

  let audioLevel = $derived(voiceStore.audioLevel);
  let canvas: HTMLCanvasElement | null = null;

  $effect(() => {
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;
    const rect = canvas.getBoundingClientRect();
    canvas.width = rect.width;
    canvas.height = rect.height;

    ctx.clearRect(0, 0, canvas.width, canvas.height);
    const barCount = 20;
    const barWidth = 4;
    const barGap = 2;
    const totalWidth = barCount * barWidth + (barCount - 1) * barGap;
    const startX = (canvas.width - totalWidth) / 2;

    for (let i = 0; i < barCount; i++) {
      const height = 4 + Math.random() * (canvas.height - 8) * audioLevel;
      const x = startX + i * (barWidth + barGap);
      const y = canvas.height - height;
      ctx.fillStyle = audioLevel > 0.5 ? var(--accent) : var(--fg-muted);
      ctx.fillRect(x, y, barWidth, height);
    }

    if (audioLevel > 0) {
      requestAnimationFrame(() => {});
    }
  });
</script>

<canvas bind:this={canvas} class="waveform"></canvas>

<style>
  .waveform {
    width: 100%;
    height: 40px;
    display: block;
  }
</style>
