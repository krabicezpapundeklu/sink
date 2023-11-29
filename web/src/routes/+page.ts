import { BATCH_SIZE, loadItems } from '$lib/shared';
import type { PageLoad } from './$types';

export const load = (async ({ fetch, url }) => {
	return await loadItems(fetch, url.searchParams, undefined, undefined, BATCH_SIZE + 1, true);
}) satisfies PageLoad;
