<script lang="ts">
  import { voiceStore } from '../../stores/voice';

  let isListening = $derived(voiceStore.isListening);
  let isSpeaking = $derived(voiceStore.isSpeaking);
  let sttStatus = $derived(voiceStore.sttStatus);
  let ttsStatus = $derived(voiceStore.ttsStatus);
</script>

<div class="voice-status">
  {#if isListening || sttStatus === 'processing'}
    <div class="status-item status-listening">
      <span class="dot"></span>
      <span class="label">Listening...</span>
    </div>
  {:else if isSpeaking || ttsStatus === 'speaking'}
    <div class="status-item status-speaking">
      <span class="dot"></span>
      <span class="label">Speaking...</span>
    </div>
  {:else}
    <div class="status-item status-idle">
      <span class="dot"></span>
      <span class="label">Ready</span>
    </div>
  {/if}
</div>

<style>
  .voice-status {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
  }

  .status-item {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--fg-muted);
  }

  .status-listening .dot {
    background: #ff4444;
    animation: pulse 1.5s infinite;
  }

  .status-speaking .dot {
    background: var(--accent);
    animation: pulse 1.5s infinite;
  }

  .status-idle .dot {
    background: #00c851;
  }

  .label {
    color: var(--fg-muted);
    font-size: 12px;
  }

  .status-listening .label {
    color: #ff4444;
    font-weight: 500;
  }

  .status-speaking .label {
    color: var(--accent);
    font-weight: 500;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }
</style>
