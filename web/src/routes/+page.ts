import { BATCH_SIZE, loadItems } from '$lib/shared';
import type { PageLoad } from './$types';

export const load = (async ({ fetch, url }) => {
	return await loadItems(fetch, url.searchParams, 0, Number.MAX_SAFE_INTEGER, BATCH_SIZE);
}) satisfies PageLoad;