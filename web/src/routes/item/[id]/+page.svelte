<script lang="ts">
	import { highlightItem } from '$lib/shared';
	import Item from '$lib/Item.svelte';
	import type { PageData } from './$types';

	export let data: PageData;
</script>

<svelte:head>
	<title>Sink - #{data.id.toLocaleString()}</title>
</svelte:head>

<div class="container p-2">
	<Item item={highlightItem(data)} />
</div>
