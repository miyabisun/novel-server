<script>
	import config from '$lib/config.js';
	import { navigate } from '$lib/router.svelte.js';
	import { checkAuth } from '$lib/auth.svelte.js';

	let username = $state('');
	let password = $state('');
	let error = $state('');
	let submitting = $state(false);

	async function handleSubmit(e) {
		e.preventDefault();
		submitting = true;
		error = '';
		try {
			const res = await fetch(`${config.path.api}/auth/login`, {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				credentials: 'include',
				body: JSON.stringify({ username, password }),
			});
			if (!res.ok) {
				error = 'ユーザー名またはパスワードが正しくありません';
				return;
			}
			await checkAuth();
			navigate('/');
		} catch {
			error = 'ログインに失敗しました';
		} finally {
			submitting = false;
		}
	}
</script>

<div class="login">
	<form onsubmit={handleSubmit}>
		<h1>ログイン</h1>
		{#if error}
			<p class="error">{error}</p>
		{/if}
		<label>
			<span>ユーザー名</span>
			<input type="text" bind:value={username} required autocomplete="username" />
		</label>
		<label>
			<span>パスワード</span>
			<input type="password" bind:value={password} required autocomplete="current-password" />
		</label>
		<button type="submit" disabled={submitting}>
			{submitting ? 'ログイン中...' : 'ログイン'}
		</button>
	</form>
</div>

<style lang="sass">
.login
	display: flex
	justify-content: center
	align-items: center
	min-height: 80vh

	form
		display: flex
		flex-direction: column
		gap: 16px
		padding: 32px
		border: 1px solid #555
		border-radius: 8px
		width: 320px

	h1
		margin: 0
		font-size: 1.4rem
		text-align: center

	label
		display: flex
		flex-direction: column
		gap: 4px

		span
			font-size: 0.9rem
			color: rgba(255, 255, 255, 0.6)

	input
		padding: 8px 12px
		border: 1px solid #555
		border-radius: 4px
		background: #333
		color: white
		font-size: 1rem

		&:focus
			outline: none
			border-color: rgba(128, 192, 255, 0.6)

	button
		padding: 10px
		border: 1px solid #555
		border-radius: 4px
		background: rgba(128, 192, 255, 0.2)
		color: white
		cursor: pointer
		font-size: 1rem

		&:hover:not(:disabled)
			background: rgba(128, 192, 255, 0.3)

		&:disabled
			opacity: 0.5
			cursor: not-allowed

	.error
		color: #ff6b6b
		margin: 0
		text-align: center
		font-size: 0.9rem
</style>
