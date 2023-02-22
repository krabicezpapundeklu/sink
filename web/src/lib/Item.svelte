<script lang="ts">
	import json from 'highlight.js/lib/languages/json';
	import plaintext from 'highlight.js/lib/languages/plaintext';
	import xml from 'highlight.js/lib/languages/xml';
	import hljs from 'highlight.js/lib/core';

	import { formatJson, formatXml, itemTypeToName, utcDateStringToLocalString } from './utils';
	import { getItem } from './api';

	import type { Item } from './model';

	hljs.registerLanguage('json', json);
	hljs.registerLanguage('plaintext', plaintext);
	hljs.registerLanguage('xml', xml);

	export let id: number;

	let item: Promise<Item>;
	let bodyPreview: string;
	let originalBody: string;

	let activeTab = 0;

	$: {
		item = getItem(id);

		item.then((value: Item): void => {
			let language = 'plaintext';
			let formattedBody = value.body;

			for (const header of value.headers) {
				if (header.name === 'content-type') {
					if (header.value.indexOf('json') !== -1) {
						language = 'json';
						formattedBody = formatJson(value.body);
					} else if (header.value.indexOf('xml') !== -1) {
						language = 'xml';
						formattedBody = formatXml(value.body);
					}

					break;
				}
			}

			bodyPreview = hljs.highlight(formattedBody, { language }).value;
			originalBody = hljs.highlight(value.body, { language }).value;
		});
	}
</script>

{#await item}
	<!-- loading -->
{:then item}
	<div>
		<span class="fs-3 me-3">{item.id.toLocaleString()}</span>
		<span>{new Date(utcDateStringToLocalString(item.submitDate)).toLocaleString()}</span>
		<div>
			{#if item.system}
				<span class="badge bg-secondary">{item.system}</span>
			{/if}
			{#if item.type}
				<span class="badge {item.type}">{itemTypeToName(item.type)}</span>
			{/if}
		</div>
	</div>
	<ul class="mt-3 nav nav-tabs">
		<li class="nav-item">
			<!-- svelte-ignore a11y-invalid-attribute -->
			<a
				class="nav-link"
				class:active={activeTab === 0}
				href="javascript:void(0)"
				on:click={() => (activeTab = 0)}>Body Preview</a
			>
		</li>
		<li class="nav-item">
			<!-- svelte-ignore a11y-invalid-attribute -->
			<a
				class="nav-link"
				class:active={activeTab === 1}
				href="javascript:void(0)"
				on:click={() => (activeTab = 1)}>Original Body</a
			>
		</li>
		<li class="nav-item">
			<!-- svelte-ignore a11y-invalid-attribute -->
			<a
				class="nav-link"
				class:active={activeTab === 2}
				href="javascript:void(0)"
				on:click={() => (activeTab = 2)}>Headers</a
			>
		</li>
	</ul>
	<div class="bg-white">
		{#if activeTab === 2}
			<table class="m-0 table table-bordered table-sm">
				<thead>
					<tr class="border-top-0">
						<th scope="col">Name</th>
						<th scope="col">Value</th>
					</tr>
				</thead>
				<tbody>
					{#each item.headers as header}
						<tr>
							<td>{header.name}</td>
							<td>{header.value}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		{:else}
			<div class="border border-top-0 p-2">
				<pre class="mb-0"><code>{@html activeTab === 0 ? bodyPreview : originalBody}</code></pre>
			</div>
		{/if}
	</div>
{/await}
