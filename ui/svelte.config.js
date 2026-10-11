/** @type {import('@sveltejs/kit').Config} */
import adapter from '@sveltejs/adapter-auto';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

const config = {
  // Consult https://github.com/sveltejs/kit/docs for documentation
  // about these options

  // Options
  kit: {
    // adapter-auto only supports some environments, see https://kit.svelte.dev/docs/adapter-auto for a list.
    adapter: adapter()
  },

  preprocess: vitePreprocess()
};

export default config;
