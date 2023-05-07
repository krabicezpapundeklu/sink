<script lang="ts">
	import { formatNumber, itemTypeToName } from '$lib/shared';
	import { page } from '$app/stores';
	import type { ItemWithHighlighting } from '$lib/model';

	export let item: ItemWithHighlighting;
	export let preventDefault = true;

	const tabs = ['body-preview', 'original-body', 'headers'];

	let activeTab = Math.max(0, tabs.indexOf($page.url.searchParams.get('view') ?? ''));
	let base: string;

	const selectTab = (e: Event, tab: number) => {
		if (preventDefault) {
			e.preventDefault();
		}

		activeTab = tab;
	};

	$: base = `/item/${item.id}?view=`;
</script>

<div>
	<span class="fs-3 me-3">#{formatNumber(item.id)}</span>
	<span>{item.submitDate}</span>
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
