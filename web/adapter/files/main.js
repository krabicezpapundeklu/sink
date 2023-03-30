import './polyfills';
import { manifest } from './manifest.js';

import {
	compact,
	default_filter,
	default_preload,
	default_transform,
	get_option,
	options,
	render_response,
	set_public_env
} from './index.js';

export async function render_route(path, data) {
	let page;

	for (const route of manifest._.routes) {
		if (route.pattern.exec(path)) {
			page = route.page;
			break;
		}
	}

	if (!page) {
		throw new Error(`no route for ${path}`);
	}

	const nodes = await Promise.all([
		...page.layouts.map((n) => (n == void 0 ? n : manifest._.nodes[n]())),
		manifest._.nodes[page.leaf]()
	]);

	const resolve_opts = {
		transformPageChunk: default_transform,
		filterSerializedResponseHeaders: default_filter,
		preload: default_preload
	};

	const state = {
		error: false,
		depth: 0,
		prerender_default: get_option(nodes, 'prerender') ?? false
	};

	let fetched = [];

	for (const datum of data) {
		if (datum) {
			fetched.push({
				url: datum.url,
				response_body: JSON.stringify(datum.data),
				method: 'GET',
				response: {
					headers: [],
					status: 200,
					statusText: 'OK'
				}
			});
		}
	}

	let branch = [];

	for (let i = 0; i < nodes.length; ++i) {
		const node = nodes[i];
		branch.push({ node, server_data: null, data: data[i] ? data[i].data : null });
	}

	const response = await render_response({
		event: {
			url: {
				pathname: path
			}
		},
		options,
		manifest,
		state,
		resolve_opts,
		page_config: {
			csr: get_option(nodes, 'csr') ?? true,
			ssr: true
		},
		status: 200,
		error: null,
		branch: compact(branch),
		undefined,
		fetched
	});

	return await response.text();
}

export { set_public_env };
