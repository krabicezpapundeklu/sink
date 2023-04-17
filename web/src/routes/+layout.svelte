<script lang="ts">
	import '../app.scss';
	import { onMount } from 'svelte';

	onMount(() => {
		const tz = Intl.DateTimeFormat().resolvedOptions().timeZone;
		document.cookie = `tz=${tz};max-age=31536000;path=/;samesite=strict`;
	});
</script>

<slot />
