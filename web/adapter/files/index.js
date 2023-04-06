import { render } from './server/main.js';

// eslint-disable-next-line no-undef, @typescript-eslint/no-unused-vars
globalThis.fetchData = function (path, search) {
	return Promise.resolve({
		data: JSON.stringify({
			id: 4,
			submitDate: '2023-03-23 17:16:55',
			system: 'test',
			type: null,
			headers: [
				{ name: 'accept', value: '*/*' },
				{ name: 'content-length', value: '1150' },
				{ name: 'content-type', value: 'application/json' },
				{ name: 'host', value: 'localhost:8080' },
				{ name: 'mgs-system-id', value: 'test' },
				{ name: 'user-agent', value: 'insomnia/2022.7.0' }
			],
			body: '{\n\t"name": "web",\n\t"version": "0.0.1",\n\t"private": true,\n\t"scripts": {\n\t\t"dev": "vite dev",\n\t\t"build": "vite build",\n\t\t"preview": "vite preview",\n\t\t"check": "svelte-kit sync && svelte-check --tsconfig ./tsconfig.json",\n\t\t"check:watch": "svelte-kit sync && svelte-check --tsconfig ./tsconfig.json --watch",\n\t\t"lint": "prettier --plugin-search-dir . --check . && eslint .",\n\t\t"format": "prettier --plugin-search-dir . --write ."\n\t},\n\t"devDependencies": {\n\t\t"@sveltejs/adapter-auto": "^2.0.0",\n\t\t"@sveltejs/adapter-node": "^1.2.3",\n\t\t"@sveltejs/adapter-static": "^2.0.1",\n\t\t"@sveltejs/kit": "^1.5.0",\n\t\t"@types/bootstrap": "^5.2.6",\n\t\t"@typescript-eslint/eslint-plugin": "^5.45.0",\n\t\t"@typescript-eslint/parser": "^5.45.0",\n\t\t"eslint": "^8.28.0",\n\t\t"eslint-config-prettier": "^8.5.0",\n\t\t"eslint-plugin-svelte3": "^4.0.0",\n\t\t"prettier": "^2.8.0",\n\t\t"prettier-plugin-svelte": "^2.8.1",\n\t\t"sass": "^1.58.3",\n\t\t"svelte": "^3.54.0",\n\t\t"svelte-check": "^3.0.1",\n\t\t"tslib": "^2.4.1",\n\t\t"typescript": "^4.9.3",\n\t\t"vite": "^4.0.0"\n\t},\n\t"type": "module",\n\t"dependencies": {\n\t\t"@popperjs/core": "^2.11.6",\n\t\t"bootstrap": "^5.2.3",\n\t\t"highlight.js": "^11.7.0"\n\t}\n}\n'
		})
	});
};

render('http://localhost/item/4')
	.then((response) => console.log(response))
	.catch((e) => console.log(`${e} ${e.stack}`));
