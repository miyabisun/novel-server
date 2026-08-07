<script>
	import config from '$lib/config.js';
	import fetcher from '$lib/fetcher.js';
	import { addFavorite as addFavoriteRequest, removeFavorite as removeFavoriteRequest } from '$lib/favorites.js';
	import { navigate } from '$lib/router.svelte.js';
	import { decodeHtml } from '$lib/decode.js';
	import ConfirmModal from '$lib/components/ConfirmModal.svelte';
	import Icon from '$lib/components/Icon.svelte';
	import { clearReaderChrome, updateReaderChrome } from '$lib/readerChrome.svelte.js';

	let { params } = $props();
	let html = $state('');
	let loading = $state(false);
	let error = $state(null);
	let title = $state('');
	let totalPages = $state(0);
	let isFav = $state(false);
	let favSaving = $state(false);
	let showUnfavConfirm = $state(false);
	let swipeDir = $state(null);
	let swipeReady = $state(false);
	let currentNum = $derived(Number(params.num));
	let canGoPrev = $derived(currentNum > 1);
	let canGoNext = $derived(!totalPages || currentNum < totalPages);
	// Holds the in-flight loadMeta() so loadPage() can wait for isFav before patching progress.
	// The metaPromise effect must stay declared before the loadPage effect (Svelte $effects run in source order).
	let metaPromise = null;

	async function loadPage(type, id, num) {
		const meta = metaPromise;
		loading = true;
		error = null;
		try {
			const data = await fetcher(`${config.path.api}/novel/${type}/${id}/pages/${num}`);
			html = data.html || '';
			await meta;
			updateProgress(type, id, num);
		} catch (e) {
			error = e.message;
			html = '';
		} finally {
			loading = false;
		}
	}

	function goTo(num) {
		if (num < 1) return;
		if (totalPages && num > totalPages) return;
		navigate(`/novel/${params.type}/${params.id}/${num}`);
	}

	function handleKeydown(e) {
		if (showUnfavConfirm) return;
		if (e.key === 'ArrowLeft') {
			e.preventDefault();
			goTo(currentNum - 1);
		} else if (e.key === 'ArrowRight') {
			e.preventDefault();
			goTo(currentNum + 1);
		} else if (e.key === 'Backspace') {
			e.preventDefault();
			navigate('/');
		}
	}

	function updateProgress(type, id, num) {
		if (!isFav) return;
		fetcher(`${config.path.api}/favorites/${type}/${id}/progress`, {
			method: 'PATCH',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ read: Number(num) }),
		}).catch(() => {});
	}

	async function loadMeta(type, id) {
		const [detailResult, favResult] = await Promise.allSettled([
			fetcher(`${config.path.api}/novel/${type}/${id}/detail`),
			fetcher(`${config.path.api}/favorites`),
		]);
		if (detailResult.status === 'fulfilled') {
			title = decodeHtml(detailResult.value.title || '');
		}
		if (favResult.status === 'fulfilled') {
			const fav = favResult.value.find((f) => f.type === type && f.id === id);
			isFav = !!fav;
			totalPages = fav?.page || (detailResult.status === 'fulfilled' ? detailResult.value.page : 0) || 0;
		} else {
			isFav = false;
			if (detailResult.status === 'fulfilled') {
				totalPages = detailResult.value.page || 0;
			}
		}
	}

	function handleFavClick() {
		if (isFav) {
			showUnfavConfirm = true;
		} else {
			addFavorite();
		}
	}

	async function addFavorite() {
		if (favSaving) return;
		favSaving = true;
		try {
			await addFavoriteRequest(params.type, params.id, { title, page: totalPages, read: currentNum });
			isFav = true;
		} catch (err) {
			alert(err.message);
		} finally {
			favSaving = false;
		}
	}

	async function executeUnfav() {
		showUnfavConfirm = false;
		if (favSaving) return;
		favSaving = true;
		try {
			await removeFavoriteRequest(params.type, params.id);
			isFav = false;
		} catch (err) {
			alert(err.message);
		} finally {
			favSaving = false;
		}
	}


	$effect(() => {
		document.title = title
			? `${currentNum}話 ${title} | novel-server`
			: 'novel-server';
		return () => { document.title = 'novel-server'; };
	});

	$effect(() => {
		metaPromise = loadMeta(params.type, params.id);
	});

	$effect(() => {
		loadPage(params.type, params.id, params.num);
	});

	$effect(() => {
		window.addEventListener('keydown', handleKeydown);
		return () => window.removeEventListener('keydown', handleKeydown);
	});

	// Compact viewports surface 目次 / unfav via the global hamburger instead of the bar.
	$effect(() => {
		updateReaderChrome({
			active: true,
			showToc: totalPages > 1,
			showUnfav: isFav,
			goToc: () => navigate(`/novel/${params.type}/${params.id}/toc`),
			requestUnfav: () => {
				if (isFav) showUnfavConfirm = true;
			},
		});
		return () => clearReaderChrome();
	});

	function readerSwipe(node) {
		let startX, startY, locked, horizontal;

		function onStart(e) {
			const touch = e.touches[0];
			startX = touch.clientX;
			startY = touch.clientY;
			locked = false;
			horizontal = false;
			swipeDir = null;
			swipeReady = false;
		}

		function onMove(e) {
			const touch = e.touches[0];
			const dx = touch.clientX - startX;
			const dy = touch.clientY - startY;

			if (!locked) {
				if (Math.abs(dx) < 5 && Math.abs(dy) < 5) return;
				locked = true;
				horizontal = Math.abs(dx) > Math.abs(dy);
			}
			if (!horizontal) return;
			e.preventDefault();
			const dir = dx < 0 ? 'next' : 'prev';
			swipeDir = dir;
			const canSwipe = dir === 'next' ? canGoNext : canGoPrev;
			swipeReady = canSwipe && Math.abs(dx) >= 50;
		}

		function onEnd(e) {
			if (locked && horizontal) {
				const dx = e.changedTouches[0].clientX - startX;
				if (dx < -50) goTo(currentNum + 1);
				else if (dx > 50) goTo(currentNum - 1);
			}
			swipeDir = null;
			swipeReady = false;
		}

		node.addEventListener('touchstart', onStart, { passive: true });
		node.addEventListener('touchmove', onMove, { passive: false });
		node.addEventListener('touchend', onEnd, { passive: true });

		return {
			destroy() {
				node.removeEventListener('touchstart', onStart);
				node.removeEventListener('touchmove', onMove);
				node.removeEventListener('touchend', onEnd);
			},
		};
	}
</script>

<nav class="reader-bar top">
	<div class="bar-title">{title || params.id}</div>
	<div class="bar-right">
		<span class="bar-page">{currentNum}{#if totalPages}/{totalPages}{/if}</span>
		<button class="nav-btn" onclick={() => goTo(currentNum - 1)} disabled={!canGoPrev}>前</button>
		<button class="toc-btn" onclick={() => navigate(`/novel/${params.type}/${params.id}/toc`)} disabled={totalPages <= 1}>目次</button>
		<button class="nav-btn" onclick={() => goTo(currentNum + 1)} disabled={!canGoNext}>次</button>
		{#if isFav}
			<button class="fav-btn-remove" onclick={handleFavClick} disabled={favSaving || !title} aria-label="お気に入りから削除"><Icon name="x" /></button>
		{:else}
			<button class="fav-btn" onclick={handleFavClick} disabled={favSaving || !title} aria-label="お気に入りに追加"><Icon name="star-outline" /></button>
		{/if}
	</div>
</nav>

{#if swipeDir === 'prev'}
	<div class="swipe-hint left" class:ready={swipeReady} class:disabled={!canGoPrev}>{#if canGoPrev}‹ 前へ{:else}<del>‹ 前へ</del>{/if}</div>
{/if}
{#if swipeDir === 'next'}
	<div class="swipe-hint right" class:ready={swipeReady} class:disabled={!canGoNext}>{#if canGoNext}次へ ›{:else}<del>次へ ›</del>{/if}</div>
{/if}

<div class="reader" use:readerSwipe>
	{#if loading}
		<p class="status">読み込み中...</p>
	{:else if error}
		<p class="status error">{error}</p>
		<p class="status"><button class="nav-btn" onclick={() => loadPage(params.type, params.id, params.num)}>再読み込み</button></p>
	{:else}
		<article class="content">
			{@html html}
		</article>
	{/if}
</div>

{#if showUnfavConfirm}
	<ConfirmModal
		message={`「${title}」をお気に入りから削除しますか？`}
		onconfirm={executeUnfav}
		oncancel={() => showUnfavConfirm = false}
	/>
{/if}

<style lang="sass">
.reader
	padding: 0 var(--sp-lg)
	max-width: 800px
	margin: 0 auto

.reader-bar
	display: flex
	align-items: center
	justify-content: space-between
	min-height: var(--subheader-h)
	padding: var(--sp-sm) 2.5%
	background: var(--c-surface)
	border-bottom: 1px solid var(--c-border)
	gap: var(--sp-sm)
	z-index: 50

	&.top
		position: sticky
		top: var(--header-h)

.bar-title
	color: var(--c-text-muted)
	font-size: var(--fs-label)
	white-space: nowrap
	overflow: hidden
	text-overflow: ellipsis
	min-width: 0
	flex: 1

.bar-right
	display: flex
	align-items: center
	gap: var(--sp-xs)
	flex-shrink: 0

.bar-page
	color: var(--c-text-muted)
	font-size: var(--fs-caption)
	margin-right: var(--sp-xs)
	white-space: nowrap

.nav-btn
	padding: var(--sp-xs) var(--sp-lg)
	border: 1px solid var(--c-border)
	background: transparent
	color: var(--c-text-muted)
	cursor: pointer
	border-radius: var(--radius-sm)
	font-size: var(--fs-label)
	font-weight: 500

	&:hover:not(:disabled)
		background: var(--c-border)

	&:disabled
		opacity: 0.5
		cursor: not-allowed

	@media (max-width: 768px)
		.bar-right > &
			display: none

.toc-btn
	padding: var(--sp-xs) var(--sp-md)
	border: 1px solid var(--c-border)
	background: transparent
	color: var(--c-text-muted)
	cursor: pointer
	border-radius: var(--radius-sm)
	font-size: var(--fs-label)
	font-weight: 500

	&:hover:not(:disabled)
		background: var(--c-border)

	&:disabled
		opacity: 0.5
		cursor: not-allowed

.fav-btn
	padding: var(--sp-xs) var(--sp-md)
	border: 1px solid var(--c-fav-border)
	background: transparent
	color: var(--c-fav)
	cursor: pointer
	border-radius: var(--radius-sm)
	font-weight: 500
	display: inline-flex
	align-items: center

	&:hover:not(:disabled)
		background: var(--c-fav-hover)

	&:disabled
		cursor: default
		opacity: 0.5

.fav-btn-remove
	padding: var(--sp-xs) var(--sp-md)
	border: 1px solid var(--c-border)
	background: transparent
	color: var(--c-danger)
	cursor: pointer
	border-radius: var(--radius-sm)
	font-weight: 500
	display: inline-flex
	align-items: center

	&:hover:not(:disabled)
		background: var(--c-danger-subtle)

	&:disabled
		cursor: default
		opacity: 0.5

.content
	padding: var(--sp-lg) 0
	line-height: 1.6
	font-size: var(--fs-body)

	:global(p)
		margin: 0 0 1em 0

	:global(br)
		line-height: 2

// On compact viewports, 目次 and unfav live in the app hamburger (fat-finger + rare use).
@media (max-width: 799px)
	.toc-btn,
	.fav-btn-remove
		display: none

</style>
