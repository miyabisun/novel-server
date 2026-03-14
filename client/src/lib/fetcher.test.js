import { describe, it, expect, vi, beforeEach } from 'vitest';
import fetcher from './fetcher.js';

function mockFetch(status, body, ok = undefined) {
	return vi.fn(() =>
		Promise.resolve({
			ok: ok ?? (status >= 200 && status < 300),
			status,
			statusText: 'Status Text',
			json: () => Promise.resolve(body),
		})
	);
}

beforeEach(() => {
	vi.restoreAllMocks();
});

describe('fetcher', () => {
	it('returns parsed JSON on success', async () => {
		globalThis.fetch = mockFetch(200, { data: 'ok' });
		const result = await fetcher('/api/test');
		expect(result).toEqual({ data: 'ok' });
	});

	it('passes options to fetch', async () => {
		globalThis.fetch = mockFetch(200, {});
		const opts = { method: 'POST', body: '{}' };
		await fetcher('/api/test', opts);
		expect(globalThis.fetch).toHaveBeenCalledWith('/api/test', opts);
	});

	it('throws friendly message for 502', async () => {
		globalThis.fetch = mockFetch(502, {});
		await expect(fetcher('/api/test')).rejects.toThrow(
			'サイトに接続できませんでした'
		);
	});

	it('throws friendly message for 503', async () => {
		globalThis.fetch = mockFetch(503, {});
		await expect(fetcher('/api/test')).rejects.toThrow(
			'サイトが一時的に利用できません'
		);
	});

	it('throws friendly message for 504', async () => {
		globalThis.fetch = mockFetch(504, {});
		await expect(fetcher('/api/test')).rejects.toThrow(
			'タイムアウトしました'
		);
	});

	it('throws error field from response body for other errors', async () => {
		globalThis.fetch = mockFetch(400, { error: 'Invalid type' });
		await expect(fetcher('/api/test')).rejects.toThrow('Invalid type');
	});

	it('falls back to status text when body has no error field', async () => {
		globalThis.fetch = mockFetch(418, {});
		await expect(fetcher('/api/test')).rejects.toThrow('418 Status Text');
	});

	it('falls back to status text when body is not JSON', async () => {
		globalThis.fetch = vi.fn(() =>
			Promise.resolve({
				ok: false,
				status: 500,
				statusText: 'Internal Server Error',
				json: () => Promise.reject(new Error('not json')),
			})
		);
		await expect(fetcher('/api/test')).rejects.toThrow(
			'500 Internal Server Error'
		);
	});
});
