// eslint-disable-next-line no-undef
const module = globalThis;

module.crypto = {
	getRandomValues: () => {
		/* body not needed */
	}
};

module.Headers = function () {
	this.has = () => true;

	this.set = () => {
		/* body not needed */
	};
};

module.Response = function (body) {
	this.body = body;
};

module.TextEncoder = function () {
	this.encode = (str) => {
		const binstr = unescape(encodeURIComponent(str));
		const arr = new Uint8Array(binstr.length);

		const split = binstr.split('');

		for (let i = 0; i < split.length; i++) {
			arr[i] = split[i].charCodeAt(0);
		}

		return arr;
	};
};
