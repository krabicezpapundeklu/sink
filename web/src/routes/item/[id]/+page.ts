import { building } from '$app/environment';
import { loadItem } from '$lib/shared';
import type { Item } from '$lib/model';
import type { PageLoad } from './$types';

export const load = (async ({ fetch, params }) => {
	if (building) {
		const result: Item = {
			headers: [],
			body: '',
			id: 0,
			submitDate: ''
		};

		return result;
	}

	return await loadItem(fetch, +params.id);
}) satisfies PageLoad;
