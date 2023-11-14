import { loadItem } from '$lib/shared';
import type { PageLoad } from './$types';

export const load = (async ({ params }) => {
	return await loadItem(+params.id);
}) satisfies PageLoad;
