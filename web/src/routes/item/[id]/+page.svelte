<script lang="ts">
	import { page } from '$app/stores';

	import Item from '$lib/Item.svelte';

	let id = +$page.params.id;
</script>

<svelte:head>
	<title>Sink - #{id.toLocaleString()}</title>
</svelte:head>

<div class="container p-2">
	<Item {id} />
</div>
