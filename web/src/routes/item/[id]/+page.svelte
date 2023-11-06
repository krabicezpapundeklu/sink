<script lang="ts">
	import { formatNumber, highlightItem } from '$lib/shared';
	import Item from '$lib/Item.svelte';
	import type { PageData } from './$types';

	export let data: PageData;
</script>

<svelte:head>
	<title>Sink - #{formatNumber(data.id)}</title>
</svelte:head>

<div class="vh-100">
	<Item item={highlightItem(data)} preventDefault={false} />
</div>
