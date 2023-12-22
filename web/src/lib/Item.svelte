<script lang="ts">
	import { formatBody, formatNumber, itemTypeFromKey } from '$lib/shared';
	import { page } from '$app/stores';
	import copy from 'copy-to-clipboard';
	import Highlighted from './Highlighted.svelte';
	import type { Item } from './model';

	export let item: Item;
	export let preventDefault = true;

	const tabs = ['body-preview', 'original-body', 'headers'];

	let activeTab = Math.max(0, tabs.indexOf($page.url.searchParams.get('view') ?? ''));
	let base = `/item/${item.id}?view=`;
	let tab: HTMLElement;

	let formattedBody = formatBody(item);

	const copyTab = () => {
		copy(tab.innerText);
	};

	const selectTab = (e: Event, tab: number) => {
		if (preventDefault) {
			e.preventDefault();
		}

		activeTab = tab;
	};
</script>

<div class="d-flex flex-column mh-100 p-2">
	<div>
		<span class="fs-3 me-3">#{formatNumber(item.id)}</span>
		<span>{item.submitDate}</span>
		<div>
			{#if item.system}
				<span class="badge bg-secondary">{item.system}</span>
			{/if}
			{#if item.type}
				{@const itemType = itemTypeFromKey(item.type)}
				<span class="badge" style="background-color: {itemType.color}">{itemType.name}</span>
			{/if}
		</div>
	</div>
	<div class="d-flex mt-3">
		<ul class="d-inline-flex nav nav-underline">
			<li class="nav-item">
				<a
					class="nav-link"
					class:active={activeTab === 0}
					data-sveltekit-preload-data="off"
					href="{base}{tabs[0]}"
					on:click={(e) => selectTab(e, 0)}>Body Preview</a
				>
			</li>
			<li class="nav-item">
				<a
					class="nav-link"
					class:active={activeTab === 1}
					data-sveltekit-preload-data="off"
					href="{base}{tabs[1]}"
					on:click={(e) => selectTab(e, 1)}>Original Body</a
				>
			</li>
			<li class="nav-item">
				<a
					class="nav-link"
					class:active={activeTab === 2}
					data-sveltekit-preload-data="off"
					href="{base}{tabs[2]}"
					on:click={(e) => selectTab(e, 2)}>Headers</a
				>
			</li>
		</ul>
		<div class="align-self-center flex-fill">
			<button class="btn btn-outline-secondary btn-sm float-end" on:click={copyTab}
				>Copy to Clipboard</button
			>
		</div>
	</div>
	<div class="bg-white border mt-1 overflow-auto" bind:this={tab}>
		{#if activeTab === 0}
			<Highlighted body={formattedBody} />
		{:else if activeTab === 1}
			<Highlighted body={item.body} />
		{:else if activeTab === 2}
			<table class="m-0 table table-sm">
				<thead>
					<tr>
						<th scope="col">Name</th>
						<th class="border-start" scope="col">Value</th>
					</tr>
				</thead>
				<tbody>
					{#each item.headers as header}
						<tr>
							<td class="border-bottom-0 border-top">{header.name}</td>
							<td class="border-bottom-0 border-start border-top">{header.value}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		{/if}
	</div>
</div>
