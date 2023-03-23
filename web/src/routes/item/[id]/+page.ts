import { loadItem } from '$lib/shared';
import type { PageLoad } from './$types';

export const load = (async ({ fetch, params }) => {
	return await loadItem(fetch, +params.id);
}) satisfies PageLoad;
