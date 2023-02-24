<script lang="ts">
	import { afterNavigate, goto } from '$app/navigation';
	import { getItems } from '$lib/api';
	import { itemTypeToName, utcDateStringToLocalString, utcDateToString } from '$lib/utils';
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
	let nextItemId = 0;

	const loadMore = async () => {
		loading = true;
		const result = await getItems($page.url.searchParams, BATCH_SIZE, nextItemId);
		loading = false;

		items.push(...result.items.slice(0, BATCH_SIZE));
		items = items;
		systems = result.systems;

		if (result.items.length > BATCH_SIZE) {
			nextItemId = result.items[BATCH_SIZE].id;
		} else {
			nextItemId = 0;
		}
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

		items = [];
		totalItems = 0;

		nextItemId = 0;

		refresh(params);
	};

	const toggleSortBy = () => {
		const params = $page.url.searchParams;

		if (asc) {
			params.delete('asc');
		} else {
			params.set('asc', 'true');
		}

		nextItemId = 0;

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
		const result = await getItems(params, BATCH_SIZE, nextItemId);
		loading = false;

		items = result.items.slice(0, BATCH_SIZE);
		systems = result.systems;
		totalItems = result.totalItems;

		if (result.items.length > BATCH_SIZE) {
			nextItemId = result.items[BATCH_SIZE].id;
		} else {
			nextItemId = 0;
		}
	});

	onMount(() => {
		const loadMoreObserver = new IntersectionObserver(
			(entries: IntersectionObserverEntry[]) => {
				if (nextItemId > 0 && entries[0].isIntersecting) {
					loadMore();
				}
			},
			{
				root: itemListElement
			}
		);

		loadMoreObserver.observe(loadMoreElement);
	});
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
						{#each items as item}
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
