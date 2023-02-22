import type { Item, ItemSearchResult } from '$lib/model';

const get = async (url: string): Promise<any> => {
	const response = await fetch(url);

	if (response.ok) {
		return await response.json();
	}

	const text = await response.text();

	alert(`HTTP ${response.status} ${response.statusText}\n${text}`);

	return Promise.reject();
};

export const getItem = async (id: number): Promise<Item> => {
	return get(`/api/item/${id}`);
};

export const getItems = async (
	params: URLSearchParams,
	batchSize: number,
	nextItemId: number
): Promise<ItemSearchResult> => {
	return get(`/api/items?${params}&batchSize=${batchSize}&nextItemId=${nextItemId}`);
};
