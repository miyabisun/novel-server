<script>
	import config from '$lib/config.js';
	import fetcher from '$lib/fetcher.js';
	import { addFavorite, removeFavorite } from '$lib/favorites.js';
	import { navigate } from '$lib/router.svelte.js';
	import { decodeHtml } from '$lib/decode.js';
	import Icon from '$lib/components/Icon.svelte';

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

	function goToc() {
		onclose();
		navigate(`/novel/${type}/${novel.id}/toc`);
	}

	async function toggleFavorite() {
		if (saving) return;
		saving = true;
		try {
			if (isFav) {
				await removeFavorite(type, novel.id);
			} else {
				await addFavorite(type, novel.id, novel);
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
		<button class="close-btn" onclick={onclose} aria-label="閉じる"><Icon name="x" /></button>
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
			<button class="btn btn-toc" onclick={goToc}>目次</button>
			<button
				class="btn btn-fav"
				onclick={toggleFavorite}
				disabled={saving}
			>
				{#if saving}
					保存中...
				{:else if isFav}
					<Icon name="star-filled" /> お気に入り解除
				{:else}
					<Icon name="star-outline" /> お気に入り追加
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
	padding: var(--sp-xl)

.modal
	background: var(--c-surface)
	border: 1px solid var(--c-border)
	border-radius: var(--radius-lg)
	padding: var(--sp-lg)
	max-width: 540px
	width: 100%
	max-height: 80vh
	overflow-y: auto
	position: relative
	box-shadow: 0 8px 32px rgba(0, 0, 0, 0.25)

.close-btn
	position: absolute
	top: var(--sp-sm)
	right: var(--sp-sm)
	width: 36px
	height: 36px
	background: transparent
	border: none
	border-radius: var(--radius-sm)
	color: var(--c-text-muted)
	cursor: pointer
	line-height: 1
	display: flex
	align-items: center
	justify-content: center

	&:hover
		color: var(--c-text)

.title
	margin: 0 0 var(--sp-lg)
	font-size: var(--fs-title)
	font-weight: 600
	color: var(--c-text)
	padding-right: 40px

.synopsis
	white-space: pre-wrap
	color: var(--c-text)
	font-size: var(--fs-body-sm)
	line-height: 1.7
	max-height: 40vh
	overflow-y: auto
	margin-bottom: var(--sp-lg)

.actions
	display: flex
	gap: var(--sp-sm)

.btn
	padding: var(--sp-sm) var(--sp-lg)
	border: 1px solid var(--c-border)
	border-radius: var(--radius-sm)
	cursor: pointer
	font-size: var(--fs-label)
	font-weight: 500
	display: inline-flex
	align-items: center
	gap: var(--sp-xs)

.btn-primary
	background: var(--c-accent)
	color: var(--c-surface)
	border-color: var(--c-accent)

	&:hover
		background: var(--c-accent-hover)
		border-color: var(--c-accent-hover)

.btn-toc
	background: transparent
	color: var(--c-text-muted)

	&:hover
		background: var(--c-border)

.btn-fav
	background: transparent
	color: var(--c-fav)
	border-color: var(--c-fav-border)

	&:hover:not(:disabled)
		background: var(--c-fav-hover)

	&:disabled
		cursor: default
		opacity: 0.5
</style>
