import { appendFileSync, copyFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { nodeResolve } from '@rollup/plugin-node-resolve';
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

			appendFileSync(
				`${tmp}/index.js`,
				'export { compact, default_filter, default_preload, default_transform, get_option, options, render_response, set_public_env };'
			);

			writeFileSync(
				`${tmp}/manifest.js`,
				`export const manifest = ${builder.generateManifest({ relativePath: './' })};\n\n` +
					`export const prerendered = new Set(${JSON.stringify(builder.prerendered.paths)});\n`
			);

			copyFileSync(`${files}/main.js`, `${tmp}/main.js`);
			copyFileSync(`${files}/polyfills.js`, `${tmp}/polyfills.js`);

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

			copyFileSync(`${files}/index.js`, `${out}/index.js`);
		}
	};
}
