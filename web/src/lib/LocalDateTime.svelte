<script lang="ts">
	const MILLISECONDS_IN_DAY = 24 * 60 * 60 * 1000;

	let { dateTime, detail = false }: { dateTime: string; detail?: boolean } = $props();

	let localDateTime = $derived.by(() => {
		const isoDate = new Date(dateTime.substring(0, 10) + 'T' + dateTime.substring(11) + '.000Z');

		let formatOptions: Intl.DateTimeFormatOptions;

		if (detail) {
			formatOptions = { dateStyle: 'full', timeStyle: 'medium' };
		} else {
			formatOptions = { dateStyle: 'short', timeStyle: 'short' };

			const now = new Date();

			if (now.getTime() - isoDate.getTime() < MILLISECONDS_IN_DAY) {
				const dayFormat = Intl.DateTimeFormat('en-us', { dateStyle: 'short' });

				if (dayFormat.format(now) === dayFormat.format(isoDate)) {
					formatOptions = { timeStyle: 'short' };
				}
			}
		}

		return Intl.DateTimeFormat('en-us', formatOptions).format(isoDate);
	});
</script>

{localDateTime}
