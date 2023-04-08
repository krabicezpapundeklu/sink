import { BATCH_SIZE, highlightItem, loadItem, loadItems } from '$lib/shared';

import type { ItemWithHighlighting } from '$lib/model';
import type { PageLoad } from './$types';

export const load = (async ({ fetch, url }) => {
	const data = await loadItems(fetch, url.searchParams, 0, Number.MAX_SAFE_INTEGER, BATCH_SIZE);

	let firstItem: ItemWithHighlighting | null;

	if (data.items.length > 0) {
		firstItem = highlightItem(await loadItem(fetch, data.items[0].id));
	} else {
		firstItem = null;
	}

	return {
		...data,
		firstItem
	};
}) satisfies PageLoad;
