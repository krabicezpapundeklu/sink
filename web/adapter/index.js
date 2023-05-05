import { copyFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { nodeResolve } from '@rollup/plugin-node-resolve';
import { PurgeCSS } from 'purgecss';
import { rollup } from 'rollup';
import commonjs from '@rollup/plugin-commonjs';
import json from '@rollup/plugin-json';

const files = fileURLToPath(new URL('./files', import.meta.url).href);

/** @type {import('.').default} */
export default function (opts = {}) {
	const { out = 'build', precompress } = opts;

	return {
		name: 'adapter',

		async adapt(builder) {
			const tmp = builder.getBuildDirectory('adapter');

			builder.rimraf(out);
			builder.rimraf(tmp);
			builder.mkdirp(tmp);

			builder.log.minor('Copying assets');

			builder.writeClient(`${out}/client${builder.config.kit.paths.base}`);
			builder.writePrerendered(`${out}/prerendered${builder.config.kit.paths.base}`);

			if (precompress) {
				builder.log.minor('Compressing assets');

				await Promise.all([
					builder.compress(`${out}/client`),
					builder.compress(`${out}/prerendered`)
				]);
			}

			builder.log.minor('Building server');

			builder.writeServer(tmp);

			writeFileSync(
				`${tmp}/manifest.js`,
				`export const manifest = ${builder.generateManifest({ relativePath: './' })};\n\n` +
					`export const prerendered = new Set(${JSON.stringify(builder.prerendered.paths)});\n`
			);

			copyFileSync(`${files}/main.js`, `${tmp}/main.js`);
			copyFileSync(`${files}/polyfills-0.js`, `${tmp}/polyfills-0.js`);
			copyFileSync(`${files}/polyfills-1.js`, `${tmp}/polyfills-1.js`);

			const bundle = await rollup({
				input: {
					main: `${tmp}/main.js`
				},
				plugins: [
					nodeResolve({
						preferBuiltins: true,
						exportConditions: ['node']
					}),
					commonjs({ strictRequires: true }),
					json()
				]
			});

			await bundle.write({
				dir: `${out}/server`,
				format: 'esm',
				inlineDynamicImports: true
			});

			const purgeCSSResult = await new PurgeCSS().purge({
				content: [`${out}/client/**/*.js`],
				css: [`${out}/client/**/*.css`],
				safelist: ['opacity-25', 'w-25', /popper$/, /^hljs/]
			});

			for (const purge of purgeCSSResult) {
				writeFileSync(purge.file, purge.css);
			}
		}
	};
}
