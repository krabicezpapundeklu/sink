<script lang="ts">
	import { base } from '$app/paths';
	import { formatBody, formatNumber, getEntityEventType, itemTypeFromKey } from '$lib/shared';
	import { page } from '$app/stores';
	import copy from 'copy-to-clipboard';
	import Highlighted from './Highlighted.svelte';
	import LocalDateTime from './LocalDateTime.svelte';
	import type { Item } from './model';

	let { item, preventDefault = true }: { item: Item; preventDefault?: boolean } = $props();

	const tabs = ['body-preview', 'original-body', 'headers'];

	let activeTab = $state(Math.max(0, tabs.indexOf($page.url.searchParams.get('view') ?? '')));
	let tabBase = `${base}/item/${item.id}?view=`;
	let tab: HTMLElement;

	let formattedBody: { body: string; language: string } = $derived(formatBody(item));

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
	<div class="bg-white border p-2 rounded shadow-sm">
		<span class="fs-3">#{formatNumber(item.eventId ?? item.id)}</span>
		{#if item.userAgent}
			<span class="badge bg-secondary">{item.userAgent}</span>
		{/if}
		<span class="ms-3"><LocalDateTime dateTime={item.submitDate} detail={true} /></span>
		<div>
			{#if item.system}
				<span class="badge bg-secondary">{item.system}</span>
			{/if}
			{#if item.type}
				{@const itemType = itemTypeFromKey(item.type)}
				<span class="badge" style="background-color: {itemType.color}"
					>{itemType.name}
					{#if item.entityEventId}
						({getEntityEventType(item.entityEventId)})
					{/if}
				</span>
			{/if}
		</div>
	</div>
	<div class="bg-white border d-flex flex-column rounded p-2 mt-2 overflow-hidden shadow-sm">
		<div class="d-flex">
			<ul class="d-inline-flex nav nav-underline">
				<li class="nav-item">
					<a
						class="nav-link"
						class:active={activeTab === 0}
						data-sveltekit-preload-data="off"
						href="{tabBase}{tabs[0]}"
						onclick={(e) => selectTab(e, 0)}>Body Preview</a
					>
				</li>
				<li class="nav-item">
					<a
						class="nav-link"
						class:active={activeTab === 1}
						data-sveltekit-preload-data="off"
						href="{tabBase}{tabs[1]}"
						onclick={(e) => selectTab(e, 1)}>Original Body</a
					>
				</li>
				<li class="nav-item">
					<a
						class="nav-link"
						class:active={activeTab === 2}
						data-sveltekit-preload-data="off"
						href="{tabBase}{tabs[2]}"
						onclick={(e) => selectTab(e, 2)}>Headers</a
					>
				</li>
			</ul>
			<div class="align-self-center flex-fill">
				<button class="btn btn-outline-secondary btn-sm float-end" onclick={copyTab}
					>Copy to Clipboard</button
				>
			</div>
		</div>
		<div class="bg-white border mt-1 overflow-auto" bind:this={tab}>
			{#if activeTab === 0}
				{#key item}
					<Highlighted body={formattedBody.body} language={formattedBody.language} />
				{/key}
			{:else if activeTab === 1}
				{#key item}
					<Highlighted body={item.body} language={formattedBody.language} />
				{/key}
			{:else if activeTab === 2}
				<table class="m-0 table table-sm">
					<thead>
						<tr>
							<th scope="col">Name</th>
							<th class="border-start" scope="col">Value</th>
						</tr>
					</thead>
					<tbody>
						{#each item.headers as header (header.name)}
							<tr>
								<td class="bg-white border-bottom-0 border-top">{header.name}</td>
								<td class="bg-white border-bottom-0 border-start border-top">{header.value}</td>
							</tr>
						{/each}
					</tbody>
				</table>
			{/if}
		</div>
	</div>
</div>
