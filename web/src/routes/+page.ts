import { BATCH_SIZE, loadItems } from '$lib/shared';
import { building } from '$app/environment';
import type { ItemSearchResult } from '$lib/model';
import type { PageLoad } from './$types';

export const load = (async ({ fetch, url }) => {
	if (building) {
		const result: ItemSearchResult = {
			items: [],
			totalItems: 0,
			systems: []
		};

		return result;
	}

	return await loadItems(fetch, url.searchParams, undefined, undefined, BATCH_SIZE + 1, true);
}) satisfies PageLoad;
