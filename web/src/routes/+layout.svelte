<script lang="ts">
	import '../app.scss';
	import { onMount } from 'svelte';

	onMount(() => {
		const currentTz = document.cookie
			.split('; ')
			.filter((c) => c.startsWith('tz='))
			.map((c) => c.substring(3))[0];

		const tz = Intl.DateTimeFormat().resolvedOptions().timeZone;

		document.cookie = `tz=${tz};max-age=31536000;path=/;samesite=strict`;

		if (tz !== currentTz) {
			alert(
				`Your timezone appears to be "${tz}".\nThe page will now refresh to show dates in your local timezone.`
			);

			window.location.reload();
		}
	});
</script>

<slot />
