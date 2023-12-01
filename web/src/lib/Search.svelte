<script lang="ts">
	import 'bootstrap/js/dist/modal.js';

	import EVENT_TYPES from '../../../event.types.json';

	import { createEventDispatcher, onMount } from 'svelte';
	import { localDateToString, ITEM_TYPES, MILLISECONDS_IN_HOUR } from '$lib/shared';

	import Modal from 'bootstrap/js/dist/modal.js';

	export let query: string;
	export let system: string[];
	export let type: string[];
	export let eventType: number[];
	export let from: string;
	export let to: string;

	export let systems: string[] = [];

	let modal: Modal;
	let modalElement: HTMLDivElement;

	const version = import.meta.env.CARGO_PKG_VERSION;

	const dispatch = createEventDispatcher();

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
		modal.hide();
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

	onMount(() => {
		modal = new Modal(modalElement);
	});
</script>

<form class="d-flex m-1" on:submit|preventDefault={search}>
	<input
		autocomplete="off"
		class="form-control"
		id="query"
		name="query"
		placeholder="Search"
		type="search"
		value={query}
	/>
	<button
		class="align-items-center border btn btn-light btn-sm d-flex ms-2"
		data-bs-target="#filters-modal"
		data-bs-toggle="modal"
		title="Filters"
		type="button"
	>
		<svg xmlns="http://www.w3.org/2000/svg" height="1.3em" viewBox="0 -960 960 960"
			><path
				style="fill: #6c757d"
				d="M427-120v-225h60v83h353v60H487v82h-60Zm-307-82v-60h247v60H120Zm187-166v-82H120v-60h187v-84h60v226h-60Zm120-82v-60h413v60H427Zm166-165v-225h60v82h187v60H653v83h-60Zm-473-83v-60h413v60H120Z"
			/></svg
		>
	</button>
	<div
		class="modal fade"
		id="filters-modal"
		tabindex="-1"
		aria-labelledby="filters-modal-label"
		aria-hidden="true"
		bind:this={modalElement}
	>
		<div class="modal-dialog">
			<div class="bg-white modal-content">
				<div class="modal-header">
					<h1 class="modal-title fs-5" id="filters-modal-label">Filters</h1>
					<button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close"
					></button>
				</div>
				<div class="modal-body">
					<div class="container-fluid p-0">
						<div class="row">
							<div class="col">
								<label class="form-label" for="system">System</label>
								{#if countSelected(system, systems)}
									<!-- svelte-ignore a11y-invalid-attribute -->
									<small
										>({countSelected(system, systems)} selected,
										<a href="javascript:void(0)" on:click={() => (system = [])}>unselect</a>)</small
									>
								{/if}
								<select
									class="form-select form-select-sm"
									id="system"
									multiple
									name="system"
									size="5"
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
									<!-- svelte-ignore a11y-invalid-attribute -->
									<small
										>({type.length} selected,
										<a href="javascript:void(0)" on:click={() => (type = [])}>unselect</a>)</small
									>
								{/if}
								<select
									class="form-select form-select-sm"
									id="type"
									multiple
									name="type"
									size="5"
									bind:value={type}
								>
									{#each ITEM_TYPES as type}
										<option value={type.key}>{type.name}</option>
									{/each}
								</select>
							</div>
						</div>
						<div class="row">
							<div class="col mt-2">
								<label class="form-label" for="eventType">Event Type</label>
								{#if eventType.length}
									<!-- svelte-ignore a11y-invalid-attribute -->
									<small
										>({eventType.length} selected,
										<a href="javascript:void(0)" on:click={() => (eventType = [])}>unselect</a
										>)</small
									>
								{/if}
								<select
									class="form-select form-select-sm"
									id="eventType"
									multiple
									name="eventType"
									size="7"
									bind:value={eventType}
								>
									{#each EVENT_TYPES as type, index}
										{#if type.id}
											<option value={type.id}>{type.name}</option>
										{:else}
											<option class="border-bottom" class:mt-2={index > 0} disabled
												>{type.name}</option
											>
										{/if}
									{/each}
								</select>
							</div>
						</div>
						<div class="row">
							<div class="col mt-2">
								<label class="form-label" for="from">Submitted From</label>
								<input
									class="form-control form-control-sm"
									id="from"
									name="from"
									type="datetime-local"
									value={from}
								/>
							</div>
							<div class="col mt-2">
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
							<button
								class="btn btn-outline-secondary btn-sm me-2"
								type="button"
								on:click={lastHour}
							>
								Last Hour
							</button>
							<button class="btn btn-outline-secondary btn-sm" type="button" on:click={today}
								>Today</button
							>
						</div>
					</div>
				</div>
				<div class="modal-footer">
					<div class="me-auto my-auto">
						Version:
						<a href="https://github.com/krabicezpapundeklu/sink/releases/tag/{version}">{version}</a
						>
					</div>
					<button type="button" class="btn btn-link" data-bs-dismiss="modal">Cancel</button>
					<button type="submit" class="btn btn-primary">Apply</button>
				</div>
			</div>
		</div>
	</div>
</form>
