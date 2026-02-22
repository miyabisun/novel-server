<script>
	import config from '$lib/config.js';
	import fetcher from '$lib/fetcher.js';
	import { navigate, link } from '$lib/router.svelte.js';
	import { decodeHtml } from '$lib/decode.js';

	let { params } = $props();
	let title = $state('');
	let episodes = $state([]);
	let loading = $state(true);
	let error = $state(null);
	let currentRead = $state(0);

	async function loadToc(type, id) {
		loading = true;
		error = null;
		currentRead = 0;
		try {
			const [data, favorites] = await Promise.all([
				fetcher(`${config.path.api}/novel/${type}/${id}/toc`),
				fetcher(`${config.path.api}/favorites`).catch(() => []),
			]);
			title = decodeHtml(data.title || '');
			episodes = data.episodes || [];
			const fav = favorites.find((f) => f.type === type && f.id === id);
			if (fav) currentRead = fav.read || 0;
		} catch (e) {
			error = e.message;
		} finally {
			loading = false;
		}
	}

	function handleKeydown(e) {
		if (e.key === 'Backspace') {
			e.preventDefault();
			navigate('/');
		}
	}

	$effect(() => {
		loadToc(params.type, params.id);
	});

	$effect(() => {
		document.title = title
			? `目次 ${title} | novel-server`
			: 'novel-server';
		return () => { document.title = 'novel-server'; };
	});
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="toc">
	<header class="toc-header">
		<h1 class="toc-title">{title || params.id}</h1>
	</header>

	{#if loading}
		<p class="status">読み込み中...</p>
	{:else if error}
		<p class="status error">{error}</p>
		<p class="status"><button class="retry-btn" onclick={() => loadToc(params.type, params.id)}>再読み込み</button></p>
	{:else if episodes.length === 0}
		<p class="status">エピソードがありません</p>
	{:else}
		<div class="ep-list">
			{#each episodes as ep (ep.num)}
				<a class="ep-card" class:current={ep.num === currentRead} href={link(`/novel/${params.type}/${params.id}/${ep.num}`)}>
					<span class="ep-num">{ep.num}</span>
					<span class="ep-title">{ep.title}</span>
				</a>
			{/each}
		</div>
	{/if}
</div>

<style lang="sass">
.toc
	padding: 0 var(--sp-4) var(--sp-5)
	max-width: 800px
	margin: 0 auto

.toc-header
	padding: var(--sp-5) 0 var(--sp-4)

.toc-title
	margin: 0
	font-size: var(--fs-lg)
	color: white
	line-height: 1.4

.status
	text-align: center
	padding: var(--sp-5)
	color: var(--c-text-sub)

	&.error
		color: #ff6b6b

.retry-btn
	padding: var(--sp-2) var(--sp-4)
	border: 1px solid var(--c-border-strong)
	background: transparent
	color: var(--c-text-sub)
	cursor: pointer
	border-radius: var(--radius-sm)
	font-size: var(--fs-sm)

	&:hover
		background: var(--c-overlay-2)

.ep-list
	display: flex
	flex-direction: column
	gap: var(--sp-2)

.ep-card
	display: flex
	align-items: baseline
	gap: var(--sp-3)
	padding: var(--sp-3)
	border: 1px solid var(--c-border)
	border-radius: var(--radius-md)
	text-decoration: none
	color: inherit

	&:hover
		background: var(--c-surface-hover)

	&.current
		border-left: 3px solid var(--c-fav)

.ep-num
	flex-shrink: 0
	font-size: var(--fs-xs)
	color: var(--c-text-muted)
	min-width: 3ch
	text-align: right

.ep-title
	line-height: 1.4
</style>
