import { render_route, set_public_env } from './server/main.js';

set_public_env({ PUBLIC_API_SERVER: 'http://localhost:8080' });

render_route('/item/814455', [
	null,
	{
		url: '/api/item/814455',
		data: {
			id: 814455,
			submitDate: '2023-02-14 12:44:51',
			system: 'qa415-va',
			type: 'event_notification',
			headers: [
				{ name: 'accept', value: '*/*' },
				{ name: 'connection', value: 'Keep-Alive' },
				{ name: 'content-length', value: '73' },
				{ name: 'content-type', value: 'application/json' },
				{ name: 'host', value: '10.0.1.141:8080' },
				{ name: 'mgs-system-id', value: 'qa415-va' },
				{ name: 'mgssystem', value: 'qa415-va' }
			],
			body: '{"entityEventId":19,"entityId":1719603,"eventDate":"2023-02-14T07:44:47"}'
		}
	}
])
	.then((response) => {
		console.log(response);
	})
	.catch((e) => {
		console.log(e);
	});
