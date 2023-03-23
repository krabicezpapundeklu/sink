import { loadItem } from '$lib/utils';
import type { PageServerLoad } from './$types';

export const load = (async ({ fetch, params }) => {
	return await loadItem(fetch, +params.id);
}) satisfies PageServerLoad;
