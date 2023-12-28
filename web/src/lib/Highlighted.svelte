<script context="module" lang="ts">
	import hljs from 'highlight.js/lib/core';
	import json from 'highlight.js/lib/languages/json';
	import plaintext from 'highlight.js/lib/languages/plaintext';
	import xml from 'highlight.js/lib/languages/xml';

	hljs.registerLanguage('json', json);
	hljs.registerLanguage('plaintext', plaintext);
	hljs.registerLanguage('xml', xml);
</script>

<script lang="ts">
	import { onMount } from 'svelte';

	export let body: string;

	let code: HTMLElement;

	onMount(() => {
		hljs.highlightElement(code);
	});
</script>

<pre class="mb-0 overflow-hidden p-2 w-fc"><code class="p-0" bind:this={code}>{body}</code></pre>
