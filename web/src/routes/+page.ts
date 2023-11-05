import { BATCH_SIZE, highlightItem, loadItems } from '$lib/shared';

import type { ItemWithHighlighting } from '$lib/model';
import type { PageLoad } from './$types';

export const load = (async ({ fetch, url }) => {
	const items = await loadItems(
		fetch,
		url.searchParams,
		0,
		Number.MAX_SAFE_INTEGER,
		BATCH_SIZE,
		true
	);

	let firstItem: ItemWithHighlighting | null;

	if (items.firstItem) {
		firstItem = highlightItem(items.firstItem);
	} else {
		firstItem = null;
	}

	return {
		...items,
		firstItem
	};
}) satisfies PageLoad;
