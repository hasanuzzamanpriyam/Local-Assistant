import { writable } from 'svelte/store';

export interface OverlayConfig {
  auraColor: string;
  cometColor: string;
  hudPosition: string;
}

export interface CursorPosition {
  x: number;
  y: number;
}

const isVisible = writable<boolean>(false);
const cursorPosition = writable<CursorPosition>({ x: 0, y: 0 });
const currentStep = writable<number>(0);
const maxSteps = writable<number>(10);
const thought = writable<string>('');
const action = writable<string>('');
const elapsedMs = writable<number>(0);

function createOverlayStore() {
  return {
    subscribe: isVisible.subscribe,
    subscribeCursor: cursorPosition.subscribe,
    subscribeStep: currentStep.subscribe,
    subscribeMaxSteps: maxSteps.subscribe,
    subscribeThought: thought.subscribe,
    subscribeAction: action.subscribe,
    subscribeElapsed: elapsedMs.subscribe,

    setCursor(x: number, y: number) {
      cursorPosition.set({ x, y });
    },

    show() {
      isVisible.set(true);
    },

    hide() {
      isVisible.set(false);
    },

    update(step: number, maxStepsVal: number, thoughtVal: string, actionVal: string, elapsed: number) {
      currentStep.set(step);
      maxSteps.set(maxStepsVal);
      thought.set(thoughtVal);
      action.set(actionVal);
      elapsedMs.set(elapsed);
      isVisible.set(true);
    },

    get config(): OverlayConfig {
      return {
        auraColor: '#007AFF',
        cometColor: '#00D4FF',
        hudPosition: 'top-center',
      };
    },
  };
}

export const overlayStore = createOverlayStore();
