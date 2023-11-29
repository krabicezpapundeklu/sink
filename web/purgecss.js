import { readFileSync, readdirSync, renameSync, statSync, writeFileSync } from 'fs';
import { basename, dirname, join } from 'path';
import { PurgeCSS } from 'purgecss';

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
	safelist: [/^hljs/],
	variables: true
});

for (const purge of purgeCSSResult) {
	const oldFile = purge.file;
	const newFile = join(dirname(oldFile), new Date().getTime() + '-' + purgedFiles.length + '.css');

	writeFileSync(oldFile, purge.css);
	renameSync(oldFile, newFile);

	console.log(`purged ${oldFile} -> ${newFile}`);

	purgedFiles.push({
		oldFile: basename(oldFile),
		newFile: basename(newFile)
	});
}

walkSync('./build', (file) => {
	if (file.endsWith('.css') || file.endsWith('.html') || file.endsWith('.js')) {
		const data = readFileSync(file, { encoding: 'utf8', flag: 'r' });
		let newData = data;

		for (const purgedFile of purgedFiles) {
			newData = newData.replace(purgedFile.oldFile, purgedFile.newFile);
		}

		if (newData != data) {
			writeFileSync(file, newData);
			console.log(`updated ${file}`);
		}
	}
});
