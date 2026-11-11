/// <reference types="vite/client" />
/// <reference types="@tauri-apps/api" />

declare module '*.svelte' {
  import type { SvelteComponent } from 'svelte';
  const component: new (options: { target: HTMLElement; props?: Record<string, any> }) => SvelteComponent;
  export default component;
}