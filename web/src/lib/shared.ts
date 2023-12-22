import ITEM_TYPES from '../../../item.types.json';

import hljs from 'highlight.js/lib/core';
import json from 'highlight.js/lib/languages/json';
import plaintext from 'highlight.js/lib/languages/plaintext';
import xml from 'highlight.js/lib/languages/xml';

import type { Item, ItemSearchResult, ItemSummary, ItemType } from './model';
import { error } from '@sveltejs/kit';

export const BATCH_SIZE = 50;
export const MILLISECONDS_IN_DAY = 24 * 60 * 60 * 1000;
export const MILLISECONDS_IN_HOUR = 60 * 60 * 1000;
export const MILLISECONDS_IN_MINUTE = 60 * 1000;

export { ITEM_TYPES };

ITEM_TYPES.sort((x, y) => x.name.localeCompare(y.name));

hljs.registerLanguage('json', json);
hljs.registerLanguage('plaintext', plaintext);
hljs.registerLanguage('xml', xml);

function dateToString(
	year: number,
	month: number,
	day: number,
	hours: number,
	minutes: number,
	seconds: number
): string {
	return (
		`${year}-${pad(month + 1)}-${pad(day)} ${pad(hours)}:${pad(minutes)}` +
		(seconds === 0 ? '' : `:${pad(seconds)}`)
	);
}

export function formatBody(item: Item): string {
	for (const header of item.headers) {
		if (header.name === 'content-type') {
			if (header.value.indexOf('json') !== -1) {
				return formatJson(item.body);
			} else if (header.value.indexOf('xml') !== -1) {
				return formatXml(item.body);
			}

			break;
		}
	}

	return item.body;
}

function formatJson(json: string): string {
	let formatted: string;

	try {
		formatted = JSON.stringify(JSON.parse(json), null, 1);
	} catch (e) {
		formatted = json;
	}

	return formatted;
}

export function formatNumber(value: number): string {
	return value.toLocaleString('en-us');
}

function formatSubmitDates(items: ItemSummary[], detail = false) {
	const options: Intl.DateTimeFormatOptions = {};
	const now = new Date();

	let dayDtf: Intl.DateTimeFormat | undefined;
	let todayDtf: Intl.DateTimeFormat | undefined;
	let nowDate: string | undefined;

	if (detail) {
		options.dateStyle = 'full';
		options.timeStyle = 'medium';
	} else {
		options.timeStyle = 'short';

		todayDtf = new Intl.DateTimeFormat('en-us', options);

		options.dateStyle = 'short';

		const dayOpts: Intl.DateTimeFormatOptions = { dateStyle: 'short', timeZone: options.timeZone };

		dayDtf = new Intl.DateTimeFormat('en-us', dayOpts);
		nowDate = dayDtf.format(now);
	}

	const defaultDtf = new Intl.DateTimeFormat('en-us', options);

	for (const item of items) {
		let dtf = defaultDtf;

		const isoDate =
			item.submitDate.substring(0, 10) + 'T' + item.submitDate.substring(11) + '.000Z';

		const submitDate = new Date(isoDate);

		if (!detail) {
			if (now.getTime() - submitDate.getTime() < MILLISECONDS_IN_DAY) {
				// eslint-disable-next-line @typescript-eslint/no-non-null-assertion
				const sd = dayDtf!.format(submitDate);

				if (sd === nowDate) {
					// eslint-disable-next-line @typescript-eslint/no-non-null-assertion
					dtf = todayDtf!;
				}
			}
		}

		item.submitDate = dtf.format(submitDate);
	}
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

export function highlightElement(e: HTMLElement): void {
	hljs.highlightElement(e);
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
	const response = await fetch(`/api/item/${itemId}`);

	if (!response.ok) {
		error(500, await response.text());
	}

	const item = await response.json();

	formatSubmitDates([item], true);

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

	const response = await fetch(`/api/items${url}`);

	if (!response.ok) {
		error(500, await response.text());
	}

	const items = await response.json();

	formatSubmitDates(items.items);

	if (items.firstItem) {
		formatSubmitDates([items.firstItem], true);
	}

	return items;
}

export function localDateToString(date: Date): string {
	return dateToString(
		date.getFullYear(),
		date.getMonth(),
		date.getDate(),
		date.getHours(),
		date.getMinutes(),
		date.getSeconds()
	);
}

function pad(value: number): string {
	return (value > 9 ? '' : '0') + value;
}

export function utcDateStringToLocalString(value: string | null): string {
	if (!value) {
		return '';
	}

	const date = new Date(utcDateStringToMs(value));

	return localDateToString(date);
}

function utcDateStringToMs(value: string): number {
	const year = +value.substring(0, 4);
	const month = +value.substring(5, 7);
	const day = +value.substring(8, 10);
	const hours = +value.substring(11, 13);
	const minutes = +value.substring(14, 16);
	const seconds = value.length === 19 ? +value.substring(17, 19) : 0;

	return Date.UTC(year, month - 1, day, hours, minutes, seconds);
}

export function utcDateToString(date: Date): string {
	return dateToString(
		date.getUTCFullYear(),
		date.getUTCMonth(),
		date.getUTCDate(),
		date.getUTCHours(),
		date.getUTCMinutes(),
		date.getSeconds()
	);
}
