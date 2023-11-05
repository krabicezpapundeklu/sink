export interface Item extends ItemSummary {
	headers: ItemHeader[];
	body: string;
}

export interface ItemHeader {
	name: string;
	value: string;
}

export interface ItemSearchResult {
	items: ItemSummary[];
	systems: string[];
	totalItems: number;
	firstItem: Item | null;
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
