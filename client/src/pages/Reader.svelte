<script>
	import config from '$lib/config.js';
	import fetcher from '$lib/fetcher.js';
	import { navigate } from '$lib/router.svelte.js';

	let { params } = $props();
	let html = $state('');
	let loading = $state(false);
	let error = $state(null);
	let title = $state('');
	let totalPages = $state(0);
	let isFav = $state(false);
	let favSaving = $state(false);
	let currentNum = $derived(Number(params.num));
	let headerHeight = $state(0);

	$effect(() => {
		const header = document.querySelector('header');
		if (!header) return;
		headerHeight = header.offsetHeight;
		const ro = new ResizeObserver(() => { headerHeight = header.offsetHeight; });
		ro.observe(header);
		return () => ro.disconnect();
	});

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
		navigate(`/novel/${params.type}/${params.id}/${num}`);
	}

	function handleKeydown(e) {
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
			title = data.title || '';
			totalPages = data.page || 0;
		} catch (e) { console.warn('loadDetail failed:', e) }
	}

	async function loadFavStatus(type, id) {
		try {
			const favorites = await fetcher(`${config.path.api}/favorites`);
			isFav = favorites.some((f) => f.type === type && f.id === id);
		} catch { isFav = false; }
	}

	async function toggleFavorite() {
		if (favSaving) return;
		favSaving = true;
		try {
			if (isFav) {
				await fetcher(`${config.path.api}/favorites/${params.type}/${params.id}`, { method: 'DELETE' });
			} else {
				await fetcher(`${config.path.api}/favorites/${params.type}/${params.id}`, {
					method: 'PUT',
					headers: { 'Content-Type': 'application/json' },
					body: JSON.stringify({ title, page: totalPages }),
				});
			}
			isFav = !isFav;
		} catch (err) {
			alert(err.message);
		} finally {
			favSaving = false;
		}
	}

	$effect(() => {
		loadDetail(params.type, params.id);
		loadFavStatus(params.type, params.id);
	});

	$effect(() => {
		loadPage(params.type, params.id, params.num);
		window.scrollTo(0, 0);
	});

	$effect(() => {
		window.addEventListener('keydown', handleKeydown);
		return () => window.removeEventListener('keydown', handleKeydown);
	});
</script>

<nav class="reader-bar top" style="top: {headerHeight}px">
	<div class="bar-title">{title || params.id}</div>
	<div class="bar-right">
		<span class="bar-page">{currentNum}{#if totalPages}/{totalPages}{/if}</span>
		<button class="nav-btn" onclick={() => goTo(currentNum - 1)} disabled={currentNum <= 1}>前</button>
		<button class="nav-btn" onclick={() => goTo(currentNum + 1)}>次</button>
		<button class="fav-btn" onclick={toggleFavorite} disabled={favSaving || !title}>
			{isFav ? '★' : '☆'}
		</button>
	</div>
</nav>

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

<style lang="sass">
.reader
	padding: 0 15px
	max-width: 800px
	margin: 0 auto

.reader-bar
	display: flex
	align-items: center
	justify-content: space-between
	padding: 8px 2.5%
	background: #2a2a2a
	border-bottom: 1px solid #444
	gap: 8px
	z-index: 50

	&.top
		position: sticky
		margin-top: -16px

.bar-title
	color: rgba(255, 255, 255, 0.75)
	font-size: 0.85rem
	white-space: nowrap
	overflow: hidden
	text-overflow: ellipsis
	min-width: 0
	flex: 1

.bar-right
	display: flex
	align-items: center
	gap: 4px
	flex-shrink: 0

.bar-page
	color: rgba(255, 255, 255, 0.5)
	font-size: 0.8rem
	margin-right: 4px
	white-space: nowrap

.nav-btn
	padding: 4px 14px
	border: 1px solid #555
	background: transparent
	color: rgba(255, 255, 255, 0.7)
	cursor: pointer
	border-radius: 4px
	font-size: 0.85rem

	&:hover:not(:disabled)
		background: rgba(255, 255, 255, 0.1)

	&:disabled
		opacity: 0.3
		cursor: not-allowed

.fav-btn
	padding: 4px 10px
	border: 1px solid rgba(255, 200, 50, 0.3)
	background: transparent
	color: rgba(255, 200, 50, 0.8)
	cursor: pointer
	border-radius: 4px
	font-size: 1rem

	&:hover:not(:disabled)
		background: rgba(255, 200, 50, 0.1)

	&:disabled
		cursor: default
		opacity: 0.7

.status
	text-align: center
	padding: 40px
	color: rgba(255, 255, 255, 0.6)

	&.error
		color: #ff6b6b

.content
	padding: 16px 0
	line-height: 2
	font-size: 1.05rem

	:global(p)
		margin: 0 0 1em 0

	:global(br)
		line-height: 2

</style>
