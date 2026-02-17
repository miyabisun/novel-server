export default (url, options = {}) =>
	fetch(url, options).then((r) => {
		if (!r.ok) throw new Error(`${r.status} ${r.statusText}`);
		return r.json();
	});
