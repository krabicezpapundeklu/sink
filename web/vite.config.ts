import { sveltekit } from '@sveltejs/kit/vite';
import path from 'path';

import type { UserConfig } from 'vite';

const config: UserConfig = {
	plugins: [sveltekit()],
	resolve: {
		alias: {
			'~bootstrap': path.resolve(__dirname, 'node_modules/bootstrap')
		}
	},
	server: {
		proxy: {
			'/api': 'http://localhost:8080'
		}
	}
};

export default config;
