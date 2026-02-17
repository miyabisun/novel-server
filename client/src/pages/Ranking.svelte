<script>
	import config from '$lib/config.js';
	import fetcher from '$lib/fetcher.js';
	import { navigate } from '$lib/router.svelte.js';
	import NovelDetailModal from '$lib/components/NovelDetailModal.svelte';

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
			activeGenre = keys.length > 1 ? keys[0] : null;
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

	function goToReader(id, num = 1) {
		navigate(`/novel/${type}/${id}/${num}`);
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
			<table>
				<thead>
					<tr>
						<th class="col-rank">#</th>
						<th class="col-title">タイトル</th>
						<th class="col-page">話数</th>
						<th class="col-actions"></th>
					</tr>
				</thead>
				<tbody>
					{#each novels as novel, i}
						<tr onclick={() => selectedNovel = novel} class="clickable">
							<td class="col-rank">{i + 1}</td>
							<td class="col-title">{novel.title}</td>
							<td class="col-page" class:tanpen={novel.noveltype === 2}>{novel.noveltype === 2 ? '短編' : novel.page}</td>
							<td class="col-actions">
								<button class="read-btn" onclick={(e) => { e.stopPropagation(); goToReader(novel.id); }}>📘</button>
								<button class="fav-btn" onclick={(e) => toggleFavorite(e, novel)}>{favIds.has(novel.id) ? '★' : '☆'}</button>
							</td>
						</tr>
					{/each}
				</tbody>
			</table>
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
	padding: 0 15px

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

.col-actions
	width: 70px
	text-align: right
	white-space: nowrap

.read-btn
	padding: 2px 8px
	border: 1px solid rgba(128, 192, 255, 0.4)
	background: transparent
	cursor: pointer
	border-radius: 3px
	font-size: 0.8rem

	&:hover
		background: rgba(128, 192, 255, 0.15)

.fav-btn
	padding: 2px 6px
	border: 1px solid rgba(255, 200, 50, 0.3)
	background: transparent
	color: rgba(255, 200, 50, 0.8)
	cursor: pointer
	border-radius: 3px
	font-size: 0.95rem

	&:hover
		background: rgba(255, 200, 50, 0.1)
		color: rgba(255, 200, 50, 1)

.col-rank
	width: 40px
	text-align: center

.col-title
	text-align: left

.col-page
	width: 60px
	text-align: right

.clickable
	cursor: pointer

// Desktop: sticky toolbar + thead
@media (min-width: 800px)
	.toolbar
		position: sticky
		top: 42px
		background: #222
		z-index: 50
		padding-bottom: 12px
		margin-bottom: 0

	table :global(thead th)
		position: sticky
		top: 76px
		background: #222
		z-index: 40
		box-shadow: 0 -8px 0 #222

// Mobile: card layout
@media (max-width: 799px)
	table
		display: block

	table :global(thead)
		display: none

	table :global(tbody)
		display: flex
		flex-direction: column
		gap: 8px

	table :global(tr)
		display: flex
		flex-wrap: wrap
		align-items: center
		gap: 4px 8px
		padding: 8px 40px 8px 8px
		border: 1px solid #444
		border-radius: 6px
		position: relative

	table :global(td)
		padding: 0

	table :global(.col-actions)
		position: absolute
		right: 0
		top: 0
		bottom: 0
		width: 40px
		display: flex
		flex-direction: column
		gap: 0
		border-left: 1px solid #444

	table :global(.col-actions .read-btn),
	table :global(.col-actions .fav-btn)
		flex: 1
		width: 100%
		border: none
		border-radius: 0
		display: flex
		align-items: center
		justify-content: center

	table :global(.col-actions .read-btn)
		border-bottom: 1px solid #444
		border-radius: 0 6px 0 0

	table :global(.col-actions .fav-btn)
		border-radius: 0 0 6px 0

	table :global(.col-rank)
		width: auto
		font-weight: bold
		&::after
			content: "位"

	table :global(.col-title)
		width: 100%
		order: 2

	table :global(.col-page)
		width: auto
		margin-left: auto
		padding-right: 10px
		font-size: 0.8rem
		color: rgba(255, 255, 255, 0.5)
		&::before
			content: "全"
		&::after
			content: "話"

	table :global(.col-page.tanpen)
		&::before, &::after
			content: none
</style>
