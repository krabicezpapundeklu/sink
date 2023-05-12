<script lang="ts">
	import { createEventDispatcher, onMount } from 'svelte';
	import { env } from '$env/dynamic/public';
	import { ITEM_TYPES, localDateToString, MILLISECONDS_IN_HOUR } from '$lib/shared';

	export let query: string;
	export let system: string[];
	export let type: string[];
	export let from: string;
	export let to: string;

	export let filterActive: boolean;
	export let systems: string[] = [];

	const version = env.PUBLIC_VERSION;

	let filterButton: HTMLElement;
	let filterDropDown: { hide: () => void };

	const dispatch = createEventDispatcher();

	const clear = (): void => {
		system = [];
		type = [];
	};

	const countSelected = (selected: string[], available: string[]): number => {
		let count = 0;

		for (const item of selected) {
			if (available.indexOf(item) !== -1) {
				++count;
			}
		}

		return count;
	};

	const lastHour = (): void => {
		const now = new Date();

		now.setSeconds(0);

		from = localDateToString(new Date(now.getTime() - MILLISECONDS_IN_HOUR));
		to = '';
	};

	const search = (e: SubmitEvent) => {
		filterDropDown.hide();
		dispatch('search', new FormData(e.target as HTMLFormElement));
	};

	const today = (): void => {
		let fromDate = new Date();

		fromDate.setHours(0);
		fromDate.setMinutes(0);
		fromDate.setSeconds(0);

		from = localDateToString(fromDate);
		to = '';
	};

	onMount(async () => {
		const Dropdown = (await import('bootstrap/js/dist/dropdown.js')).default;

		filterDropDown = new Dropdown(filterButton);

		filterButton.addEventListener('hide.bs.dropdown', () => {
			filterButton.focus();
		});
	});
</script>

<form class="dropdown input-group m-1" on:submit|preventDefault={search}>
	<label class="visually-hidden" for="query">Query</label>
	<input
		autocomplete="off"
		class="border-0 form-control rounded-start"
		id="query"
		name="query"
		placeholder="Search"
		type="search"
		value={query}
	/>
	<button
		class="btn btn-primary filter rounded-end"
		data-bs-auto-close="outside"
		data-bs-toggle="dropdown"
		title="Filter{filterActive ? ' active' : ''}"
		type="button"
		bind:this={filterButton}
	>
		{#if filterActive}
			<svg
				fill="white"
				version="1.1"
				viewBox="0 0 512 512"
				width="1em"
				xml:space="preserve"
				xmlns:xlink="http://www.w3.org/1999/xlink"
				xmlns="http://www.w3.org/2000/svg"
			>
				<g>
					<g>
						<polygon
							points="0,0 0,128 201.143,329.143 201.143,512 310.857,475.429 310.857,329.143 512,128 512,0"
						/>
					</g>
				</g>
			</svg>
		{:else}
			<svg
				fill="white"
				version="1.1"
				viewBox="0 0 300.906 300.906"
				width="1em"
				xml:space="preserve"
				xmlns:xlink="http://www.w3.org/1999/xlink"
				xmlns="http://www.w3.org/2000/svg"
			>
				<g>
					<g>
						<path
							d="M288.953,0h-277c-5.522,0-10,4.478-10,10v49.531c0,5.522,4.478,10,10,10h12.372l91.378,107.397v113.978
			c0,3.688,2.03,7.076,5.281,8.816c1.479,0.792,3.101,1.184,4.718,1.184c1.94,0,3.875-0.564,5.548-1.68l49.5-33
			c2.782-1.854,4.453-4.977,4.453-8.32v-80.978l91.378-107.397h12.372c5.522,0,10-4.478,10-10V10C298.953,4.478,294.476,0,288.953,0
			z M167.587,166.77c-1.539,1.809-2.384,4.105-2.384,6.48v79.305l-29.5,19.666V173.25c0-2.375-0.845-4.672-2.384-6.48L50.585,69.531
			h199.736L167.587,166.77z M278.953,49.531h-257V20h257V49.531z"
						/>
					</g>
				</g>
			</svg>
		{/if}
	</button>
	<div class="dropdown-menu mt-1 p-2 shadow w-100">
		<div class="row">
			<div class="col">
				<label class="form-label" for="system">System</label>
				{#if countSelected(system, systems)}
					<small>({countSelected(system, systems)} selected)</small>
				{/if}
				<select
					class="form-select form-select-sm"
					id="system"
					multiple
					name="system"
					bind:value={system}
				>
					{#each systems as system}
						<option value={system}>{system}</option>
					{/each}
				</select>
			</div>
			<div class="col">
				<label class="form-label" for="type">Type</label>
				{#if type.length}
					<small>({type.length} selected)</small>
				{/if}
				<select class="form-select form-select-sm" id="type" multiple name="type" bind:value={type}>
					{#each ITEM_TYPES as type}
						<option value={type.key}>{type.name}</option>
					{/each}
				</select>
			</div>
		</div>
		<div class="mt-2 row">
			<div class="col">
				<label class="form-label" for="from">Submitted From</label>
				<input
					class="form-control form-control-sm"
					id="from"
					name="from"
					type="datetime-local"
					value={from}
				/>
			</div>
			<div class="col">
				<label class="form-label" for="to">Submitted To</label>
				<input
					class="form-control form-control-sm"
					id="to"
					name="to"
					type="datetime-local"
					value={to}
				/>
			</div>
		</div>
		<div class="d-flex justify-content-end mt-2">
			<button class="btn btn-outline-secondary btn-sm me-2" type="button" on:click={lastHour}>
				Last Hour
			</button>
			<button class="btn btn-outline-secondary btn-sm" type="button" on:click={today}>Today</button>
		</div>
		<div class="border-top d-flex justify-content-end mt-2 pt-2">
			<div class="me-auto my-auto">
				Version:
				<a href="https://github.com/krabicezpapundeklu/sink/releases/tag/{version}">{version}</a>
			</div>
			<button class="btn btn-link me-2" type="reset" on:click={clear}>Clear</button>
			<button class="btn btn-primary">Search</button>
		</div>
	</div>
</form>
