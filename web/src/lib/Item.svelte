<script lang="ts">
	import { formatNumber, itemTypeToName, utcDateStringToLocalString } from '$lib/shared';
	import { onMount } from 'svelte';
	import type { ItemWithHighlighting } from '$lib/model';

	const tabs = ['#body-preview', '#original-body', '#headers'];

	export let item: ItemWithHighlighting;

	let activeTab = 0;

	onMount(() => {
		activeTab = Math.max(0, tabs.indexOf(location.hash));
	});
</script>

<div>
	<span class="fs-3 me-3">#{formatNumber(item.id)}</span>
	<span>{utcDateStringToLocalString(item.submitDate)}</span>
	<div>
		{#if item.system}
			<span class="badge bg-secondary">{item.system}</span>
		{/if}
		{#if item.type}
			<span class="badge {item.type}">{itemTypeToName(item.type)}</span>
		{/if}
	</div>
</div>
<ul class="mt-3 nav nav-tabs">
	<li class="nav-item">
		<a
			class="nav-link"
			class:active={activeTab === 0}
			href={tabs[0]}
			on:click={() => (activeTab = 0)}>Body Preview</a
		>
	</li>
	<li class="nav-item">
		<a
			class="nav-link"
			class:active={activeTab === 1}
			href={tabs[1]}
			on:click={() => (activeTab = 1)}>Original Body</a
		>
	</li>
	<li class="nav-item">
		<a
			class="nav-link"
			class:active={activeTab === 2}
			href={tabs[2]}
			on:click={() => (activeTab = 2)}>Headers</a
		>
	</li>
</ul>
<div class="bg-white">
	{#if activeTab === 2}
		<table class="m-0 table table-bordered table-sm">
			<thead>
				<tr class="border-top-0">
					<th scope="col">Name</th>
					<th scope="col">Value</th>
				</tr>
			</thead>
			<tbody>
				{#each item.headers as header}
					<tr>
						<td>{header.name}</td>
						<td>{header.value}</td>
					</tr>
				{/each}
			</tbody>
		</table>
	{:else}
		<div class="border border-top-0 p-2">
			<pre class="mb-0"><code
					>{@html activeTab === 0 ? item.highlightedBodyPreview : item.higlightedBody}</code
				></pre>
		</div>
	{/if}
</div>
