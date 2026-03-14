export { routes, matchRoute } from '$lib/routes.js';
import { matchRoute } from '$lib/routes.js';

export function getBasePath() {
	return (window.__BASE_PATH__ || '').replace(/\/+$/, '');
}

export function link(path) {
	return `${getBasePath()}${path}`;
}

let _routeIndex = $state(0);
let _params = $state({});

function getPathFromURL() {
	const base = getBasePath();
	let path = window.location.pathname;
	if (base && path.startsWith(base)) {
		path = path.slice(base.length) || '/';
	}
	return path;
}

function syncRoute() {
	const result = matchRoute(getPathFromURL());
	_routeIndex = result.index;
	_params = result.params;
}

function scrollMainToTop() {
	requestAnimationFrame(() => {
		window.scrollTo(0, 0);
	});
}

export function navigate(path) {
	history.pushState({}, '', getBasePath() + path);
	syncRoute();
	scrollMainToTop();
}

window.addEventListener('popstate', () => {
	syncRoute();
	scrollMainToTop();
});

// Initialize on load
syncRoute();

export const router = {
	get index() { return _routeIndex; },
	get params() { return _params; },
};
