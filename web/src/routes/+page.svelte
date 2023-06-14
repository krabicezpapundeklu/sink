<script lang="ts">
	import { afterNavigate, goto } from '$app/navigation';

	import {
		BATCH_SIZE,
		formatNumber,
		highlightItem,
		itemTypeToName,
		loadItem,
		loadItems,
		MILLISECONDS_IN_MINUTE,
		utcDateStringToLocalString,
		utcDateToString
	} from '$lib/shared';

	import { onMount } from 'svelte';
	import { page } from '$app/stores';

	import type { PageData } from './$types';

	import Item from '$lib/Item.svelte';
	import Search from '$lib/Search.svelte';

	export let data: PageData;

	let itemListElement: HTMLElement;
	let loadMoreElement: HTMLElement;

	let query: string;
	let system: string[];
	let type: string[];
	let from: string;
	let to: string;
	let asc: boolean;

	let loading = false;

	let items = data.items.slice(0, BATCH_SIZE);
	let systems = data.systems;
	let totalItems = data.totalItems;
	let hasMoreItems = data.items.length > BATCH_SIZE;
	let activeItem = data.firstItem;

	const loadMore = async () => {
		loading = true;

		const firstItemId = asc ? items[items.length - 1].id + 1 : 0;
		const lastItemId = asc ? Number.MAX_SAFE_INTEGER : items[items.length - 1].id - 1;

		const result = await loadItems(
			fetch,
			$page.url.searchParams,
			firstItemId,
			lastItemId,
			BATCH_SIZE
		);

		items.push(...result.items.slice(0, BATCH_SIZE));
		items = items;
		systems = result.systems;

		if (asc) {
			totalItems = result.totalItems;
		}

		hasMoreItems = result.items.length > BATCH_SIZE;

		loading = false;
	};

	const prefillFilters = () => {
		const params = $page.url.searchParams;

		query = params.get('query') ?? '';
		system = (params.get('system') ?? '').split(',').filter((s) => s.length);
		type = (params.get('type') ?? '').split(',').filter((t) => t.length);
		from = utcDateStringToLocalString(params.get('from'));
		to = utcDateStringToLocalString(params.get('to'));
		asc = (params.get('asc') ?? 'false') === 'true';
	};

	const refresh = (params: URLSearchParams) => {
		loading = true;
		goto(`?${params}${location.hash}`, { keepFocus: true, invalidateAll: true });
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

				let val = params.get(key) ?? '';

				if (val) {
					val += ',';
				}

				val += v;

				params.set(key, val);
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

	const selectItem = async (itemId: number) => {
		if (!activeItem || activeItem.id !== itemId) {
			activeItem = highlightItem(await loadItem(fetch, itemId));
		}
	};

	const toggleSortBy = () => {
		const params = $page.url.searchParams;

		if (asc) {
			params.delete('asc');
		} else {
			params.set('asc', 'true');
		}

		items = [];
		hasMoreItems = false;

		refresh(params);
	};

	afterNavigate(async () => {
		prefillFilters();

		items = data.items.slice(0, BATCH_SIZE);
		systems = data.systems;
		totalItems = data.totalItems;
		hasMoreItems = data.items.length > BATCH_SIZE;
		activeItem = data.firstItem;

		loading = false;
	});

	onMount(() => {
		const loadMoreObserver = new IntersectionObserver(
			(entries: IntersectionObserverEntry[]) => {
				if (hasMoreItems && !loading && entries[0].isIntersecting) {
					loadMore();
				}
			},
			{
				root: itemListElement
			}
		);

		loadMoreObserver.observe(loadMoreElement);

		setInterval(async () => {
			if (loading || (asc && hasMoreItems)) {
				return;
			}

			const firstItemId =
				items.length === 0 ? 1 : asc ? items[items.length - 1].id + 1 : items[0].id + 1;

			const lastItemId = Number.MAX_SAFE_INTEGER;
			const result = await loadItems(fetch, $page.url.searchParams, firstItemId, lastItemId);

			if (asc) {
				items = items.concat(result.items);
			} else {
				items = result.items.concat(items);
			}

			systems = result.systems;
			totalItems += result.items.length;
		}, MILLISECONDS_IN_MINUTE);
	});

	prefillFilters();
</script>

<svelte:head>
	<title>Sink</title>
</svelte:head>

<div class="d-flex flex-column vh-100">
	<nav class="navbar">
		<div class="m-auto w-50">
			<Search {query} {system} {type} {from} {to} {systems} on:search={search} />
		</div>
	</nav>
	<div class="d-flex flex-fill overflow-hidden">
		<div class="d-flex flex-column" style="min-width: 25em">
			<div class="align-items-center border-bottom d-flex justify-content-between p-2">
				<div>{formatNumber(totalItems)} Items</div>
				<div class="d-flex">
					<label class="form-label m-auto me-2 text-nowrap" for="asc">Sort By</label>
					<select
						class="form-select form-select-sm"
						id="asc"
						name="asc"
						value={asc}
						on:change={toggleSortBy}
					>
						<option value={false} selected={!asc}>Latest</option>
						<option value={true} selected={asc}>Oldest</option>
					</select>
				</div>
			</div>
			<div class="overflow-auto" bind:this={itemListElement}>
				{#if items.length > 0}
					<div class="list-group list-group-flush p-2">
						{#each items as item, index (item.id)}
							<a
								class="list-group-item list-group-item-action"
								class:active={activeItem && activeItem.id === item.id}
								data-sveltekit-preload-data="off"
								href="/item/{item.id}"
								on:click|preventDefault={() => selectItem(item.id)}
							>
								<div class="d-flex justify-content-between">
									<span>#{formatNumber(item.id)}</span>
									<span>{item.submitDate}</span>
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
		{#if activeItem}
			<div
				class="border-start border-top flex-fill ms-1 mt-1 overflow-auto p-2 rounded-top shadow"
				style="background-color: #f5f5f5"
			>
				<Item item={activeItem} />
			</div>
		{:else}
			<div
				class="border-start border-top d-flex flex-fill ms-1 mt-1 rounded-top shadow"
				style="background-color: #f5f5f5"
			>
				<div class="m-auto opacity-25 w-25">
					<svg
						xmlns="http://www.w3.org/2000/svg"
						shape-rendering="geometricPrecision"
						text-rendering="geometricPrecision"
						image-rendering="optimizeQuality"
						fill-rule="evenodd"
						clip-rule="evenodd"
						viewBox="0 0 424 511.51"
						><path
							fill-rule="nonzero"
							d="M174.43 443.27H21.31C9.54 443.27 0 433.73 0 421.97V21.3C0 9.51 9.52 0 21.31 0h200.94c3.64 0 6.97 1.66 9.15 4.36l104.84 102.09c5.64 5.64 8.62 10.07 8.62 11.43l-.02 135.35c-7.59-3.2-15.53-5.72-23.76-7.49l-.01-113.62h-98.82c-6.64 0-11.94-5.41-11.94-11.95V23.69H23.8v395.78h140.26c2.7 8.32 6.18 16.28 10.37 23.8zm118.07-169.1c28.59 0 54.48 11.59 73.22 30.33 18.75 18.74 30.33 44.63 30.33 73.23 0 20.92-6.2 40.39-16.87 56.68L424 483.26l-30.9 28.25-43.23-47.56c-16.42 10.95-36.15 17.34-57.37 17.34-28.6 0-54.49-11.6-73.22-30.34-18.75-18.74-30.34-44.63-30.34-73.22 0-28.6 11.59-54.49 30.33-73.23 18.74-18.74 44.63-30.33 73.23-30.33zm59.62 43.93c-15.25-15.26-36.33-24.7-59.62-24.7s-44.37 9.44-59.62 24.7c-15.26 15.26-24.7 36.34-24.7 59.63 0 23.28 9.44 44.37 24.7 59.62 15.25 15.26 36.33 24.69 59.62 24.69s44.37-9.43 59.62-24.69c15.26-15.26 24.7-36.34 24.7-59.62 0-23.29-9.44-44.37-24.7-59.63zm-36.35 21.39 14.49 14.57-23.37 23.67 23.39 23.69-14.53 14.49-23.25-23.54-23.27 23.58-14.49-14.57 23.36-23.67-23.38-23.69 14.53-14.49 23.24 23.54 23.28-23.58z"
						/></svg
					>
				</div>
			</div>
		{/if}
	</div>
</div>
