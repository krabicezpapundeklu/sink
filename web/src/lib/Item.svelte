<script lang="ts">
	import { browser } from '$app/environment';
	import { itemTypeToName, utcDateStringToLocalString } from '$lib/shared';
	import type { ItemWithHighlighting } from '$lib/model';

	export let item: ItemWithHighlighting;

	let activeTab = 0;
</script>

<div>
	{#if browser}
		<span class="fs-3 me-3">{item.id.toLocaleString()}</span>
		<span>{new Date(utcDateStringToLocalString(item.submitDate)).toLocaleString()}</span>
	{/if}
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
		<!-- svelte-ignore a11y-invalid-attribute -->
		<a
			class="nav-link"
			class:active={activeTab === 0}
			href="javascript:void(0)"
			on:click={() => (activeTab = 0)}>Body Preview</a
		>
	</li>
	<li class="nav-item">
		<!-- svelte-ignore a11y-invalid-attribute -->
		<a
			class="nav-link"
			class:active={activeTab === 1}
			href="javascript:void(0)"
			on:click={() => (activeTab = 1)}>Original Body</a
		>
	</li>
	<li class="nav-item">
		<!-- svelte-ignore a11y-invalid-attribute -->
		<a
			class="nav-link"
			class:active={activeTab === 2}
			href="javascript:void(0)"
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
