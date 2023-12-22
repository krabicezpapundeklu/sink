<script lang="ts">
	import { onMount } from 'svelte';
	import { highlightElement } from './shared';

	export let body: string;

	let code: HTMLElement;

	onMount(() => {
		highlightElement(code);
	});
</script>

<pre class="mb-0 overflow-hidden p-2 w-fc"><code class="p-0" bind:this={code}>{body}</code></pre>
