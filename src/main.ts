import { mount } from 'svelte';
import App from './App.svelte';

const app = mount(App, {
  target: document.getElementById('app')!, // Ensure your index.html has a div with id="app"
});

export default app;