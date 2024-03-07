<script lang="ts">
	import EVENT_TYPES from '../../../event.types.json';

	import 'bootstrap/dist/js/bootstrap';

	import { createEventDispatcher } from 'svelte';

	import { ITEM_TYPES, itemTypeFromKey } from '$lib/shared';

	export let query: string;
	export let system: string[];
	export let type: string[];
	export let eventType: number[];
	export let systems: string[] = [];

	let form: HTMLFormElement;

	const version = import.meta.env.CARGO_PKG_VERSION;

	const dispatch = createEventDispatcher();

	const search = () => {
		dispatch('search', new FormData(form));
	};
</script>

<form class="d-flex w-100" on:submit|preventDefault={search} bind:this={form}>
	<input
		class="form-control"
		id="query"
		name="query"
		placeholder="Search"
		type="search"
		value={query}
		style="width: 25em"
	/>
	<div class="dropdown ms-2">
		<button
			class="btn btn-outline-secondary dropdown-toggle"
			type="button"
			data-bs-toggle="dropdown"
			data-bs-auto-close="outside"
			aria-expanded="false"
		>
			<span class="text-nowrap me-1"
				><b>System:</b>
				{system.length === 0 ? 'All' : system.length === 1 ? system[0] : 'Multiple'}</span
			>
		</button>
		<div class="bg-white border-0 dropdown-menu p-0 shadow-sm">
			<ul class="list-group">
				{#each systems as s, i}
					<li class="list-group-item list-group-item-action text-nowrap p-1">
						<input
							class="form-check-input m-1"
							id="system-{i}"
							name="system"
							type="checkbox"
							value={s}
							bind:group={system}
							on:change={search}
						/>
						<label class="form-check-label me-1 stretched-link" for="system-{i}">{s}</label>
					</li>
				{/each}
			</ul>
		</div>
	</div>

	<div class="dropdown ms-2">
		<button
			class="btn btn-outline-secondary dropdown-toggle"
			type="button"
			data-bs-toggle="dropdown"
			data-bs-auto-close="outside"
			aria-expanded="false"
		>
			<span class="text-nowrap me-1"
				><b>Type:</b>
				{type.length === 0
					? 'All'
					: type.length === 1
						? itemTypeFromKey(type[0]).name
						: 'Multiple'}</span
			>
		</button>
		<div class="bg-white border-0 dropdown-menu p-0 shadow-sm">
			<ul class="list-group">
				{#each ITEM_TYPES as t, i}
					<li class="list-group-item list-group-item-action text-nowrap p-1">
						<input
							class="form-check-input m-1"
							id="type-{i}"
							name="type"
							type="checkbox"
							value={t.key}
							bind:group={type}
							on:change={search}
						/>
						<label class="form-check-label me-1 stretched-link" for="type-{i}">{t.name}</label>
					</li>
				{/each}
			</ul>
		</div>
	</div>

	{#if type.includes('event_notification') || type.includes('event_payload')}
		<div class="dropdown ms-2">
			<button
				class="btn btn-outline-secondary dropdown-toggle"
				type="button"
				data-bs-toggle="dropdown"
				data-bs-auto-close="outside"
				aria-expanded="false"
			>
				<span class="text-nowrap me-1"
					><b>Event Type:</b>
					{eventType.length === 0 ? 'All' : `${eventType.length} selected`}</span
				>
			</button>
			<div class="bg-white border dropdown-menu mh-30em overflow-auto p-0 shadow-sm">
				<ul class="list-group list-group-flush overflow-hidden">
					{#each EVENT_TYPES as t, index}
						{#if t.id}
							<li class="list-group-item list-group-item-action text-nowrap p-1">
								<input
									class="form-check-input m-1"
									id="eventType-{index}"
									name="eventType"
									type="checkbox"
									value={t.id}
									bind:group={eventType}
									on:change={search}
								/>
								<label class="form-check-label me-4 stretched-link" for="eventType-{index}"
									>{t.name}</label
								>
							</li>
						{:else}
							<li class="list-group-item list-group-item-action disabled" class:mt-2={index > 0}>
								{t.name}
							</li>
						{/if}
					{/each}
				</ul>
			</div>
		</div>
	{/if}
	<div class="ms-auto my-auto">
		<a href="https://github.com/krabicezpapundeklu/sink/releases/tag/{version}" target="_blank"
			>{version}</a
		>
	</div>
</form>
