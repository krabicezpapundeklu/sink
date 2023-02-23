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
	batchSize: number,
	nextItemId: number
): Promise<ItemSearchResult> {
	return get(`/api/items?${params}&batchSize=${batchSize}&nextItemId=${nextItemId}`);
}
