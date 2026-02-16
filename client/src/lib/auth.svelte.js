import config from '$lib/config.js';

let _authenticated = $state(false);
let _loading = $state(true);

export const auth = {
	get authenticated() { return _authenticated; },
	get loading() { return _loading; },
};

export async function checkAuth() {
	_loading = true;
	try {
		const res = await fetch(`${config.path.api}/auth/me`, { credentials: 'include' });
		_authenticated = res.ok;
	} catch {
		_authenticated = false;
	} finally {
		_loading = false;
	}
}

export async function logout() {
	await fetch(`${config.path.api}/auth/logout`, {
		method: 'POST',
		credentials: 'include',
	});
	_authenticated = false;
}
