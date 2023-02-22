export interface Item {
	id: number;
	submitDate: string;
	system?: string;
	type?: string;
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
}

export interface ItemSummary {
	id: number;
	submitDate: string;
	system?: string;
	type?: string;
}
