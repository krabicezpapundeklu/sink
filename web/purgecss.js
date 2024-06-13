import { basename, dirname, join } from 'path';
import { createHash } from 'crypto';
import { PurgeCSS } from 'purgecss';
import { readFileSync, readdirSync, renameSync, statSync, unlinkSync, writeFileSync } from 'fs';

const walkSync = (dir, callback) => {
	const files = readdirSync(dir);

	files.forEach((file) => {
		const filepath = join(dir, file);
		const stats = statSync(filepath);

		if (stats.isDirectory()) {
			walkSync(filepath, callback);
		} else if (stats.isFile()) {
			callback(filepath);
		}
	});
};

let purgedFiles = [];

const purgeCSSResult = await new PurgeCSS().purge({
	content: ['./build/**/*.js'],
	css: ['./build/**/*.css'],
	variables: true
});

for (const purge of purgeCSSResult) {
	const hash = createHash('md5').update(purge.css).digest('hex');
	const oldFile = purge.file;
	const newFile = join(dirname(oldFile), `styles-${hash}.css`);

	writeFileSync(oldFile, purge.css);
	renameSync(oldFile, newFile);

	console.log(`purged ${oldFile} -> ${newFile}`);

	purgedFiles.push({
		oldFile: basename(oldFile),
		newFile: basename(newFile),
		newFileFullPath: newFile,
		used: false
	});
}

walkSync('./build', (file) => {
	if (file.endsWith('.css') || file.endsWith('.html') || file.endsWith('.js')) {
		const data = readFileSync(file, { encoding: 'utf8', flag: 'r' });
		let newData = data;

		for (const purgedFile of purgedFiles) {
			const currentData = newData;
			newData = newData.replace(purgedFile.oldFile, purgedFile.newFile);

			if (newData !== currentData) {
				purgedFile.used = true;
			}
		}

		if (newData != data) {
			writeFileSync(file, newData);
			console.log(`updated ${file}`);
		}
	}
});

for (const purgedFile of purgedFiles) {
	if (!purgedFile.used) {
		const file = purgedFile.newFileFullPath;
		console.log(`unused ${file}`);
		unlinkSync(file);
	}
}
