import type { Item, ItemSearchResult } from '$lib/model';

async function get<T>(url: string): Promise<T> {
	const response = await fetch(url);

	if (response.ok) {
		return await response.json();
	}

	const text = await response.text();

	alert(`HTTP ${response.status} ${response.statusText}\n${text}`);

	return Promise.reject();
}

export async function getItem(id: number): Promise<Item> {
	return get(`/api/item/${id}`);
}

export async function getItems(
	params: URLSearchParams,
	firsItemId: number,
	lastItemId: number,
	batchSize?: number
): Promise<ItemSearchResult> {
	let url = `/api/items?${params}&firstItemId=${firsItemId}&lastItemId=${lastItemId}`;

	if (batchSize) {
		url += `&batchSize=${batchSize}`;
	}

	return get(url);
}
