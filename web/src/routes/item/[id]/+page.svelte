<script lang="ts">
	import { browser } from '$app/environment';
	import { highlightItem } from '$lib/shared';
	import Item from '$lib/Item.svelte';
	import type { PageData } from './$types';

	export let data: PageData;
</script>

<svelte:head>
	{#if browser}
		<title>Sink - #{data.id.toLocaleString()}</title>
	{:else}
		<title>Sink</title>
	{/if}
</svelte:head>

<div class="container p-2">
	<Item item={highlightItem(data)} />
</div>
