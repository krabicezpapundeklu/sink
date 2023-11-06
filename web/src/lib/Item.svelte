<script lang="ts">
	import { formatNumber, itemTypeToName } from '$lib/shared';
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import type { ItemWithHighlighting } from '$lib/model';

	export let item: ItemWithHighlighting;
	export let preventDefault = true;

	const tabs = ['body-preview', 'original-body', 'headers'];

	let activeTab = Math.max(0, tabs.indexOf($page.url.searchParams.get('view') ?? ''));
	let base: string;
	let copy: (text: string) => boolean;
	let tab: HTMLElement;

	const copyTab = () => {
		copy(tab.innerText);
	};

	const selectTab = (e: Event, tab: number) => {
		if (preventDefault) {
			e.preventDefault();
		}

		activeTab = tab;
	};

	onMount(async () => {
		copy = (await import('copy-to-clipboard')).default;
	});

	$: base = `/item/${item.id}?view=`;
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
				<span class="badge {item.type}">{itemTypeToName(item.type)}</span>
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
	<div class="border mt-1 overflow-auto white-bg" bind:this={tab}>
		{#if activeTab === 2}
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
		{:else}
			<pre class="mb-0 overflow-hidden p-2 w-fc"><code
					>{@html activeTab === 0 ? item.highlightedBodyPreview : item.higlightedBody}</code
				></pre>
		{/if}
	</div>
</div>
