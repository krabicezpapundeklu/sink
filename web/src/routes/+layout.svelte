<script lang="ts">
	import '../app.scss';
	import { onMount } from 'svelte';

	onMount(() => {
		const hasTz = document.cookie.split('; ').find((c) => c.startsWith('tz='));
		const tz = Intl.DateTimeFormat().resolvedOptions().timeZone;

		document.cookie = `tz=${tz};max-age=31536000;path=/;samesite=strict`;

		if (!hasTz) {
			window.location.reload();
		}
	});
</script>

<slot />
