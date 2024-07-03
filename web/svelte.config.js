import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	compilerOptions: {
		runes: true
	},
	preprocess: vitePreprocess(),
	kit: {
		adapter: adapter({
			fallback: 'fallback.html'
		}),
		paths: {
			base: '/sink'
		},
		prerender: {
			entries: ['/', '/item/0']
		}
	}
};

export default config;
