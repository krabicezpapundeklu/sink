import { BATCH_SIZE, loadItems } from '$lib/shared';
import type { PageLoad } from './$types';

export const load = (async ({ url }) => {
	return await loadItems(url.searchParams, 0, Number.MAX_SAFE_INTEGER, BATCH_SIZE, true);
}) satisfies PageLoad;
