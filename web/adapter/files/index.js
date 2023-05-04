import { render } from './server/main.js';

// eslint-disable-next-line no-undef
const module = globalThis;

module.TIME_ZONE = 'Europe/London';

module.console.error = module.console.log;

module.fetchData = function (path) {
	let result = {};

	if (path === '/api/items') {
		result = {
			items: [
				{
					id: 814519,
					submitDate: '2023-04-13 11:37:05',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814518,
					submitDate: '2023-04-13 11:37:05',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814517,
					submitDate: '2023-04-13 11:37:04',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814516,
					submitDate: '2023-04-13 11:37:04',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814515,
					submitDate: '2023-04-13 11:37:03',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814514,
					submitDate: '2023-04-13 11:37:03',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814513,
					submitDate: '2023-04-13 11:37:02',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814512,
					submitDate: '2023-04-13 11:37:02',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814511,
					submitDate: '2023-04-13 11:37:01',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814510,
					submitDate: '2023-04-13 11:36:29',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814509,
					submitDate: '2023-04-13 11:36:28',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814508,
					submitDate: '2023-04-13 11:36:28',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814507,
					submitDate: '2023-04-13 11:36:27',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814506,
					submitDate: '2023-04-13 11:36:26',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814505,
					submitDate: '2023-04-13 11:36:26',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814504,
					submitDate: '2023-04-13 11:36:25',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814503,
					submitDate: '2023-04-13 11:36:25',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814502,
					submitDate: '2023-04-13 11:36:24',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814501,
					submitDate: '2023-04-13 11:36:24',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814500,
					submitDate: '2023-04-13 11:36:23',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814499,
					submitDate: '2023-04-13 11:36:23',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814498,
					submitDate: '2023-04-13 11:36:22',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814497,
					submitDate: '2023-04-13 11:36:22',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814496,
					submitDate: '2023-04-13 11:36:21',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814495,
					submitDate: '2023-04-13 11:36:21',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814494,
					submitDate: '2023-04-13 11:36:20',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814493,
					submitDate: '2023-04-13 11:36:19',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814492,
					submitDate: '2023-04-13 11:36:19',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814491,
					submitDate: '2023-04-13 11:36:18',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814490,
					submitDate: '2023-04-13 11:36:18',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814489,
					submitDate: '2023-04-13 11:36:17',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814488,
					submitDate: '2023-04-13 11:36:17',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814487,
					submitDate: '2023-04-13 11:36:16',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814486,
					submitDate: '2023-04-13 11:36:16',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814485,
					submitDate: '2023-04-13 11:36:15',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814484,
					submitDate: '2023-04-13 11:36:15',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814483,
					submitDate: '2023-04-13 11:36:14',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814482,
					submitDate: '2023-04-13 11:36:14',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814481,
					submitDate: '2023-04-13 11:36:13',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814480,
					submitDate: '2023-04-13 11:35:35',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814479,
					submitDate: '2023-04-13 11:35:34',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814478,
					submitDate: '2023-04-13 11:35:34',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814477,
					submitDate: '2023-04-13 11:35:33',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814476,
					submitDate: '2023-04-13 11:35:33',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814475,
					submitDate: '2023-04-13 11:35:32',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814474,
					submitDate: '2023-04-13 11:35:31',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814473,
					submitDate: '2023-04-13 11:35:31',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814472,
					submitDate: '2023-04-13 11:35:30',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814471,
					submitDate: '2023-04-13 11:35:30',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814470,
					submitDate: '2023-04-13 11:35:29',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814469,
					submitDate: '2023-04-13 11:35:29',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814468,
					submitDate: '2023-04-13 11:35:28',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814467,
					submitDate: '2023-04-13 11:35:28',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814466,
					submitDate: '2023-04-13 11:35:27',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814465,
					submitDate: '2023-04-13 11:35:27',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814464,
					submitDate: '2023-04-13 11:35:26',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814463,
					submitDate: '2023-04-13 11:35:25',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814462,
					submitDate: '2023-04-13 11:35:25',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814461,
					submitDate: '2023-04-13 11:35:24',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814460,
					submitDate: '2023-04-13 11:35:23',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814391,
					submitDate: '2023-02-10 23:33:31',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814390,
					submitDate: '2023-02-10 23:30:46',
					system: 'tc-nrc-254-1',
					type: 'application_created'
				},
				{
					id: 814389,
					submitDate: '2023-02-10 23:29:16',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814388,
					submitDate: '2023-02-10 23:26:31',
					system: 'tc-nrc-254-1',
					type: 'application_created'
				},
				{
					id: 814387,
					submitDate: '2023-02-10 23:25:01',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814386,
					submitDate: '2023-02-10 23:22:16',
					system: 'tc-nrc-254-1',
					type: 'application_created'
				},
				{
					id: 814385,
					submitDate: '2023-02-10 23:20:46',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814384,
					submitDate: '2023-02-10 23:18:01',
					system: 'tc-nrc-254-1',
					type: 'application_created'
				},
				{
					id: 814383,
					submitDate: '2023-02-10 23:16:31',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814382,
					submitDate: '2023-02-10 23:13:46',
					system: 'tc-nrc-254-1',
					type: 'application_created'
				},
				{
					id: 814381,
					submitDate: '2023-02-10 23:12:16',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814380,
					submitDate: '2023-02-10 23:09:31',
					system: 'tc-nrc-254-1',
					type: 'application_created'
				},
				{
					id: 814379,
					submitDate: '2023-02-10 23:08:01',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814378,
					submitDate: '2023-02-10 23:05:16',
					system: 'tc-nrc-254-1',
					type: 'application_created'
				},
				{
					id: 814377,
					submitDate: '2023-02-10 23:03:46',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814376,
					submitDate: '2023-02-10 23:01:01',
					system: 'tc-nrc-254-1',
					type: 'application_created'
				},
				{
					id: 814375,
					submitDate: '2023-02-10 22:59:31',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814374,
					submitDate: '2023-02-10 22:56:46',
					system: 'tc-nrc-254-1',
					type: 'application_created'
				},
				{
					id: 814373,
					submitDate: '2023-02-10 22:55:16',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814372,
					submitDate: '2023-02-10 22:52:31',
					system: 'tc-nrc-254-1',
					type: 'application_created'
				},
				{
					id: 814371,
					submitDate: '2023-02-10 22:51:01',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814370,
					submitDate: '2023-02-10 22:48:16',
					system: 'tc-nrc-254-1',
					type: 'application_created'
				},
				{
					id: 814369,
					submitDate: '2023-02-10 22:46:46',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814368,
					submitDate: '2023-02-10 22:44:06',
					system: 'tc-nrc-254-1',
					type: 'application_created'
				},
				{
					id: 814367,
					submitDate: '2023-02-10 22:42:36',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814366,
					submitDate: '2023-02-10 22:39:56',
					system: 'tc-nrc-254-1',
					type: 'application_created'
				},
				{
					id: 814365,
					submitDate: '2023-02-10 22:38:26',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814364,
					submitDate: '2023-02-10 22:35:46',
					system: 'tc-nrc-254-1',
					type: 'application_created'
				},
				{
					id: 814363,
					submitDate: '2023-02-10 22:34:16',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814362,
					submitDate: '2023-02-10 22:31:31',
					system: 'tc-nrc-254-1',
					type: 'application_created'
				},
				{
					id: 814361,
					submitDate: '2023-02-10 22:30:06',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814360,
					submitDate: '2023-02-10 22:27:16',
					system: 'tc-nrc-254-1',
					type: 'application_created'
				},
				{
					id: 814359,
					submitDate: '2023-02-10 22:25:51',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814358,
					submitDate: '2023-02-10 22:23:01',
					system: 'tc-nrc-254-1',
					type: 'application_created'
				},
				{
					id: 814357,
					submitDate: '2023-02-10 22:21:36',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814356,
					submitDate: '2023-02-10 22:18:46',
					system: 'tc-nrc-254-1',
					type: 'application_created'
				},
				{
					id: 814355,
					submitDate: '2023-02-10 22:17:21',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814354,
					submitDate: '2023-02-10 22:14:31',
					system: 'tc-nrc-254-1',
					type: 'application_created'
				},
				{
					id: 814353,
					submitDate: '2023-02-10 22:13:06',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				},
				{
					id: 814352,
					submitDate: '2023-02-10 22:10:17',
					system: 'tc-nrc-254-1',
					type: 'application_created'
				},
				{
					id: 814351,
					submitDate: '2023-02-10 22:08:51',
					system: 'tc-nrc-254-1',
					type: 'certificate_created'
				}
			],
			systems: [
				'qa-maint',
				'qa-maint-bopmp',
				'qa-maint-oobw',
				'qa-maint492',
				'qa414',
				'qa414-gsa',
				'qa414-oobw',
				'qa415-gsa',
				'qa415-va',
				'tc-fletc-1',
				'tc-fletc-2',
				'tc-nrc-254-1'
			],
			totalItems: 73071,
			filter: {
				query: null,
				system: null,
				type: null,
				from: null,
				to: null,
				asc: null,
				firstItemId: 0,
				lastItemId: 9007199254740991,
				batchSize: 100
			}
		};
	} else if (path === '/api/item/814519') {
		result = {
			id: 814519,
			submitDate: '2023-04-13 11:37:05',
			system: 'tc-nrc-254-1',
			type: 'certificate_created',
			headers: [
				{ name: 'accept', value: '*/*' },
				{ name: 'content-length', value: '1618' },
				{ name: 'content-type', value: 'application/xml' },
				{ name: 'host', value: 'localhost:8080' },
				{ name: 'user-agent', value: 'insomnia/2023.1.0' }
			],
			body: '<?xml version=\'1.0\' encoding=\'UTF-8\'?>\n<env:Envelope xmlns:env="http://schemas.xmlsoap.org/soap/envelope/" xmlns:xsd="http://www.w3.org/2001/XMLSchema" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" env:encodingStyle="">\n <env:Body>\n  <tns:createdCertificate xmlns:tns="http://www.monster.com/mgs/esb">\n   <certificate>\n    <id>421992</id>\n    <mgsSystem>tc-nrc-254-1</mgsSystem>\n    <vacancyId>35101</vacancyId>\n    <name>ALL OFC-2023-0028-02.09.2023MIKE1</name>\n    <location>Bridgeport, AL, US</location>\n    <gradesList>\n     <grade>09</grade>\n    </gradesList>\n    <seriesList>\n     <series>0201</series>\n    </seriesList>\n    <listType>1</listType>\n    <certificateAppointmentType>Permanent</certificateAppointmentType>\n    <dualCertOption>1</dualCertOption>\n    <dateReceivedHR/>\n    <randomNumber>2</randomNumber>\n    <applyVetPref>0</applyVetPref>\n    <sortByLastname>false</sortByLastname>\n    <sortByScore>true</sortByScore>\n    <rule>0</rule>\n    <status>0</status>\n    <issueDate>2023-02-09T10:42:51-05:00</issueDate>\n    <expirationDate>2023-05-10T23:59:59-04:00</expirationDate>\n    <statusChangeDate/>\n    <signedDate>2023-02-10T09:15:55-05:00</signedDate>\n    <certificateApplicationsList>\n     <certificateApplication>\n      <vacancyId>35101</vacancyId>\n      <applicantId>10</applicantId>\n      <gradeId>09</gradeId>\n      <applicationRank>1</applicationRank>\n      <status/>\n      <totalScore>10.0</totalScore>\n      <vetPrefPoints>31</vetPrefPoints>\n     </certificateApplication>\n    </certificateApplicationsList>\n   </certificate>\n  </tns:createdCertificate>\n </env:Body>\n</env:Envelope>'
		};
	}

	return Promise.resolve({
		data: JSON.stringify(result)
	});
};

render('http://localhost/')
	.then((response) => console.log(response))
	.catch((e) => console.log(`${e} ${e.stack}`));
