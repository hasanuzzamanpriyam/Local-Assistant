<script lang="ts">
  let { src } = $props();
</script>

<div class="screenshot-viewer">
  {#if src}
    <img src={`data:image/png;base64,${src}`} alt="Agent screenshot" />
  {/if}
</div>

<style>
  .screenshot-viewer {
    width: 100%;
    max-width: 400px;
    margin-top: 8px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    overflow: hidden;
    background: var(--bg);
  }
  .screenshot-viewer img {
    width: 100%;
    height: auto;
    display: block;
  }
</style>
