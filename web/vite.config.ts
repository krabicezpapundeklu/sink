import { isoImport } from 'vite-plugin-iso-import';
import { sveltekit } from '@sveltejs/kit/vite';
import type { UserConfig } from 'vite';

const config: UserConfig = {
	build: {
		minify: 'terser',
		sourcemap: true,
		terserOptions: {
			compress: {
				drop_console: true,
				hoist_funs: true,
				passes: 2,
				unsafe: true
			}
		}
	},
	envPrefix: 'CARGO_',
	plugins: [isoImport(), sveltekit()],
	server: {
		proxy: {
			'/sink/api': 'http://127.0.0.1:8080'
		}
	}
};

export default config;
