import 'core-js/actual/url';
import 'web-streams-polyfill/es2018';
import 'whatwg-fetch';
import { TextEncoder } from 'text-encoding-utf-8';

// eslint-disable-next-line no-undef
const module = globalThis;

module.crypto = {
	getRandomValues: () => {
		/* body not needed */
	}
};

module.TextEncoder = TextEncoder;

Array.prototype.at = function (index) {
	if (index < 0) {
		return this[this.length + index];
	}

	return this[index];
};
