import './app.css';
import '@fontsource-variable/dm-sans';
import App from './App.svelte';

const app = new App({
  target: document.getElementById('svelte') || document.body,
});

export default app;
