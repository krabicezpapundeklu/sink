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

	const EVENT_REGEX = /regex:"entityEventId"\\s\*:\\s\*(\d+)\\D/;

	const events = [
		{ label: 'APPIAN_CASE', events: [{ id: 91, label: 'APPIAN_SEND_PAYLOAD' }] },
		{ label: 'APPIAN_EIE', events: [{ id: 90, label: 'APPIAN_EIE_EVENT' }] },
		{
			label: 'APPLICANT',
			events: [
				{ id: 49, label: 'RESUME_CREATED_UPDATED' },
				{ id: 50, label: 'RESUME_CREATED_UPDATED_DOC' }
			]
		},
		{
			label: 'APPLICATION',
			events: [
				{ id: 44, label: 'CREATED' },
				{ id: 48, label: 'FIRST_HIRED' },
				{ id: 46, label: 'HIRED' },
				{ id: 47, label: 'UNHIRED' },
				{ id: 45, label: 'UPDATED' }
			]
		},
		{
			label: 'CERTIFICATE',
			events: [
				{ id: 42, label: 'CREATED' },
				{ id: 72, label: 'EXPIRATION_UPDATED' },
				{ id: 71, label: 'STATUS_CHANGED' },
				{ id: 43, label: 'UPDATED' }
			]
		},
		{ label: 'CERTIFICATE_APPLICANT', events: [{ id: 51, label: 'CREATED' }] },
		{
			label: 'CERTIFICATE_APPLICATION',
			events: [
				{ id: 64, label: 'HIRED' },
				{ id: 73, label: 'SELECTED' },
				{ id: 65, label: 'UNHIRED' },
				{ id: 74, label: 'UNSELECTED' }
			]
		},
		{
			label: 'CERTIFICATE_ASSIGNEE',
			events: [
				{ id: 75, label: 'SO_ASSIGN_CERTIFICATE' },
				{ id: 78, label: 'SO_CERTIFICATE_ASSIGNEE_RETURNED' },
				{ id: 76, label: 'SO_CERTIFICATE_ASSIGNEE_VIEWED' },
				{ id: 77, label: 'SO_UNASSIGN_CERTIFICATE' }
			]
		},
		{ label: 'ONBOARDING_NEW_HIRE', events: [{ id: 68, label: 'UPDATED' }] },
		{
			label: 'ONBOARDING_PROCESS',
			events: [
				{ id: 5, label: 'OB_COMPLETED' },
				{ id: 67, label: 'OB_REASSIGNED' },
				{ id: 3, label: 'OB_RESUMED' },
				{ id: 1, label: 'OB_STARTED' },
				{ id: 2, label: 'OB_SUSPENDED' },
				{ id: 4, label: 'OB_TERMINATED' }
			]
		},
		{
			label: 'ONBOARDING_TASK',
			events: [
				{ id: 69, label: 'OB_COMPLETED' },
				{ id: 66, label: 'OB_REASSIGNED' },
				{ id: 70, label: 'OB_STARTED' }
			]
		},
		{
			label: 'POSITION_CLASSIFICATION',
			events: [
				{ id: 16, label: 'PC_CL_APPROVED' },
				{ id: 18, label: 'PC_CL_APPROVED_DELETED' },
				{ id: 17, label: 'PC_CL_APPROVED_UPDATED' },
				{ id: 13, label: 'PC_CS_APPROVED' },
				{ id: 14, label: 'PC_CS_APPROVED_DELETED' },
				{ id: 15, label: 'PC_CS_APPROVED_UPDATED' },
				{ id: 84, label: 'PC_FS_APPROVED' },
				{ id: 88, label: 'PC_FS_APPROVED_ACTIVATED' },
				{ id: 87, label: 'PC_FS_APPROVED_DEACTIVATED' },
				{ id: 85, label: 'PC_FS_APPROVED_DELETED' },
				{ id: 86, label: 'PC_FS_APPROVED_TERMINATED' },
				{ id: 83, label: 'PC_FS_APPROVED_UPDATED' },
				{ id: 19, label: 'PC_IDPD_APPROVED' },
				{ id: 21, label: 'PC_IDPD_APPROVED_DELETED' },
				{ id: 20, label: 'PC_IDPD_APPROVED_UPDATED' },
				{ id: 8, label: 'PC_PD_APPROVED' },
				{ id: 12, label: 'PC_PD_APPROVED_ACTIVATED' },
				{ id: 11, label: 'PC_PD_APPROVED_DEACTIVATED' },
				{ id: 9, label: 'PC_PD_APPROVED_DELETED' },
				{ id: 10, label: 'PC_PD_APPROVED_TERMINATED' },
				{ id: 7, label: 'PC_PD_APPROVED_UPDATED' }
			]
		},
		{
			label: 'VACANCY',
			events: [
				{ id: 35, label: 'ANNOUNCED' },
				{ id: 37, label: 'ANNOUNCEMENT_UPDATED' },
				{ id: 53, label: 'APPROVED' },
				{ id: 56, label: 'CANCELLED' },
				{ id: 36, label: 'CERTIFICATE_CREATED' },
				{ id: 34, label: 'CLOSED' },
				{ id: 41, label: 'CREATED' },
				{ id: 39, label: 'DELETED' },
				{ id: 55, label: 'FILLED' },
				{ id: 40, label: 'FIRST_HIRED' },
				{ id: 80, label: 'NOT_READY_FOR_APPROVAL' },
				{ id: 33, label: 'OPENED' },
				{ id: 52, label: 'READY_FOR_APPROVAL' },
				{ id: 54, label: 'UNAPPROVED' },
				{ id: 38, label: 'UNLINK' },
				{ id: 57, label: 'UN_PUBLISHED' }
			]
		},
		{
			label: 'VACANCY_REVIEW',
			events: [
				{ id: 81, label: 'CREATED' },
				{ id: 82, label: 'STATUS_CHANGED' }
			]
		},
		{
			label: 'VAC_DATA_TX',
			events: [
				{ id: 26, label: 'ANNOUNCED' },
				{ id: 28, label: 'ANNOUNCEMENT_UPDATED' },
				{ id: 59, label: 'APPROVED' },
				{ id: 62, label: 'CANCELLED' },
				{ id: 27, label: 'CERTIFICATE_CREATED' },
				{ id: 23, label: 'CLOSED' },
				{ id: 32, label: 'CREATED' },
				{ id: 30, label: 'DELETED' },
				{ id: 61, label: 'FILLED' },
				{ id: 31, label: 'FIRST_HIRED' },
				{ id: 24, label: 'HIRED' },
				{ id: 79, label: 'NOT_READY_FOR_APPROVAL' },
				{ id: 22, label: 'OPENED' },
				{ id: 58, label: 'READY_FOR_APPROVAL' },
				{ id: 60, label: 'UNAPPROVED' },
				{ id: 25, label: 'UNHIRED' },
				{ id: 29, label: 'UNLINK' },
				{ id: 63, label: 'UN_PUBLISHED' }
			]
		}
	];

	const version = env.PUBLIC_VERSION;

	let form: HTMLFormElement;
	let filterButton: HTMLElement;
	let queryButton: HTMLElement;
	let filterDropDown: { hide: () => void };
	let queryDropDown: { hide: () => void };

	let selectedEvent: number | null;

	const dispatch = createEventDispatcher();

	const clear = (): void => {
		system = [];
		type = [];
		selectedEvent = null;
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

	const preselectEvent = (query?: string) => {
		const match = EVENT_REGEX.exec(query ?? new FormData(form).get('query') + '');

		if (match) {
			selectedEvent = +match[1];
		} else {
			selectedEvent = null;
		}
	};

	const search = (e: SubmitEvent) => {
		filterDropDown.hide();
		dispatch('search', new FormData(e.target as HTMLFormElement));
	};

	const selectEvent = (id: number) => {
		queryDropDown.hide();
		selectedEvent = id;

		const q = (new FormData(form).get('query') + '').replace(EVENT_REGEX, '').trim();
		query = `regex:"entityEventId"\\s*:\\s*${id}\\D` + (q.length > 0 ? ' ' : '') + q;

		let data = new FormData(form);

		data.set('query', query);
		dispatch('search', data);
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

		queryDropDown = new Dropdown(queryButton);

		queryButton.addEventListener('hide.bs.dropdown', () => {
			queryButton.focus();
		});
	});

	preselectEvent(query);
</script>

<form class="dropdown m-1" bind:this={form} on:submit|preventDefault={search}>
	<div class="border d-flex p-1 rounded-pill text-bg-light w-100">
		<label class="visually-hidden" for="query">Query</label>
		<button
			class="align-items-center btn btn-link btn-sm d-flex me-1 rounded-start-pill"
			data-bs-auto-close="outside"
			data-bs-toggle="dropdown"
			title="Queries"
			type="button"
			bind:this={queryButton}
		>
			<svg xmlns="http://www.w3.org/2000/svg" height="1.3em" viewBox="0 -960 960 960"
				><path
					style="fill: #6c757d"
					d="M260-260h560v-560h-60v278l-93-57-93 57v-278H260v560Zm0 60q-24 0-42-18t-18-42v-560q0-24 18-42t42-18h560q24 0 42 18t18 42v560q0 24-18 42t-42 18H260ZM140-80q-24 0-42-18t-18-42v-620h60v620h620v60H140Zm434-740h186-186Zm-314 0h560-560Z"
				/></svg
			>
		</button>
		<ul class="dropdown-menu mt-1 overflow-auto p-1" style="max-height: 80vh">
			{#each events as group, i}
				<li><h6 class="dropdown-header">{group.label}</h6></li>
				{#each group.events as event}
					<button
						class="dropdown-item"
						type="button"
						class:active={event.id === selectedEvent}
						on:click={() => selectEvent(event.id)}>{event.label}</button
					>
				{/each}
				{#if i < events.length - 1}
					<li><hr class="dropdown-divider" /></li>
				{/if}
			{/each}
		</ul>
		<input
			autocomplete="off"
			class="border-0 form-control"
			id="query"
			name="query"
			placeholder="Search"
			type="search"
			value={query}
			on:change={() => preselectEvent()}
		/>
		<button
			class="align-items-center btn btn-link btn-sm d-flex filter rounded-end-pill"
			data-bs-auto-close="outside"
			data-bs-toggle="dropdown"
			title="Filter"
			type="button"
			bind:this={filterButton}
		>
			<svg xmlns="http://www.w3.org/2000/svg" height="1.3em" viewBox="0 -960 960 960"
				><path
					style="fill: #6c757d"
					d="M427-120v-225h60v83h353v60H487v82h-60Zm-307-82v-60h247v60H120Zm187-166v-82H120v-60h187v-84h60v226h-60Zm120-82v-60h413v60H427Zm166-165v-225h60v82h187v60H653v83h-60Zm-473-83v-60h413v60H120Z"
				/></svg
			>
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
					<select
						class="form-select form-select-sm"
						id="type"
						multiple
						name="type"
						bind:value={type}
					>
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
				<button class="btn btn-outline-secondary btn-sm" type="button" on:click={today}
					>Today</button
				>
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
	</div>
</form>
