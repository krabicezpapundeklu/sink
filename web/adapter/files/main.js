import './polyfills';

import { manifest } from './manifest.js';
import { Server } from './index.js';

const server = new Server(manifest);

export async function init(version) {
	await server.init({ env: { PUBLIC_VERSION: version } });
}

export async function render(path) {
	const response = await server.respond(new Request(path));
	const body = await response.text();

	return body;
}
