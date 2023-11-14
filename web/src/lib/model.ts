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
	totalItems: number;
	systems: string[];
	firstItem?: Item;
}

export interface ItemSummary {
	id: number;
	submitDate: string;
	system?: string;
	type?: string;
}

export interface ItemType {
	name: string;
	key: string;
	color: string;
}

export interface ItemWithHighlighting extends Item {
	higlightedBody: string;
	highlightedBodyPreview: string;
}
