const el = document.createElement('textarea');

export function decodeHtml(str) {
	if (!str) return '';
	el.innerHTML = str;
	return el.value;
}
