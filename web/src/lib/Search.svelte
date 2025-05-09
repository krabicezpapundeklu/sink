<script lang="ts">
	import EVENT_TYPES from '../../../event.types.json';

	import 'bootstrap/js/dist/collapse';
	import 'bootstrap/js/dist/dropdown';

	import { ITEM_TYPES, itemTypeFromKey } from '$lib/shared';

	let {
		query,
		system,
		type,
		eventType,
		systems = [],
		from,
		onsearch
	}: {
		query: string;
		system: string[];
		type: string[];
		eventType: number[];
		systems: string[];
		from: string;
		onsearch: (data: FormData) => void;
	} = $props();

	let form: HTMLFormElement;
	let dateFilter: HTMLInputElement;
	let selectedDate: string = $state('');

	const version = import.meta.env.CARGO_PKG_VERSION;

	const applyDateFilter = (data: FormData) => {
		let date = dateFilter.valueAsDate;

		if (date) {
			const utcDate = new Date(
				date.getUTCFullYear(),
				date.getUTCMonth(),
				date.getUTCDate(),
				date.getUTCHours(),
				date.getUTCMinutes(),
				date.getUTCSeconds()
			);

			selectedDate = date.toLocaleDateString('en-us');
			data.set('from', utcDate.toISOString().substring(0, 16).replace('T', ' '));
			utcDate.setDate(utcDate.getDate() + 1);
			data.set('to', utcDate.toISOString().substring(0, 16).replace('T', ' '));
		} else {
			selectedDate = '';
		}
	};

	const clearFilter = (...filters: string[]) => {
		const data = new FormData(form);

		applyDateFilter(data);

		for (const filter of filters) {
			data.delete(filter);
		}

		onsearch(data);
	};

	const getSelectedEventGroups = () => {
		let selected = [];

		for (const eventTypeGroup of EVENT_TYPES) {
			if (isSelected(eventTypeGroup, eventType)) {
				selected.push(eventTypeGroup.name);
			}
		}

		return selected;
	};

	const isSelected = (
		eventTypeGroup: { types: { id: number }[] },
		selectedEventTypes: number[]
	) => {
		for (const et of eventTypeGroup.types) {
			if (selectedEventTypes.indexOf(et.id) !== -1) {
				return true;
			}
		}

		return false;
	};

	const search = (e: Event) => {
		e.preventDefault();

		let data = new FormData(form);

		applyDateFilter(data);
		onsearch(data);
	};

	$effect(() => {
		if (from) {
			const date = new Date(from.replace(' ', 'T') + 'Z');
			selectedDate = date.toLocaleDateString('en-us');
			dateFilter.value = `${date.getFullYear()}-${('0' + (date.getMonth() + 1)).slice(-2)}-${('0' + date.getDate()).slice(-2)}`;
		} else {
			selectedDate = '';
			dateFilter.value = '';
		}
	});

	const selectedEventGroups = getSelectedEventGroups();
</script>

<form class="d-flex w-100" onsubmit={search} bind:this={form}>
	<label class="d-none" for="query">Search</label>
	<div class="input-group w-25em">
		<input
			class="form-control"
			id="query"
			name="query"
			placeholder="Search"
			type="text"
			value={query}
		/>
		<button
			class="btn btn-outline-secondary"
			type="button"
			title="Clear Search"
			onclick={() => clearFilter('query')}>&#x2715;</button
		>
	</div>
	<div class="dropdown ms-2">
		<div class="btn-group" role="group">
			<button
				class="btn btn-outline-secondary dropdown-toggle flat-end"
				type="button"
				data-bs-toggle="dropdown"
				data-bs-auto-close="outside"
				data-bs-reference="parent"
				aria-expanded="false"
			>
				<span class="text-nowrap me-1"
					><b>System:</b>
					{system.length === 0
						? 'All'
						: system.length === 1
							? system[0]
							: `${system.length} selected`}</span
				>
			</button>
			<div class="bg-white dropdown-menu p-0 shadow-sm">
				<fieldset>
					<legend class="visually-hidden">System</legend>
					<ul class="list-group">
						{#each systems as s, i (s)}
							<li class="border-0 list-group-item list-group-item-action text-nowrap p-1">
								<input
									class="form-check-input m-1"
									id="system-{i}"
									name="system"
									type="checkbox"
									value={s}
									bind:group={system}
									onchange={search}
								/>
								<label class="form-check-label me-1 stretched-link" for="system-{i}">{s}</label>
							</li>
						{/each}
					</ul>
				</fieldset>
			</div>
			<button
				class="btn btn-outline-secondary rounded-end"
				type="button"
				title="Clear System Filter"
				onclick={() => clearFilter('system')}>&#x2715;</button
			>
		</div>
	</div>

	<div class="dropdown ms-2">
		<div class="btn-group" role="group">
			<button
				class="btn btn-outline-secondary dropdown-toggle flat-end"
				type="button"
				data-bs-toggle="dropdown"
				data-bs-auto-close="outside"
				data-bs-reference="parent"
				aria-expanded="false"
			>
				<span class="text-nowrap me-1"
					><b>Type:</b>
					{type.length === 0
						? 'All'
						: type.length === 1
							? itemTypeFromKey(type[0]).name
							: `${type.length} selected`}</span
				>
			</button>
			<div class="bg-white dropdown-menu p-0 shadow-sm">
				<fieldset>
					<legend class="visually-hidden">Type</legend>
					<ul class="list-group">
						{#each ITEM_TYPES as t, i (t)}
							<li class="border-0 list-group-item list-group-item-action text-nowrap p-1">
								<input
									class="form-check-input m-1"
									id="type-{i}"
									name="type"
									type="checkbox"
									value={t.key}
									bind:group={type}
									onchange={search}
								/>
								<label class="form-check-label me-1 stretched-link" for="type-{i}">{t.name}</label>
							</li>
						{/each}
					</ul>
				</fieldset>
			</div>
			<button
				class="btn btn-outline-secondary rounded-end"
				type="button"
				title="Clear Type Filter"
				onclick={() => clearFilter('type', 'eventType')}>&#x2715;</button
			>
		</div>
	</div>

	{#if type.includes('event_notification') || type.includes('event_payload')}
		<div class="dropdown ms-2">
			<div class="btn-group" role="group">
				<button
					class="btn btn-outline-secondary dropdown-toggle flat-end"
					type="button"
					data-bs-toggle="dropdown"
					data-bs-auto-close="outside"
					data-bs-reference="parent"
					aria-expanded="false"
				>
					<span class="text-nowrap me-1"
						><b>Event Type:</b>
						{eventType.length === 0 ? 'All' : `${eventType.length} selected`}</span
					>
				</button>
				<div class="dropdown-menu mh-30em mw-25em overflow-auto p-0 shadow-sm">
					<div class="accordion accordion-flush overflow-hidden">
						<fieldset>
							<legend class="visually-hidden">Event Type</legend>
							{#each EVENT_TYPES as eventGroup (eventGroup.name)}
								{@const selected = selectedEventGroups.indexOf(eventGroup.name) !== -1}
								<div class="accordion-item">
									<h2 class="accordion-header">
										<button
											class="accordion-button bg-white"
											type="button"
											data-bs-toggle="collapse"
											data-bs-target="#eventGroup-{eventGroup.name}"
											aria-expanded={selected}
											aria-controls="#eventGroup-{eventGroup.name}"
											class:collapsed={!selected}
											class:fw-bold={isSelected(eventGroup, eventType)}
										>
											{eventGroup.name}
										</button>
									</h2>
									<div
										id="eventGroup-{eventGroup.name}"
										class="accordion-collapse collapse {selected ? 'show' : ''}"
									>
										<div class="accordion-body p-0">
											<ul class="list-group list-group-flush">
												{#each eventGroup.types as t (t.id)}
													<li
														class="border-bottom-0 list-group-item list-group-item-action text-nowrap"
													>
														<input
															class="form-check-input me-1"
															id="eventType-{t.id}"
															name="eventType"
															type="checkbox"
															value={t.id}
															bind:group={eventType}
															onchange={search}
														/>
														<label class="form-check-label stretched-link" for="eventType-{t.id}"
															>{t.id} - {t.name}</label
														>
													</li>
												{/each}
											</ul>
										</div>
									</div>
								</div>
							{/each}
						</fieldset>
					</div>
				</div>
				<button
					class="btn btn-outline-secondary rounded-end"
					type="button"
					title="Clear Event Type Filter"
					onclick={() => clearFilter('eventType')}>&#x2715;</button
				>
			</div>
		</div>
	{/if}

	<div class="ms-2">
		<div class="btn-group" role="group">
			<button
				class="btn btn-outline-secondary dropdown-toggle flat-end"
				type="button"
				onclick={() => dateFilter.showPicker()}
			>
				<span class="text-nowrap me-1"
					><b>Date:</b>
					{selectedDate ? selectedDate : 'All'}
				</span>
			</button>
			<button
				class="btn btn-outline-secondary rounded-end"
				type="button"
				title="Clear Date Filter"
				onclick={() => clearFilter('from', 'to')}>&#x2715;</button
			>
			<label class="visually-hidden" for="date-filter"
				>Date: {selectedDate ? selectedDate : 'All'}</label
			>
			<input
				class="date-filter"
				id="date-filter"
				type="date"
				bind:this={dateFilter}
				onchange={search}
			/>
		</div>
	</div>

	<div class="ms-auto my-auto">
		<a
			class="ps-2"
			href="https://github.com/krabicezpapundeklu/sink/releases/{version
				? `tag/${version}`
				: 'latest'}"
			target="_blank">{version ?? 'DEV'}</a
		>
	</div>
</form>
