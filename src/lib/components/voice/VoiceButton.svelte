<script lang="ts">
  import { voiceStore } from '../../stores/voice';
  import { stt } from '../../api/voice';

  let isPressed = $state(false);
  let mediaRecorder: MediaRecorder | null = null;
  let audioChunks: Blob[] = [];

  let isListening = $derived(voiceStore.isListening);
  let sttStatus = $derived(voiceStore.sttStatus);

  let { onTranscript }: { onTranscript?: (text: string) => void } = $props();

  async function startRecording() {
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      mediaRecorder = new MediaRecorder(stream, { mimeType: 'audio/webm' });
      audioChunks = [];

      mediaRecorder.ondataavailable = (e) => {
        if (e.data.size > 0) audioChunks.push(e.data);
      };

      mediaRecorder.onstop = async () => {
        stream.getTracks().forEach((t) => t.stop());
        const blob = new Blob(audioChunks, { type: 'audio/webm' });
        const arrayBuffer = await blob.arrayBuffer();
        const base64 = btoa(
          String.fromCharCode(...new Uint8Array(arrayBuffer))
        );

        voiceStore.setSttStatus('processing');
        try {
          const result = await stt(base64, 'webm');
          voiceStore.setTranscript(result.text);
          onTranscript?.(result.text);
          voiceStore.setSttStatus('ready');
        } catch (e) {
          console.error('STT failed:', e);
          voiceStore.setSttStatus('error');
        }
      };

      mediaRecorder.start(100);
      voiceStore.startListening();
    } catch (e) {
      console.error('Microphone access failed:', e);
      voiceStore.setSttStatus('error');
    }
  }

  async function stopRecording() {
    if (mediaRecorder && mediaRecorder.state !== 'inactive') {
      mediaRecorder.stop();
    }
    voiceStore.stopListening();
    isPressed = false;
  }

  function handlePointerDown() {
    isPressed = true;
    startRecording();
  }

  function handlePointerUp() {
    stopRecording();
  }
</script>

<button
  class="voice-button"
  class:recording={isPressed}
  class:processing={sttStatus === 'processing'}
  onpointerdown={handlePointerDown}
  onpointerup={handlePointerUp}
  onpointerleave={handlePointerUp}
  aria-label="Push to talk"
  title="Hold to speak"
>
  {#if sttStatus === 'processing'}
    <span class="button-icon">⏳</span>
    <span class="button-label">Transcribing...</span>
  {:else if isPressed}
    <span class="button-icon">🎙️</span>
    <span class="button-label">Listening...</span>
  {:else}
    <span class="button-icon">🎤</span>
    <span class="button-label">Hold to talk</span>
  {/if}
</button>

<style>
  .voice-button {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 24px;
    border: none;
    border-radius: 50px;
    background: var(--bg-elevated);
    color: var(--fg);
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s;
  }

  .voice-button:hover {
    background: var(--bg-hover);
  }

  .voice-button.recording {
    background: #ff4444;
    color: white;
    animation: pulse 1.5s infinite;
  }

  .voice-button.processing {
    background: var(--accent);
    color: white;
  }

  .voice-button:active {
    transform: scale(0.95);
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.7; }
  }

  .button-icon {
    font-size: 20px;
  }

  .button-label {
    font-weight: 500;
  }
</style>
