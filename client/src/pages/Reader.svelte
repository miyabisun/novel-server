<script>
	import config from '$lib/config.js';
	import fetcher from '$lib/fetcher.js';
	import { navigate } from '$lib/router.svelte.js';

	let { params } = $props();
	let html = $state('');
	let loading = $state(false);
	let error = $state(null);
	let title = $state('');
	let currentNum = $derived(Number(params.num));

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
		if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') {
			e.preventDefault();
			goTo(currentNum - 1);
		} else if (e.key === 'ArrowRight' || e.key === 'ArrowDown') {
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
		} catch (e) { console.warn('loadDetail failed:', e) }
	}

	$effect(() => {
		loadDetail(params.type, params.id);
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

<div class="reader">
	<nav class="nav">
		<button onclick={() => navigate('/')}>← ランキングへ</button>
		<span class="page-info">{title || params.id} / {currentNum}話</span>
		<div class="page-controls">
			<button onclick={() => goTo(currentNum - 1)} disabled={currentNum <= 1}>← 前</button>
			<button onclick={() => goTo(currentNum + 1)}>次 →</button>
		</div>
	</nav>

	{#if loading}
		<p class="status">読み込み中...</p>
	{:else if error}
		<p class="status error">{error}</p>
	{:else}
		<article class="content">
			{@html html}
		</article>
	{/if}

	<nav class="nav bottom">
		<button onclick={() => goTo(currentNum - 1)} disabled={currentNum <= 1}>← 前のページ</button>
		<button onclick={() => goTo(currentNum + 1)}>次のページ →</button>
	</nav>
</div>

<style lang="sass">
.reader
	padding: 0 15px
	max-width: 800px
	margin: 0 auto

.nav
	display: flex
	align-items: center
	justify-content: space-between
	padding: 8px 0
	border-bottom: 1px solid #555
	flex-wrap: wrap
	gap: 8px

	&.bottom
		border-bottom: none
		border-top: 1px solid #555
		justify-content: center
		gap: 16px
		margin-top: 24px
		padding-top: 16px

.page-info
	color: rgba(255, 255, 255, 0.6)
	font-size: 0.9rem

.page-controls
	display: flex
	gap: 4px

button
	padding: 4px 12px
	border: 1px solid #555
	background: transparent
	color: rgba(255, 255, 255, 0.7)
	cursor: pointer
	border-radius: 4px

	&:hover:not(:disabled)
		background: rgba(255, 255, 255, 0.1)

	&:disabled
		opacity: 0.3
		cursor: not-allowed

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
