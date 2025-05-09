import { base } from '$app/paths';
import { error } from '@sveltejs/kit';
import EVENT_TYPES from '../../../event.types.json';
import ITEM_TYPES from '../../../item.types.json';
import type { Item, ItemSearchResult, ItemSummary, ItemType } from './model';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
declare let initialData: any;

export const BATCH_SIZE = 50;

export { ITEM_TYPES };

ITEM_TYPES.sort((x, y) => x.name.localeCompare(y.name));

export function formatBody(item: Item): { body: string; language: string } {
	for (const header of item.headers) {
		if (header.name === 'content-type') {
			if (header.value.indexOf('json') !== -1) {
				return { body: formatJson(item.body), language: 'json5' };
			} else if (header.value.indexOf('xml') !== -1) {
				return { body: formatXml(item.body), language: 'markup' };
			}

			break;
		}
	}

	return { body: item.body, language: 'plain' };
}

function formatJson(json: string): string {
	let formatted: string;

	try {
		formatted = JSON.stringify(JSON.parse(json), null, 1);
		// eslint-disable-next-line @typescript-eslint/no-unused-vars
	} catch (e) {
		formatted = json;
	}

	return formatted;
}

export function formatNumber(value: number): string {
	return value.toLocaleString('en-us');
}

function formatXml(xml: string): string {
	let formatted = '';
	let indent = '';

	xml.split(/>\s*</).forEach((node: string): void => {
		if (node.match(/^\/\w/)) {
			indent = indent.substring(1);
		}

		formatted += indent + '<' + node + '>\r\n';

		if (node.match(/^<?\w[^>]*[^/]$/)) {
			indent += ' ';
		}
	});

	formatted = formatted.substring(1, formatted.length - 3);

	return formatted;
}

export function getEntityEventType(id?: number): string {
	if (id) {
		for (const group of EVENT_TYPES) {
			for (const type of group.types) {
				if (type.id === id) {
					return `${id} - ${group.name} / ${type.name}`;
				}
			}
		}
	}

	return '';
}

function getInitialDataOnce() {
	let data = null;

	if (initialData) {
		if (initialData !== '%INITIAL_DATA%') {
			data = initialData;
		}

		initialData = null;
	}

	return data;
}

export function getUserAgent(item: ItemSummary): string {
	const userAgent = item.userAgent;

	if (userAgent) {
		if (userAgent.startsWith('AmazonAPIGateway')) {
			return 'API';
		}

		if (userAgent.startsWith('insomnia')) {
			return 'Insomnia';
		}

		if (userAgent.startsWith('PostmanRuntime')) {
			return 'Postman';
		}

		if (userAgent.startsWith('Synapse-')) {
			return 'WSO';
		}
	}

	return '';
}

export function itemTypeFromKey(key: string): ItemType {
	for (const type of ITEM_TYPES) {
		if (type.key === key) {
			return type;
		}
	}

	return { name: key, key, color: 'red' };
}

export async function loadItem(
	fetch: (input: RequestInfo) => Promise<Response>,
	itemId: number
): Promise<Item> {
	let item = getInitialDataOnce();

	if (!item) {
		const response = await fetch(`${base}/api/item/${itemId}`);

		if (!response.ok) {
			error(500, await response.text());
		}

		item = await response.json();
	}

	return item;
}

export async function loadItems(
	fetch: (input: RequestInfo) => Promise<Response>,
	params: URLSearchParams,
	firstItemId?: number,
	lastItemId?: number,
	batchSize?: number,
	loadFirstItem?: boolean
): Promise<ItemSearchResult> {
	let items = getInitialDataOnce();

	if (!items) {
		let url = '';

		if (firstItemId) {
			url += `&firstItemId=${firstItemId}`;
		}

		if (lastItemId) {
			url += `&lastItemId=${lastItemId}`;
		}

		if (batchSize) {
			url += `&batchSize=${batchSize}`;
		}

		if (loadFirstItem) {
			url += '&loadFirstItem=true';
		}

		if (params.size > 0) {
			url += `&${params}`;
		}

		if (url.length > 0) {
			url = '?' + url.substring(1);
		}

		const response = await fetch(`${base}/api/items${url}`);

		if (!response.ok) {
			error(500, await response.text());
		}

		items = await response.json();
	}

	return items;
}
