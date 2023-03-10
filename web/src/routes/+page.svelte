<script lang="ts">
	import { afterNavigate, goto } from '$app/navigation';
	import { getItems } from '$lib/api';

	import {
		itemTypeToName,
		MILLISECONDS_IN_MINUTE,
		utcDateStringToLocalString,
		utcDateToString
	} from '$lib/utils';

	import { onMount } from 'svelte';
	import { page } from '$app/stores';

	import type { AfterNavigate } from '@sveltejs/kit';
	import type { ItemSummary } from '$lib/model';

	import Item from '$lib/Item.svelte';
	import Search from '$lib/Search.svelte';

	const BATCH_SIZE = 100;

	let itemListElement: HTMLElement;
	let loadMoreElement: HTMLElement;

	let query: string;
	let system: string;
	let type: string;
	let from: string;
	let to: string;
	let asc: boolean;

	let loading = false;

	let items: ItemSummary[] = [];
	let systems: string[] = [];
	let totalItems = 0;

	let activeItemId: number;
	let hasMoreItems = false;

	const loadMore = async () => {
		loading = true;

		const firstItemId = asc ? items[items.length - 1].id + 1 : 0;
		const lastItemId = asc ? Number.MAX_SAFE_INTEGER : items[items.length - 1].id - 1;

		const result = await getItems($page.url.searchParams, firstItemId, lastItemId, BATCH_SIZE);

		items.push(...result.items.slice(0, BATCH_SIZE));
		items = items;
		systems = result.systems;

		if (asc) {
			totalItems = result.totalItems;
		}

		hasMoreItems = result.items.length > BATCH_SIZE;

		loading = false;
	};

	const refresh = (params: URLSearchParams) => {
		goto(`?${params}`, { keepFocus: true });
	};

	const search = (e: CustomEvent) => {
		const formData = e.detail as FormData;
		const params = new URLSearchParams();

		for (const [key, value] of formData.entries()) {
			let v = value.toString().trim();

			if (v) {
				if (key === 'from' || key === 'to') {
					v = utcDateToString(new Date(v));
				}

				params.set(key, v);
			}
		}

		if (asc) {
			params.set('asc', 'true');
		}

		items = [];
		totalItems = 0;
		hasMoreItems = false;

		refresh(params);
	};

	const toggleSortBy = () => {
		const params = $page.url.searchParams;

		if (asc) {
			params.delete('asc');
		} else {
			params.set('asc', 'true');
		}

		hasMoreItems = false;

		refresh(params);
	};

	afterNavigate(async (e: AfterNavigate) => {
		if (e.type === 'enter') {
			return;
		}

		const params = $page.url.searchParams;

		query = params.get('query') || '';
		system = params.get('system') || '';
		type = params.get('type') || '';
		from = utcDateStringToLocalString(params.get('from'));
		to = utcDateStringToLocalString(params.get('to'));
		asc = (params.get('asc') || 'false') === 'true';

		loading = true;
		items = [];

		const result = await getItems(params, 0, Number.MAX_SAFE_INTEGER, BATCH_SIZE);

		items = result.items.slice(0, BATCH_SIZE);
		systems = result.systems;
		totalItems = result.totalItems;
		hasMoreItems = result.items.length > BATCH_SIZE;

		loading = false;
	});

	onMount(() => {
		const loadMoreObserver = new IntersectionObserver(
			(entries: IntersectionObserverEntry[]) => {
				if (hasMoreItems && entries[0].isIntersecting) {
					loadMore();
				}
			},
			{
				root: itemListElement
			}
		);

		loadMoreObserver.observe(loadMoreElement);
	});

	setInterval(async () => {
		if (loading || (asc && hasMoreItems)) {
			return;
		}

		const firstItemId =
			items.length === 0 ? 1 : asc ? items[items.length - 1].id + 1 : items[0].id + 1;

		const lastItemId = Number.MAX_SAFE_INTEGER;
		const result = await getItems($page.url.searchParams, firstItemId, lastItemId);

		if (asc) {
			items = items.concat(result.items);
		} else {
			items = result.items.concat(items);
		}

		systems = result.systems;
		totalItems += result.items.length;
	}, MILLISECONDS_IN_MINUTE);
</script>

<svelte:head>
	<title>Sink</title>
</svelte:head>

<div class="d-flex flex-column vh-100">
	<nav class="navbar shadow">
		<div class="m-auto w-50">
			<Search {query} {system} {type} {from} {to} {systems} on:search={search} />
		</div>
	</nav>
	<div class="d-flex flex-fill overflow-hidden">
		<div class="border-end d-flex flex-column" style="min-width: 25em">
			<div class="align-items-center border-bottom d-flex justify-content-between p-2">
				<div>{totalItems.toLocaleString()} Items</div>
				<div class="d-flex">
					<label class="form-label m-auto me-2 text-nowrap" for="asc">Sort By</label>
					<select
						class="form-select form-select-sm"
						id="asc"
						name="asc"
						value={asc}
						on:change={toggleSortBy}
					>
						<option value={false}>Latest</option>
						<option value={true}>Oldest</option>
					</select>
				</div>
			</div>
			<div class="overflow-auto p-2">
				{#if items.length > 0}
					<div class="list-group list-group-flush" bind:this={itemListElement}>
						{#each items as item, index (item.id)}
							<a
								class="list-group-item list-group-item-action"
								class:active={item.id === activeItemId}
								href="/item/{item.id}"
								on:click|preventDefault={() => (activeItemId = item.id)}
							>
								<div class="d-flex justify-content-between">
									<span>{item.id.toLocaleString()}</span>
									<span>
										{new Date(utcDateStringToLocalString(item.submitDate)).toLocaleString()}
									</span>
								</div>
								<div>
									{#if item.system}
										<span class="badge bg-secondary">{item.system}</span>
									{/if}
									{#if item.type}
										<span class="badge {item.type}">{itemTypeToName(item.type)}</span>
									{/if}
								</div>
							</a>
						{/each}
					</div>
				{:else if !loading}
					<div class="text-center text-muted">We didn't find anything to show here.</div>
				{/if}
				{#if loading}
					<div class="m-2 text-center">
						<div class="spinner-border text-muted" role="status" />
					</div>
				{/if}
				<div bind:this={loadMoreElement} />
			</div>
		</div>
		<div class="flex-fill overflow-auto p-2">
			{#if activeItemId}
				<Item id={activeItemId} />
			{/if}
		</div>
	</div>
</div>
