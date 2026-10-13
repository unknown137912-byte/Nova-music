import './app.css';
import './lib/reference-ui.css';
import '@fontsource-variable/dm-sans';
import App from './App.svelte';

const app = new App({
  target: document.getElementById('svelte') || document.body,
});

export default app;
