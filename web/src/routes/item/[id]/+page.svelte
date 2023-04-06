<script lang="ts">
	import { browser } from '$app/environment';
	import { highlightItem } from '$lib/shared';
	import Item from '$lib/Item.svelte';
	import type { PageData } from './$types';

	export let data: PageData;

	let item = highlightItem(data);
</script>

<svelte:head>
	{#if browser}
		<title>Sink - #{data.id.toLocaleString()}</title>
	{:else}
		<title>Sink</title>
	{/if}
</svelte:head>

<div class="container p-2">
	{#if item}
		<Item {item} />
	{/if}
</div>
