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

	const events = [
		{ id: 91, label: 'APPIAN_CASE APPIAN_SEND_PAYLOAD' },
		{ id: 90, label: 'APPIAN_EIE APPIAN_EIE_EVENT' },
		{ id: 49, label: 'APPLICANT RESUME_CREATED_UPDATED' },
		{ id: 50, label: 'APPLICANT RESUME_CREATED_UPDATED_DOC' },
		{ id: 44, label: 'APPLICATION CREATED' },
		{ id: 48, label: 'APPLICATION FIRST_HIRED' },
		{ id: 46, label: 'APPLICATION HIRED' },
		{ id: 47, label: 'APPLICATION UNHIRED' },
		{ id: 45, label: 'APPLICATION UPDATED' },
		{ id: 42, label: 'CERTIFICATE CREATED' },
		{ id: 72, label: 'CERTIFICATE EXPIRATION_UPDATED' },
		{ id: 71, label: 'CERTIFICATE STATUS_CHANGED' },
		{ id: 43, label: 'CERTIFICATE UPDATED' },
		{ id: 51, label: 'CERTIFICATE_APPLICANT CREATED' },
		{ id: 64, label: 'CERTIFICATE_APPLICATION HIRED' },
		{ id: 73, label: 'CERTIFICATE_APPLICATION SELECTED' },
		{ id: 65, label: 'CERTIFICATE_APPLICATION UNHIRED' },
		{ id: 74, label: 'CERTIFICATE_APPLICATION UNSELECTED' },
		{ id: 75, label: 'CERTIFICATE_ASSIGNEE SO_ASSIGN_CERTIFICATE' },
		{ id: 78, label: 'CERTIFICATE_ASSIGNEE SO_CERTIFICATE_ASSIGNEE_RETURNED' },
		{ id: 76, label: 'CERTIFICATE_ASSIGNEE SO_CERTIFICATE_ASSIGNEE_VIEWED' },
		{ id: 77, label: 'CERTIFICATE_ASSIGNEE SO_UNASSIGN_CERTIFICATE' },
		{ id: 68, label: 'ONBOARDING_NEW_HIRE UPDATED' },
		{ id: 5, label: 'ONBOARDING_PROCESS OB_COMPLETED' },
		{ id: 67, label: 'ONBOARDING_PROCESS OB_REASSIGNED' },
		{ id: 3, label: 'ONBOARDING_PROCESS OB_RESUMED' },
		{ id: 1, label: 'ONBOARDING_PROCESS OB_STARTED' },
		{ id: 2, label: 'ONBOARDING_PROCESS OB_SUSPENDED' },
		{ id: 4, label: 'ONBOARDING_PROCESS OB_TERMINATED' },
		{ id: 69, label: 'ONBOARDING_TASK OB_COMPLETED' },
		{ id: 66, label: 'ONBOARDING_TASK OB_REASSIGNED' },
		{ id: 70, label: 'ONBOARDING_TASK OB_STARTED' },
		{ id: 16, label: 'POSITION_CLASSIFICATION PC_CL_APPROVED' },
		{ id: 18, label: 'POSITION_CLASSIFICATION PC_CL_APPROVED_DELETED' },
		{ id: 17, label: 'POSITION_CLASSIFICATION PC_CL_APPROVED_UPDATED' },
		{ id: 13, label: 'POSITION_CLASSIFICATION PC_CS_APPROVED' },
		{ id: 14, label: 'POSITION_CLASSIFICATION PC_CS_APPROVED_DELETED' },
		{ id: 15, label: 'POSITION_CLASSIFICATION PC_CS_APPROVED_UPDATED' },
		{ id: 84, label: 'POSITION_CLASSIFICATION PC_FS_APPROVED' },
		{ id: 88, label: 'POSITION_CLASSIFICATION PC_FS_APPROVED_ACTIVATED' },
		{ id: 87, label: 'POSITION_CLASSIFICATION PC_FS_APPROVED_DEACTIVATED' },
		{ id: 85, label: 'POSITION_CLASSIFICATION PC_FS_APPROVED_DELETED' },
		{ id: 86, label: 'POSITION_CLASSIFICATION PC_FS_APPROVED_TERMINATED' },
		{ id: 83, label: 'POSITION_CLASSIFICATION PC_FS_APPROVED_UPDATED' },
		{ id: 19, label: 'POSITION_CLASSIFICATION PC_IDPD_APPROVED' },
		{ id: 21, label: 'POSITION_CLASSIFICATION PC_IDPD_APPROVED_DELETED' },
		{ id: 20, label: 'POSITION_CLASSIFICATION PC_IDPD_APPROVED_UPDATED' },
		{ id: 8, label: 'POSITION_CLASSIFICATION PC_PD_APPROVED' },
		{ id: 12, label: 'POSITION_CLASSIFICATION PC_PD_APPROVED_ACTIVATED' },
		{ id: 11, label: 'POSITION_CLASSIFICATION PC_PD_APPROVED_DEACTIVATED' },
		{ id: 9, label: 'POSITION_CLASSIFICATION PC_PD_APPROVED_DELETED' },
		{ id: 10, label: 'POSITION_CLASSIFICATION PC_PD_APPROVED_TERMINATED' },
		{ id: 7, label: 'POSITION_CLASSIFICATION PC_PD_APPROVED_UPDATED' },
		{ id: 35, label: 'VACANCY ANNOUNCED' },
		{ id: 37, label: 'VACANCY ANNOUNCEMENT_UPDATED' },
		{ id: 53, label: 'VACANCY APPROVED' },
		{ id: 56, label: 'VACANCY CANCELLED' },
		{ id: 36, label: 'VACANCY CERTIFICATE_CREATED' },
		{ id: 34, label: 'VACANCY CLOSED' },
		{ id: 41, label: 'VACANCY CREATED' },
		{ id: 39, label: 'VACANCY DELETED' },
		{ id: 55, label: 'VACANCY FILLED' },
		{ id: 40, label: 'VACANCY FIRST_HIRED' },
		{ id: 80, label: 'VACANCY NOT_READY_FOR_APPROVAL' },
		{ id: 33, label: 'VACANCY OPENED' },
		{ id: 52, label: 'VACANCY READY_FOR_APPROVAL' },
		{ id: 54, label: 'VACANCY UNAPPROVED' },
		{ id: 38, label: 'VACANCY UNLINK' },
		{ id: 57, label: 'VACANCY UN_PUBLISHED' },
		{ id: 81, label: 'VACANCY_REVIEW CREATED' },
		{ id: 82, label: 'VACANCY_REVIEW STATUS_CHANGED' },
		{ id: 26, label: 'VAC_DATA_TX ANNOUNCED' },
		{ id: 28, label: 'VAC_DATA_TX ANNOUNCEMENT_UPDATED' },
		{ id: 59, label: 'VAC_DATA_TX APPROVED' },
		{ id: 62, label: 'VAC_DATA_TX CANCELLED' },
		{ id: 27, label: 'VAC_DATA_TX CERTIFICATE_CREATED' },
		{ id: 23, label: 'VAC_DATA_TX CLOSED' },
		{ id: 32, label: 'VAC_DATA_TX CREATED' },
		{ id: 30, label: 'VAC_DATA_TX DELETED' },
		{ id: 61, label: 'VAC_DATA_TX FILLED' },
		{ id: 31, label: 'VAC_DATA_TX FIRST_HIRED' },
		{ id: 24, label: 'VAC_DATA_TX HIRED' },
		{ id: 79, label: 'VAC_DATA_TX NOT_READY_FOR_APPROVAL' },
		{ id: 22, label: 'VAC_DATA_TX OPENED' },
		{ id: 58, label: 'VAC_DATA_TX READY_FOR_APPROVAL' },
		{ id: 60, label: 'VAC_DATA_TX UNAPPROVED' },
		{ id: 25, label: 'VAC_DATA_TX UNHIRED' },
		{ id: 29, label: 'VAC_DATA_TX UNLINK' },
		{ id: 63, label: 'VAC_DATA_TX UN_PUBLISHED' }
	];

	const version = env.PUBLIC_VERSION;

	let form: HTMLFormElement;
	let filterButton: HTMLElement;
	let queryButton: HTMLElement;
	let filterDropDown: { hide: () => void };
	let queryDropDown: { hide: () => void };

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

	const selectEvent = (id: number) => {
		queryDropDown.hide();
		query = `regex:"entityEventId"\\s*:\\s*${id}\\D`;

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
</script>

<form class="dropdown m-1" bind:this={form} on:submit|preventDefault={search}>
	<div class="border d-flex p-1 rounded-pill text-bg-light w-100">
		<label class="visually-hidden" for="query">Query</label>
		<button
			class="align-items-center btn btn-link d-flex me-1 rounded-start-pill"
			data-bs-auto-close="outside"
			data-bs-toggle="dropdown"
			title="Queries"
			type="button"
			bind:this={queryButton}
		>
			<svg xmlns="http://www.w3.org/2000/svg" height="1.5em" viewBox="0 -960 960 960"
				><path
					style="fill: #6c757d"
					d="M260-260h560v-560h-60v278l-93-57-93 57v-278H260v560Zm0 60q-24 0-42-18t-18-42v-560q0-24 18-42t42-18h560q24 0 42 18t18 42v560q0 24-18 42t-42 18H260ZM140-80q-24 0-42-18t-18-42v-620h60v620h620v60H140Zm434-740h186-186Zm-314 0h560-560Z"
				/></svg
			>
		</button>
		<ul class="dropdown-menu mt-1 overflow-auto pe-1" style="max-height: 80vh">
			<li><h6 class="dropdown-header">Events</h6></li>
			{#each events as event}
				<li>
					<!-- svelte-ignore a11y-invalid-attribute -->
					<a class="dropdown-item" href="#" on:click={() => selectEvent(event.id)}>{event.label}</a>
				</li>
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
		/>
		<button
			class="align-items-center btn btn-link d-flex filter rounded-end-pill"
			data-bs-auto-close="outside"
			data-bs-toggle="dropdown"
			title="Filter"
			type="button"
			bind:this={filterButton}
		>
			<svg xmlns="http://www.w3.org/2000/svg" height="1.5em" viewBox="0 -960 960 960"
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
