import { error, type HandleFetch } from '@sveltejs/kit';

declare function fetchData(path: string): { data?: string; error?: string };

declare const TIME_ZONE: string | null | undefined;

export const handleFetch = (async ({ fetch, request }) => {
	if (typeof fetchData === 'undefined') {
		return fetch(request);
	}

	const url = new URL(request.url);

	if (typeof TIME_ZONE !== 'undefined' && TIME_ZONE) {
		url.searchParams.append('tz', TIME_ZONE);
	}

	const result = await fetchData(`${url.pathname}${url.search}`);

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
