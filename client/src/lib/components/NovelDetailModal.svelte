<script>
	import config from '$lib/config.js';
	import fetcher from '$lib/fetcher.js';
	import { navigate } from '$lib/router.svelte.js';
	import { decodeHtml } from '$lib/decode.js';

	let { type, novel, isFav = false, onToggleFav, onclose } = $props();

	let synopsis = $state('');
	let loading = $state(true);
	let error = $state(null);
	let saving = $state(false);

	$effect(() => {
		document.body.style.overflow = 'hidden';
		return () => { document.body.style.overflow = ''; };
	});

	$effect(() => {
		loading = true;
		error = null;
		fetcher(`${config.path.api}/novel/${type}/${novel.id}/detail`)
			.then((data) => { synopsis = data.synopsis ?? ''; })
			.catch((e) => { error = e.message; })
			.finally(() => { loading = false; });
	});

	function handleKeydown(e) {
		if (e.key === 'Escape') onclose();
	}

	function handleBackdrop(e) {
		if (e.target === e.currentTarget) onclose();
	}

	function goRead() {
		const path = `/novel/${type}/${novel.id}/1`;
		onclose();
		navigate(path);
	}

	async function toggleFavorite() {
		if (saving) return;
		saving = true;
		try {
			if (isFav) {
				await fetcher(`${config.path.api}/favorites/${type}/${novel.id}`, { method: 'DELETE' });
			} else {
				await fetcher(`${config.path.api}/favorites/${type}/${novel.id}`, {
					method: 'PUT',
					headers: { 'Content-Type': 'application/json' },
					body: JSON.stringify({ title: novel.title, page: novel.page }),
				});
			}
			onToggleFav?.(novel.id);
		} catch (err) {
			alert(err.message);
		} finally {
			saving = false;
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="backdrop" onclick={handleBackdrop}>
	<div class="modal">
		<button class="close-btn" onclick={onclose}>&times;</button>
		<h2 class="title">{decodeHtml(novel.title)}</h2>

		{#if loading}
			<p class="status">読み込み中...</p>
		{:else if error}
			<p class="status error">{error}</p>
		{:else}
			<div class="synopsis">{synopsis || 'あらすじなし'}</div>
		{/if}

		<div class="actions">
			<button class="btn btn-primary" onclick={goRead}>第1話を読む</button>
			<button
				class="btn btn-fav"
				onclick={toggleFavorite}
				disabled={saving}
			>
				{#if saving}
					保存中...
				{:else if isFav}
					★ お気に入り解除
				{:else}
					☆ お気に入り追加
				{/if}
			</button>
		</div>
	</div>
</div>

<style lang="sass">
.backdrop
	position: fixed
	inset: 0
	background: var(--c-backdrop)
	z-index: 200
	display: flex
	align-items: center
	justify-content: center
	padding: var(--sp-5)

.modal
	background: var(--c-surface)
	border: 1px solid var(--c-border-strong)
	border-radius: var(--radius-lg)
	padding: var(--sp-5)
	max-width: 540px
	width: 100%
	max-height: 80vh
	overflow-y: auto
	position: relative

.close-btn
	position: absolute
	top: var(--sp-4)
	right: var(--sp-4)
	background: transparent
	border: none
	color: var(--c-text-sub)
	font-size: var(--fs-xl)
	cursor: pointer
	line-height: 1

	&:hover
		color: white

.title
	margin: 0 0 var(--sp-4)
	font-size: var(--fs-lg)
	color: white
	padding-right: 30px

.status
	color: var(--c-text-sub)
	text-align: center
	padding: var(--sp-5) 0

	&.error
		color: #ff6b6b

.synopsis
	white-space: pre-wrap
	color: var(--c-text)
	font-size: var(--fs-sm)
	line-height: 1.7
	max-height: 40vh
	overflow-y: auto
	margin-bottom: var(--sp-5)

.actions
	display: flex
	gap: var(--sp-3)

.btn
	padding: var(--sp-3) var(--sp-4)
	border: 1px solid var(--c-border-strong)
	border-radius: var(--radius-sm)
	cursor: pointer
	font-size: var(--fs-sm)

.btn-primary
	background: var(--c-accent-bg)
	color: var(--c-accent)
	border-color: var(--c-accent-border)

	&:hover
		background: var(--c-accent-bg-hover)

.btn-fav
	background: transparent
	color: var(--c-fav)
	border-color: var(--c-fav-border)

	&:hover:not(:disabled)
		background: var(--c-fav-hover)

	&:disabled
		cursor: default
		opacity: 0.7
</style>
