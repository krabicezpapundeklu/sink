<script lang="ts">
	import { createEventDispatcher, onMount } from 'svelte';
	import { env } from '$env/dynamic/public';
	import { ITEM_TYPES, localDateToString, MILLISECONDS_IN_HOUR } from '$lib/shared';

	export let query: string;
	export let system: string[];
	export let type: string[];
	export let from: string;
	export let to: string;

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
		type="button"
		bind:this={filterButton}
	>
		Filter
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
