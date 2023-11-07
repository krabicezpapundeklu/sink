<script lang="ts">
	import { afterNavigate, goto } from '$app/navigation';

	import {
		BATCH_SIZE,
		formatNumber,
		highlightItem,
		itemTypeFromKey,
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
	<div class="border-top d-flex flex-fill overflow-hidden">
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
			<div class="me-1 overflow-auto" bind:this={itemListElement}>
				{#if items.length > 0}
					<div class="list-group list-group-flush p-2">
						{#each items as item (item.id)}
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
										{@const itemType = itemTypeFromKey(item.type)}
										<span class="badge" style="background-color: {itemType?.color}"
											>{itemType?.name}</span
										>
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
				class="border-start d-flex flex-column flex-fill mw-0 shadow"
				style="background-color: #f5f5f5"
			>
				<Item item={activeItem} />
			</div>
		{:else}
			<div class="border-start d-flex flex-fill shadow" style="background-color: #f5f5f5">
				<div class="m-auto opacity-25 w-25">
					<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 -960 960 960"
						><path
							d="M270-80q-78 0-134-56T80-270q0-78 56-134t134-56q78 0 134 56t56 134q0 78-56 134T270-80Zm566-40L573-383q-14 11-31.5 21.5T508-344q-5-14-11-28.5T483-399q54-21 91.5-69.5T612-584q0-81-57-138.5T417-780q-82 0-139.5 57.5T220-584q0 17 3.5 35.5T232-517q-13 2-29 6.5T174-500q-7-18-10.5-40t-3.5-44q0-107 75-181.5T417-840q106 0 180.5 75T672-584q0 43-15 85t-41 73l264 262-44 44Zm-635-56 69-69 68 68 23-23-69-69 71-71-23-23-70 70-70-70-23 23 70 70-70 70 24 24Z"
						/></svg
					>
				</div>
			</div>
		{/if}
	</div>
</div>
