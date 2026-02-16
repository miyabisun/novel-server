import { navigate } from '$lib/router.svelte.js';

export default (url, options = {}) =>
	fetch(url, { ...options, credentials: 'include' }).then((r) => {
		if (r.status === 401) {
			navigate('/login');
			throw new Error('Unauthorized');
		}
		if (!r.ok) throw new Error(`${r.status} ${r.statusText}`);
		return r.json();
	});
