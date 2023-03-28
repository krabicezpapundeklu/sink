globalThis.crypto = {
	getRandomValues: () => {}
};

globalThis.Headers = function () {
	(this.has = () => true), (this.set = () => {});
};

globalThis.Response = function (body) {
	this.body = body;
};

globalThis.TextEncoder = function () {
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
