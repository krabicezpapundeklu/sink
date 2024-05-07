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
		onsearch
	}: {
		query: string;
		system: string[];
		type: string[];
		eventType: number[];
		systems: string[];
		onsearch: (data: FormData) => void;
	} = $props();

	let form: HTMLFormElement;

	const version = import.meta.env.CARGO_PKG_VERSION;

	const clearFilter = (...filters: string[]) => {
		const data = new FormData(form);

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
		onsearch(new FormData(form));
	};

	const selectedEventGroups = getSelectedEventGroups();
</script>

<form class="d-flex w-100" onsubmit={(e) => search(e)} bind:this={form}>
	<label class="d-none" for="query">Search</label>
	<input
		class="form-control w-25em"
		id="query"
		name="query"
		placeholder="Search"
		type="search"
		value={query}
	/>

	<div class="dropdown ms-2">
		<div class="btn-group" role="group">
			<button
				class="btn btn-outline-secondary"
				type="button"
				title="Clear System Filter"
				onclick={() => clearFilter('system')}>&#x2715;</button
			>
			<button
				class="btn btn-outline-secondary dropdown-toggle"
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
				<ul class="list-group">
					{#each systems as s, i}
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
			</div>
		</div>
	</div>

	<div class="dropdown ms-2">
		<div class="btn-group" role="group">
			<button
				class="btn btn-outline-secondary"
				type="button"
				title="Clear Type Filter"
				onclick={() => clearFilter('type', 'eventType')}>&#x2715;</button
			>
			<button
				class="btn btn-outline-secondary dropdown-toggle"
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
				<ul class="list-group">
					{#each ITEM_TYPES as t, i}
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
			</div>
		</div>
	</div>

	{#if type.includes('event_notification') || type.includes('event_payload')}
		<div class="dropdown ms-2">
			<div class="btn-group" role="group">
				<button
					class="btn btn-outline-secondary"
					type="button"
					title="Clear Event Type Filter"
					onclick={() => clearFilter('eventType')}>&#x2715;</button
				>
				<button
					class="btn btn-outline-secondary dropdown-toggle"
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
						{#each EVENT_TYPES as eventGroup}
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
											{#each eventGroup.types as t}
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
					</div>
				</div>
			</div>
		</div>
	{/if}
	<div class="ms-auto my-auto">
		<a
			class="ps-2"
			href="https://github.com/krabicezpapundeklu/sink/releases/tag/{version}"
			target="_blank">{version}</a
		>
	</div>
</form>
