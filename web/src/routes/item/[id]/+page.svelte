<script lang="ts">
	import Item from '$lib/Item.svelte';
	import type { PageData } from './$types';

	export let data: PageData;
</script>

<svelte:head>
	<title>Sink - #{data.item.id.toLocaleString()}</title>
</svelte:head>

<div class="container p-2">
	<Item {...data} />
</div>
