<script lang="ts">
  import { sidecarStore } from '../../stores/sidecar';
  import { overlayStore } from '../../stores/overlay';
  import { overlayShow, overlayHide } from '../../api/overlay';

  let { taskId = '' } = $props();
  let unlisten: (() => void) | null = null;

  async function startListening() {
    unlisten = await sidecarStore.listen((event) => {
      if (event.id !== taskId) return;
      
      switch (event.event) {
        case 'task_started':
          overlayStore.show();
          overlayShow(event.cursorX ?? 0, event.cursorY ?? 0).catch(console.error);
          break;
        case 'step':
          overlayStore.update(
            event.step ?? 0,
            event.max_steps ?? 10,
            event.thought ?? '',
            event.action ?? '',
            Date.now()
          );
          break;
        case 'task_complete':
        case 'task_stopped':
        case 'error':
          overlayStore.hide();
          overlayHide().catch(console.error);
          break;
      }
    });
  }

  $effect(() => {
    startListening();
    return () => {
      unlisten?.();
    };
  });
</script>

<!-- Controller is headless; it just coordinates overlay state -->
