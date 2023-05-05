import { error, type HandleFetch } from '@sveltejs/kit';

declare function fetchData(
	path: string,
	search: string,
	tz: string
): { data?: string; error?: string };

declare const TIME_ZONE: string | null | undefined;

export const handleFetch = (async ({ fetch, request }) => {
	if (typeof fetchData === 'undefined') {
		return fetch(request);
	}

	let tz = '';

	if (typeof TIME_ZONE !== 'undefined' && TIME_ZONE) {
		tz = TIME_ZONE;
	}

	const url = new URL(request.url);
	const result = await fetchData(url.pathname, url.search, tz);

	if (result.data) {
		return Promise.resolve(
			new Response(result.data, {
				status: 200,
				statusText: 'OK'
			})
		);
	} else {
		throw error(500, result.error);
	}
}) satisfies HandleFetch;
