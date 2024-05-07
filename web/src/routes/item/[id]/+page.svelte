<script lang="ts">
	import { formatNumber } from '$lib/shared';
	import Item from '$lib/Item.svelte';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();
</script>

<svelte:head>
	<title>Sink - #{formatNumber(data.id)}</title>
</svelte:head>

<div class="vh-100">
	<Item item={data} preventDefault={false} />
</div>
