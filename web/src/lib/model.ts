export interface Item extends ItemSummary {
	headers: ItemHeader[];
	body: string;
}

export interface ItemFilter {
	query: string | null;
	system: string | null;
	type: string | null;
	from: string | null;
	to: string | null;
	asc: boolean | null;
	firstItemId: number | null;
	lastItemId: number | null;
	batchSize: number | null;
}

export interface ItemHeader {
	name: string;
	value: string;
}

export interface ItemSearchResult {
	items: ItemSummary[];
	systems: string[];
	totalItems: number;
	filter: ItemFilter;
}

export interface ItemSummary {
	id: number;
	submitDate: string;
	system: string | null;
	type: string | null;
}

export interface ItemType {
	name: string;
	key: string;
}

export interface ItemWithHighlighting extends Item {
	higlightedBody: string;
	highlightedBodyPreview: string;
}
