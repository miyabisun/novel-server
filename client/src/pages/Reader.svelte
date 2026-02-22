<script>
	import config from '$lib/config.js';
	import fetcher from '$lib/fetcher.js';
	import { navigate } from '$lib/router.svelte.js';
	import { decodeHtml } from '$lib/decode.js';

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

	async function loadPage(type, id, num) {
		loading = true;
		error = null;
		try {
			const data = await fetcher(`${config.path.api}/novel/${type}/${id}/pages/${num}`);
			html = data.html || '';
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
		if (showUnfavConfirm) {
			if (e.key === 'Escape') showUnfavConfirm = false;
			return;
		}
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
		fetch(`${config.path.api}/favorites/${type}/${id}/progress`, {
			method: 'PATCH',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ read: Number(num) }),
		}).catch(() => {});
	}

	async function loadDetail(type, id) {
		try {
			const data = await fetcher(`${config.path.api}/novel/${type}/${id}/detail`);
			title = decodeHtml(data.title || '');
			totalPages = data.page || 0;
		} catch (e) { console.warn('loadDetail failed:', e) }
	}

	async function loadFavStatus(type, id) {
		try {
			const favorites = await fetcher(`${config.path.api}/favorites`);
			isFav = favorites.some((f) => f.type === type && f.id === id);
		} catch { isFav = false; }
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
			await fetcher(`${config.path.api}/favorites/${params.type}/${params.id}`, {
				method: 'PUT',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ title, page: totalPages }),
			});
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
			await fetcher(`${config.path.api}/favorites/${params.type}/${params.id}`, { method: 'DELETE' });
			isFav = false;
		} catch (err) {
			alert(err.message);
		} finally {
			favSaving = false;
		}
	}

	function handleBackdrop(e) {
		if (e.target === e.currentTarget) showUnfavConfirm = false;
	}

	$effect(() => {
		document.title = title
			? `${currentNum}話 ${title} | novel-server`
			: 'novel-server';
		return () => { document.title = 'novel-server'; };
	});

	$effect(() => {
		loadDetail(params.type, params.id);
		loadFavStatus(params.type, params.id);
	});

	$effect(() => {
		loadPage(params.type, params.id, params.num);
	});

	$effect(() => {
		window.addEventListener('keydown', handleKeydown);
		return () => window.removeEventListener('keydown', handleKeydown);
	});

	$effect(() => {
		const node = document.querySelector('.reader');
		if (!node) return;

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

		return () => {
			node.removeEventListener('touchstart', onStart);
			node.removeEventListener('touchmove', onMove);
			node.removeEventListener('touchend', onEnd);
		};
	});
</script>

<nav class="reader-bar top">
	<div class="bar-title">{title || params.id}</div>
	<div class="bar-right">
		<span class="bar-page">{currentNum}{#if totalPages}/{totalPages}{/if}</span>
		<button class="nav-btn" onclick={() => goTo(currentNum - 1)} disabled={!canGoPrev}>前</button>
		<button class="toc-btn" onclick={() => navigate(`/novel/${params.type}/${params.id}/toc`)} disabled={totalPages <= 1}>目次</button>
		<button class="nav-btn" onclick={() => goTo(currentNum + 1)} disabled={!canGoNext}>次</button>
		{#if isFav}
			<button class="fav-btn-remove" onclick={handleFavClick} disabled={favSaving || !title}>✕</button>
		{:else}
			<button class="fav-btn" onclick={handleFavClick} disabled={favSaving || !title}>☆</button>
		{/if}
	</div>
</nav>

{#if swipeDir === 'prev'}
	<div class="swipe-hint left" class:ready={swipeReady} class:disabled={!canGoPrev}>{#if canGoPrev}‹ 前へ{:else}<del>‹ 前へ</del>{/if}</div>
{/if}
{#if swipeDir === 'next'}
	<div class="swipe-hint right" class:ready={swipeReady} class:disabled={!canGoNext}>{#if canGoNext}次へ ›{:else}<del>次へ ›</del>{/if}</div>
{/if}

<div class="reader">
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
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="backdrop" onclick={handleBackdrop}>
		<div class="modal">
			<p class="modal-message">「{title}」をお気に入りから削除しますか？</p>
			<div class="modal-actions">
				<button class="btn btn-cancel" onclick={() => showUnfavConfirm = false}>キャンセル</button>
				<button class="btn btn-delete" onclick={executeUnfav}>削除</button>
			</div>
		</div>
	</div>
{/if}

<style lang="sass">
.reader
	padding: 0 var(--sp-4)
	max-width: 800px
	margin: 0 auto

.reader-bar
	display: flex
	align-items: center
	justify-content: space-between
	padding: var(--sp-3) 2.5%
	background: var(--c-surface)
	border-bottom: 1px solid var(--c-border)
	gap: var(--sp-3)
	z-index: 50

	&.top
		position: sticky
		top: 0

.bar-title
	color: var(--c-text-sub)
	font-size: var(--fs-sm)
	white-space: nowrap
	overflow: hidden
	text-overflow: ellipsis
	min-width: 0
	flex: 1

.bar-right
	display: flex
	align-items: center
	gap: var(--sp-1)
	flex-shrink: 0

.bar-page
	color: var(--c-text-muted)
	font-size: var(--fs-xs)
	margin-right: var(--sp-1)
	white-space: nowrap

.nav-btn
	padding: var(--sp-1) var(--sp-4)
	border: 1px solid var(--c-border-strong)
	background: transparent
	color: var(--c-text-sub)
	cursor: pointer
	border-radius: var(--radius-sm)
	font-size: var(--fs-sm)

	&:hover:not(:disabled)
		background: var(--c-overlay-2)

	&:disabled
		opacity: 0.3
		cursor: not-allowed

	@media (max-width: 768px)
		.bar-right > &
			display: none

.toc-btn
	padding: var(--sp-1) var(--sp-3)
	border: 1px solid var(--c-border-strong)
	background: transparent
	color: var(--c-text-sub)
	cursor: pointer
	border-radius: var(--radius-sm)
	font-size: var(--fs-sm)

	&:hover:not(:disabled)
		background: var(--c-overlay-2)

	&:disabled
		opacity: 0.3
		cursor: not-allowed

.fav-btn
	padding: var(--sp-1) var(--sp-3)
	border: 1px solid var(--c-fav-border)
	background: transparent
	color: var(--c-fav)
	cursor: pointer
	border-radius: var(--radius-sm)
	font-size: var(--fs-md)

	&:hover:not(:disabled)
		background: var(--c-fav-hover)

	&:disabled
		cursor: default
		opacity: 0.7

.fav-btn-remove
	padding: var(--sp-1) var(--sp-3)
	border: 1px solid var(--c-danger-border)
	background: transparent
	color: var(--c-danger-dim)
	cursor: pointer
	border-radius: var(--radius-sm)
	font-size: var(--fs-md)

	&:hover:not(:disabled)
		background: var(--c-danger-hover)

	&:disabled
		cursor: default
		opacity: 0.7

.status
	text-align: center
	padding: var(--sp-6)
	color: var(--c-text-sub)

	&.error
		color: #ff6b6b

.content
	padding: var(--sp-4) 0
	line-height: 2
	font-size: 1.05rem

	:global(p)
		margin: 0 0 1em 0

	:global(br)
		line-height: 2

.swipe-hint
	position: fixed
	top: 50%
	transform: translateY(-50%)
	padding: var(--sp-4)
	background: rgba(0, 0, 0, 0.7)
	color: var(--c-text-sub)
	font-size: var(--fs-sm)
	border-radius: var(--radius-lg)
	z-index: 100
	pointer-events: none
	transition: background 0.15s, color 0.15s

	&.left
		left: var(--sp-4)

	&.right
		right: var(--sp-4)

	&.ready
		background: rgba(0, 0, 0, 0.85)
		color: rgba(255, 255, 255, 0.95)

	&.disabled
		color: rgba(255, 255, 255, 0.55)

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
	max-width: 360px
	width: 100%

.modal-message
	margin: 0 0 var(--sp-5)
	font-size: var(--fs-md)
	color: var(--c-text)
	line-height: 1.6
	overflow-wrap: break-word

.modal-actions
	display: flex
	gap: var(--sp-3)
	justify-content: flex-end

.btn
	padding: var(--sp-3) var(--sp-4)
	border: 1px solid var(--c-border-strong)
	border-radius: var(--radius-sm)
	cursor: pointer
	font-size: var(--fs-sm)

.btn-cancel
	background: transparent
	color: var(--c-text-sub)

	&:hover
		background: var(--c-overlay-2)

.btn-delete
	background: var(--c-danger-bg)
	color: var(--c-danger)
	border-color: var(--c-danger-border)

	&:hover
		background: var(--c-danger-bg-hover)

</style>
