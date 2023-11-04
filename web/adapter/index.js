import { writeFileSync } from 'node:fs';
import { PurgeCSS } from 'purgecss';

export default function () {
	return {
		name: 'adapter',

		async adapt(builder) {
			const out = 'build';
			const tmp = builder.getBuildDirectory('adapter');

			builder.rimraf(out);
			builder.rimraf(tmp);
			builder.mkdirp(tmp);

			builder.log.minor('Copying assets');

			builder.writeClient(`${out}/client${builder.config.kit.paths.base}`);

			await builder.generateFallback(`${out}/client/index.html`);

			const purgeCSSResult = await new PurgeCSS().purge('./purgecss.config.cjs');

			for (const purge of purgeCSSResult) {
				writeFileSync(purge.file, purge.css);
			}
		}
	};
}
