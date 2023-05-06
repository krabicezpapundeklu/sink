import './polyfills';

import { manifest } from './manifest.js';
import { Server } from './index.js';

const server = new Server(manifest);

export async function render(path) {
	await server.init({ env: {} });

	const response = await server.respond(new Request(path));
	const body = await response.text();

	return body;
}
