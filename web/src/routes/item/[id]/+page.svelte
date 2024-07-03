<script lang="ts">
	import { building } from '$app/environment';
	import { formatNumber } from '$lib/shared';
	import Item from '$lib/Item.svelte';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();
</script>

<svelte:head>
	<title>Sink - #{building ? '#id#' : formatNumber(data.eventId ?? data.id)}</title>
</svelte:head>

<div class="vh-100">
	<Item item={data} preventDefault={false} />
</div>
