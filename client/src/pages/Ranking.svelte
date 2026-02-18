<script>
	import config from '$lib/config.js';
	import fetcher from '$lib/fetcher.js';
	import { link } from '$lib/router.svelte.js';
	import NovelDetailModal from '$lib/components/NovelDetailModal.svelte';
	import { decodeHtml } from '$lib/decode.js';

	let { type } = $props();

	const allPeriods = [
		{ key: 'daily', label: '日間' },
		{ key: 'weekly', label: '週間' },
		{ key: 'monthly', label: '月間' },
		{ key: 'quarter', label: '四半期', exclude: ['kakuyomu'] },
		{ key: 'yearly', label: '年間' },
	];
	let periods = $derived(allPeriods.filter((p) => !p.exclude?.includes(type)));
	let activePeriod = $state('daily');
	let ranking = $state(null);
	let activeGenre = $state(null);
	let loading = $state(false);
	let error = $state(null);
	let selectedNovel = $state(null);
	let favIds = $state(new Set());
	let genres = $derived(ranking ? Object.keys(ranking) : []);

	async function loadRanking(t, period) {
		loading = true;
		error = null;
		try {
			const [rankingData, favorites] = await Promise.all([
				fetcher(`${config.path.api}/novel/${t}/ranking?period=${period}`),
				fetcher(`${config.path.api}/favorites`).catch(() => []),
			]);
			ranking = rankingData;
			favIds = new Set(favorites.filter((f) => f.type === t).map((f) => f.id));
			const keys = Object.keys(ranking);
			activeGenre = keys.length > 1 ? (keys.includes(activeGenre) ? activeGenre : keys[0]) : null;
		} catch (e) {
			error = e.message;
			ranking = null;
		} finally {
			loading = false;
		}
	}

	function selectPeriod(period) {
		activePeriod = period;
		loadRanking(type, period);
	}

	function updateFavIds(id) {
		favIds.has(id) ? favIds.delete(id) : favIds.add(id);
		favIds = new Set(favIds);
	}

	async function toggleFavorite(e, novel) {
		e.stopPropagation();
		const isFav = favIds.has(novel.id);
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
			updateFavIds(novel.id);
		} catch (err) {
			alert(err.message);
		}
	}

	$effect(() => {
		activePeriod = 'daily';
		loadRanking(type, 'daily');
	});
</script>

<div class="ranking">
	<div class="toolbar">
		{#if genres.length > 1}
			<div class="genre-tabs">
				{#each genres as genre}
					<button
						class="genre-tab"
						class:active={activeGenre === genre}
						onclick={() => activeGenre = genre}
					>{genre}</button>
				{/each}
			</div>
		{/if}
		<div class="period-tabs">
			{#each periods as p}
				<button
					class="period-tab"
					class:active={activePeriod === p.key}
					onclick={() => selectPeriod(p.key)}
				>{p.label}</button>
			{/each}
		</div>
	</div>

	{#if loading}
		<p class="status">読み込み中...</p>
	{:else if error}
		<p class="status error">{error}</p>
	{:else if ranking}
		{@const visibleGenres = activeGenre ? [[activeGenre, ranking[activeGenre] ?? []]] : Object.entries(ranking)}
		{#each visibleGenres as [genre, novels]}
			<div class="novel-grid">
				{#each novels as novel, i}
					<a href={link(`/novel/${type}/${novel.id}/1`)} class="novel-card">
						<div class="card-body">
							<div class="card-header">
								<span class="card-rank">{i + 1}位</span>
								<span class="card-page" class:tanpen={novel.noveltype === 2}>{novel.noveltype === 2 ? '短編' : `${novel.page}話`}</span>
							</div>
							<div class="card-title">{decodeHtml(novel.title)}</div>
						</div>
						<div class="card-actions">
							<button class="detail-btn" onclick={(e) => { e.preventDefault(); e.stopPropagation(); selectedNovel = novel; }}>📖</button>
							<button class="fav-btn" onclick={(e) => { e.preventDefault(); toggleFavorite(e, novel); }}>{favIds.has(novel.id) ? '★' : '☆'}</button>
						</div>
					</a>
				{/each}
			</div>
		{/each}
	{/if}
</div>

{#if selectedNovel}
	<NovelDetailModal
		{type}
		novel={selectedNovel}
		isFav={favIds.has(selectedNovel.id)}
		onToggleFav={updateFavIds}
		onclose={() => selectedNovel = null}
	/>
{/if}

<style lang="sass">
.ranking
	padding: 12px 15px 0

.toolbar
	display: flex
	justify-content: space-between
	align-items: center
	margin-bottom: 12px
	flex-wrap: wrap
	gap: 8px

.period-tabs
	display: flex
	gap: 4px
	margin-left: auto

.period-tab
	padding: 4px 10px
	border: 1px solid #444
	background: transparent
	color: rgba(255, 255, 255, 0.6)
	cursor: pointer
	border-radius: 3px
	font-size: 0.8rem

	&:hover
		background: rgba(255, 255, 255, 0.08)

	&.active
		background: rgba(255, 255, 255, 0.15)
		color: white
		border-color: rgba(128, 192, 255, 0.5)

.status
	text-align: center
	padding: 20px
	color: rgba(255, 255, 255, 0.6)

	&.error
		color: #ff6b6b

.genre-tabs
	display: flex
	gap: 4px
	flex-wrap: wrap

.genre-tab
	padding: 4px 12px
	border: 1px solid #444
	background: transparent
	color: rgba(255, 255, 255, 0.6)
	cursor: pointer
	border-radius: 3px
	font-size: 0.85rem

	&:hover
		background: rgba(255, 255, 255, 0.08)

	&.active
		background: rgba(255, 255, 255, 0.15)
		color: white
		border-color: rgba(128, 192, 255, 0.5)

.novel-grid
	display: flex
	flex-direction: column
	gap: 8px

.novel-card
	display: flex
	border: 1px solid #444
	border-radius: 6px
	text-decoration: none
	color: inherit

	&:hover
		background-color: rgba(255, 255, 255, 0.08)

.card-body
	flex: 1
	min-width: 0
	display: flex
	flex-direction: column
	gap: 4px
	padding: 8px

.card-header
	display: flex
	align-items: center
	gap: 8px

.card-rank
	font-size: 0.8rem
	font-weight: bold
	color: rgba(255, 255, 255, 0.5)

.card-page
	font-size: 0.8rem
	color: rgba(255, 255, 255, 0.5)

	&.tanpen
		color: rgba(255, 255, 255, 0.4)

.card-title
	line-height: 1.4

.card-actions
	display: flex
	flex-direction: column
	flex-shrink: 0
	width: 40px
	border-left: 1px solid #444

.detail-btn, .fav-btn
	flex: 1
	width: 100%
	border: none
	border-radius: 0
	background: transparent
	cursor: pointer
	display: flex
	align-items: center
	justify-content: center
	font-size: 0.9rem

.detail-btn
	border-bottom: 1px solid #444
	border-radius: 0 6px 0 0

	&:hover
		background: rgba(128, 192, 255, 0.15)

.fav-btn
	border-radius: 0 0 6px 0
	color: rgba(255, 200, 50, 0.8)

	&:hover
		background: rgba(255, 200, 50, 0.1)
		color: rgba(255, 200, 50, 1)

// Desktop
@media (min-width: 800px)
	.toolbar
		position: sticky
		top: 0
		background: #222
		z-index: 50
		padding-top: 12px
		padding-bottom: 12px
		margin-bottom: 0

	.card-title
		font-size: 1rem
</style>
