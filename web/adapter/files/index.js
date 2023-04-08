import { render } from './server/main.js';

// eslint-disable-next-line no-undef
const module = globalThis;

module.TIME_ZONE = 'Europe/London';

module.console.error = module.console.log;

module.fetchData = function () {
	return Promise.resolve({
		data: JSON.stringify({
			id: 1000000,
			submitDate: '2023-04-08 13:50:39',
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
			body: '{}'
		})
	});
};

render('http://localhost/item/4')
	.then((response) => console.log(response))
	.catch((e) => console.log(`${e} ${e.stack}`));
