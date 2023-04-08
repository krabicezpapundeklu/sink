import { browser } from '$app/environment';

import hljs from 'highlight.js/lib/core';
import json from 'highlight.js/lib/languages/json';
import plaintext from 'highlight.js/lib/languages/plaintext';
import xml from 'highlight.js/lib/languages/xml';

import type { Item, ItemSearchResult, ItemType, ItemWithHighlighting } from './model';

declare const TIME_ZONE: string | null | undefined;

export const BATCH_SIZE = 100;
export const MILLISECONDS_IN_DAY = 24 * 60 * 60 * 1000;
export const MILLISECONDS_IN_HOUR = 60 * 60 * 1000;
export const MILLISECONDS_IN_MINUTE = 60 * 1000;

const NUMBER_FORMAT = new Intl.NumberFormat('en-US');

export const ITEM_TYPES: ItemType[] = [
	{ name: 'Event Notification', key: 'event_notification' },
	{ name: 'Event Payload', key: 'event_payload' },
	{ name: 'PC Folder CL', key: 'folder_cl' },
	{ name: 'PC Folder FS', key: 'folder_fs' },
	{ name: 'PC Folder IDPD', key: 'folder_idpd' },
	{ name: 'PC Folder SINGLEPD', key: 'folder_pd' },
	{ name: 'SOAP Application Created', key: 'application_created' },
	{ name: 'SOAP Application Updated', key: 'application_updated' },
	{ name: 'SOAP Certificate Created', key: 'certificate_created' },
	{ name: 'SOAP Certificate Updated', key: 'certificate_updated' },
	{ name: 'SOAP Selectee Created', key: 'selectee_created' },
	{ name: 'SOAP Selectee Updated', key: 'selectee_updated' },
	{ name: 'SOAP Status Updated', key: 'status_updated' },
	{ name: 'SOAP Vacancy Created', key: 'vacancy_created' },
	{ name: 'SOAP Vacancy Updated', key: 'vacancy_updated' }
];

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

function formatJson(json: string): string {
	try {
		return JSON.stringify(JSON.parse(json), null, 1);
	} catch (e) {
		return json;
	}
}

export function formatNumber(value: number): string {
	return NUMBER_FORMAT.format(value);
}

export function formatSubmitDate(value: string, detail = false): string {
	const options: Intl.DateTimeFormatOptions = {};

	if (!browser) {
		if (typeof TIME_ZONE !== 'undefined' && TIME_ZONE) {
			options.timeZone = TIME_ZONE;
		} else {
			return '';
		}
	}

	const isoDate = value.substring(0, 10) + 'T' + value.substring(11) + '.000Z';
	const submitDate = new Date(isoDate);

	if (detail) {
		options.dateStyle = 'full';
		options.timeStyle = 'medium';
	} else {
		const now = new Date();

		options.timeStyle = 'short';

		if (
			now.getDate() !== submitDate.getDate() ||
			now.getMonth() !== submitDate.getMonth() ||
			now.getFullYear() !== submitDate.getFullYear()
		) {
			options.dateStyle = 'short';
		}
	}

	return new Intl.DateTimeFormat('en-us', options).format(submitDate);
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

	return formatted.substring(1, formatted.length - 3);
}

export function highlightItem(item: Item): ItemWithHighlighting {
	let language = 'plaintext';
	let formattedBody = item.body;

	for (const header of item.headers) {
		if (header.name === 'content-type') {
			if (header.value.indexOf('json') !== -1) {
				language = 'json';
				formattedBody = formatJson(item.body);
			} else if (header.value.indexOf('xml') !== -1) {
				language = 'xml';
				formattedBody = formatXml(item.body);
			}

			break;
		}
	}

	const higlightedBody = hljs.highlight(item.body, { language }).value;
	const highlightedBodyPreview = hljs.highlight(formattedBody, { language }).value;

	return {
		...item,
		higlightedBody,
		highlightedBodyPreview
	};
}

export function itemTypeToName(key: string): string {
	for (const type of ITEM_TYPES) {
		if (type.key === key) {
			return type.name;
		}
	}

	return '';
}

export async function loadItem(
	fetch: (input: RequestInfo) => Promise<Response>,
	itemId: number
): Promise<Item> {
	const response = await fetch(`/api/item/${itemId}`);
	const item = await response.json();

	return item;
}

export async function loadItems(
	fetch: (input: RequestInfo) => Promise<Response>,
	params: URLSearchParams,
	firstItemId: number,
	lastItemId: number,
	batchSize?: number
): Promise<ItemSearchResult> {
	let url = `/api/items?firstItemId=${firstItemId}&lastItemId=${lastItemId}`;

	if (batchSize) {
		url += `&batchSize=${batchSize}`;
	}

	url += `&${params}`;

	const response = await fetch(url);
	const items = await response.json();

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
