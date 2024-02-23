<script lang="ts">
	import EVENT_TYPES from '../../../event.types.json';

	import 'bootstrap/dist/js/bootstrap';

	import { createEventDispatcher } from 'svelte';

	import {
		localDateToString,
		ITEM_TYPES,
		MILLISECONDS_IN_HOUR,
		itemTypeFromKey
	} from '$lib/shared';

	export let query: string;
	export let system: string[];
	export let type: string[];
	export let eventType: number[];
	export let from: string;
	export let to: string;
	export let systems: string[] = [];

	let form: HTMLFormElement;
	let dateFilter = '';

	const version = import.meta.env.CARGO_PKG_VERSION;

	const dispatch = createEventDispatcher();

	const formatDateFilter = (from: string, to: string) => {
		if (from) {
			const fromDate = new Date(from).toLocaleString();

			if (to) {
				const toDate = new Date(to).toLocaleString();
				return `${fromDate} - ${toDate}`;
			} else {
				return `From ${fromDate}`;
			}
		}

		if (to) {
			const toDate = new Date(to).toLocaleString();
			return `To ${toDate}`;
		}

		return 'Any';
	};

	const lastHour = (): void => {
		const now = new Date();

		now.setSeconds(0);

		from = localDateToString(new Date(now.getTime() - MILLISECONDS_IN_HOUR));
		to = '';
	};

	const search = () => {
		dispatch('search', new FormData(form));
	};

	const today = (): void => {
		let fromDate = new Date();

		fromDate.setHours(0);
		fromDate.setMinutes(0);
		fromDate.setSeconds(0);

		from = localDateToString(fromDate);
		to = '';
	};

	$: dateFilter = formatDateFilter(from, to);
</script>

<form class="d-flex w-100" on:submit|preventDefault={search} bind:this={form}>
	<input
		class="form-control"
		id="query"
		name="query"
		placeholder="Search"
		type="search"
		value={query}
		style="width: 24em"
	/>
	<div class="dropdown ms-2">
		<button
			class="btn btn-secondary dropdown-toggle"
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
		<div class="dropdown-menu bg-white p-2">
			{#each systems as s, i}
				<div class="form-check">
					<input
						class="form-check-input"
						id="system-{i}"
						name="system"
						type="checkbox"
						value={s}
						bind:group={system}
						on:change={search}
					/>
					<label class="form-check-label text-nowrap" for="system-{i}">{s}</label>
				</div>
			{/each}
		</div>
	</div>

	<div class="dropdown ms-2">
		<button
			class="btn btn-secondary dropdown-toggle"
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
		<div class="dropdown-menu bg-white p-0">
			<div class="d-flex">
				<div class="p-2">
					{#each ITEM_TYPES as t, i}
						<div class="form-check">
							<input
								class="form-check-input"
								id="type-{i}"
								name="type"
								type="checkbox"
								value={t.key}
								bind:group={type}
								on:change={search}
							/>
							<label class="form-check-label text-nowrap" for="type-{i}">{t.name}</label>
						</div>
					{/each}
				</div>
				{#if type.includes('event_notification') || type.includes('event_payload')}
					<div class="border-start m-2" style="overflow-x: hidden; max-height: 25em; width: 23em">
						<div class="me-2 ms-2">
							{#each EVENT_TYPES as t, index}
								{#if t.id}
									<div class="form-check">
										<input
											class="form-check-input"
											id="eventType-{index}"
											name="eventType"
											type="checkbox"
											value={t.id}
											bind:group={eventType}
											on:change={search}
										/>
										<label class="form-check-label text-nowrap" for="eventType-{index}"
											>{t.name}</label
										>
									</div>
								{:else}
									<div class="border-bottom" class:mt-2={index > 0}>{t.name}</div>
								{/if}
							{/each}
						</div>
					</div>
				{/if}
			</div>
		</div>
	</div>

	<div class="dropdown ms-2">
		<button
			class="btn btn-secondary dropdown-toggle"
			type="button"
			data-bs-toggle="dropdown"
			data-bs-auto-close="outside"
			aria-expanded="false"
		>
			<span class="text-nowrap me-1"
				><b>Date:</b>
				{dateFilter}</span
			>
		</button>
		<div class="dropdown-menu bg-white p-2">
			<div class="container-fluid p-0">
				<div class="row">
					<div class="col mt-2">
						<label class="form-label" for="from">From</label>
						<input
							class="form-control form-control-sm"
							id="from"
							name="from"
							type="datetime-local"
							value={from}
							on:change={search}
						/>
					</div>
					<div class="col mt-2">
						<label class="form-label" for="to">To</label>
						<input
							class="form-control form-control-sm"
							id="to"
							name="to"
							type="datetime-local"
							value={to}
							on:change={search}
						/>
					</div>
				</div>
				<div class="d-flex justify-content-end mt-2">
					<button class="btn btn-outline-secondary btn-sm me-2" on:click={lastHour}>
						Last Hour
					</button>
					<button class="btn btn-outline-secondary btn-sm" on:click={today}>Today</button>
				</div>
			</div>
		</div>
	</div>
	<div class="ms-auto my-auto">
		<a href="https://github.com/krabicezpapundeklu/sink/releases/tag/{version}" target="_blank"
			>{version}</a
		>
	</div>
</form>
