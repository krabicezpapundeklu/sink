<script lang="ts">
	import { onMount } from 'svelte';
	import { Prism } from '../prism';

	let { body, language }: { body: string; language: string } = $props();

	let code: HTMLElement;

	onMount(() => {
		Prism.highlightElement(code);
	});
</script>

<pre class="m-0 overflow-hidden p-2 w-fc"><code class="p-0 language-{language}" bind:this={code}
		>{body}</code
	></pre>
