<script>
	import config from '$lib/config.js';
	import fetcher from '$lib/fetcher.js';
	import { link } from '$lib/router.svelte.js';
	import NovelDetailModal from '$lib/components/NovelDetailModal.svelte';
	import ConfirmModal from '$lib/components/ConfirmModal.svelte';
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
	let activeGenre = $state('総合');
	let loading = $state(false);
	let error = $state(null);
	let selectedNovel = $state(null);
	let favIds = $state(new Set());
	let deleteTarget = $state(null);

	// Search state
	let searchQuery = $state('');
	let searchResults = $state(null);
	let searchLoading = $state(false);

	let isSearchMode = $derived(activeGenre === '検索');

	let genreOptions = $derived.by(() => {
		if (!ranking) return [];
		const keys = Object.keys(ranking).filter((k) => k !== '総合');
		return ['総合', ...keys, '検索'];
	});

	let displayNovels = $derived.by(() => {
		if (isSearchMode) return searchResults ?? [];
		if (!ranking) return [];
		return ranking[activeGenre] ?? [];
	});

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
			// Reset genre if current genre doesn't exist in new ranking
			if (activeGenre !== '総合' && activeGenre !== '検索') {
				const keys = Object.keys(ranking);
				if (!keys.includes(activeGenre)) {
					activeGenre = '総合';
				}
			}
		} catch (e) {
			error = e.message;
			ranking = null;
		} finally {
			loading = false;
		}
	}

	function updateUrlParams() {
		const params = new URLSearchParams();
		if (activePeriod !== 'daily') params.set('period', activePeriod);
		if (activeGenre !== '総合') params.set('genre', activeGenre);
		const qs = params.toString();
		const url = window.location.pathname + (qs ? '?' + qs : '');
		history.replaceState({}, '', url);
	}

	function selectPeriod(period) {
		activePeriod = period;
		updateUrlParams();
		loadRanking(type, period);
	}

	function selectGenre(value) {
		activeGenre = value;
		updateUrlParams();
		if (value === '検索') {
			searchResults = null;
		}
	}

	async function executeSearch() {
		const q = searchQuery.trim();
		if (!q) return;
		searchLoading = true;
		error = null;
		try {
			searchResults = await fetcher(`${config.path.api}/novel/${type}/search?q=${encodeURIComponent(q)}`);
		} catch (e) {
			error = e.message;
			searchResults = null;
		} finally {
			searchLoading = false;
		}
	}

	function handleSearchKeydown(e) {
		if (e.key === 'Enter') executeSearch();
	}

	function updateFavIds(id) {
		const next = new Set(favIds);
		if (next.has(id)) next.delete(id)
		else next.add(id)
		favIds = next;
	}

	async function addFavorite(novel) {
		try {
			await fetcher(`${config.path.api}/favorites/${type}/${novel.id}`, {
				method: 'PUT',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ title: novel.title, page: novel.page }),
			});
			updateFavIds(novel.id);
		} catch (err) {
			alert(err.message);
		}
	}

	async function removeFavorite(novel) {
		try {
			await fetcher(`${config.path.api}/favorites/${type}/${novel.id}`, { method: 'DELETE' });
			updateFavIds(novel.id);
		} catch (err) {
			alert(err.message);
		}
	}

	function confirmDelete(novel) {
		deleteTarget = novel;
	}

	function cancelDelete() {
		deleteTarget = null;
	}

	async function executeDelete() {
		if (!deleteTarget) return;
		try {
			await removeFavorite(deleteTarget);
		} finally {
			cancelDelete();
		}
	}

	function readUrlParams() {
		const params = new URLSearchParams(window.location.search);
		return {
			genre: params.get('genre') || '総合',
			period: params.get('period') || 'daily',
		};
	}

	$effect(() => {
		const saved = readUrlParams();
		activePeriod = saved.period;
		activeGenre = saved.genre;
		searchQuery = '';
		searchResults = null;
		loadRanking(type, saved.period);
	});

</script>

<div class="ranking">
	<div class="toolbar">
		<select class="genre-select" value={activeGenre} onchange={(e) => selectGenre(e.target.value)}>
			{#each genreOptions as opt}
				<option value={opt}>{opt}</option>
			{/each}
		</select>
		{#if isSearchMode}
			<div class="search-box">
				<input
					class="search-input"
					type="text"
					placeholder="タイトルを入力..."
					bind:value={searchQuery}
					onkeydown={handleSearchKeydown}
				/>
				<button class="search-btn" onclick={executeSearch} disabled={searchLoading}>🔍</button>
			</div>
		{:else}
			<select class="period-select" value={activePeriod} onchange={(e) => selectPeriod(e.target.value)}>
				{#each periods as p}
					<option value={p.key}>{p.label}</option>
				{/each}
			</select>
		{/if}
	</div>

	{#if loading || searchLoading}
		<p class="status">読み込み中...</p>
	{:else if error}
		<p class="status error">{error}</p>
	{:else if isSearchMode && !searchResults}
		<p class="status">キーワードを入力して検索してください</p>
	{:else if displayNovels.length > 0}
		<div class="novel-grid">
			{#each displayNovels as novel, i}
				<div class="novel-card-wrapper">
					<div
						class="novel-card"
						class:is-fav={favIds.has(novel.id)}
					>
						<div class="card-body">
							<div class="card-header">
								{#if !isSearchMode}
									<span class="card-rank">{i + 1}位</span>
								{/if}
								<span class="card-page" class:tanpen={novel.noveltype === 2}>{novel.noveltype === 2 ? '短編' : `${novel.page}話`}</span>
							</div>
							<div class="card-title"><a href={link(`/novel/${type}/${novel.id}/1`)}>{decodeHtml(novel.title)}</a></div>
						</div>
						<div class="card-actions">
							<button class="detail-btn" onclick={() => selectedNovel = novel}>📖</button>
							{#if favIds.has(novel.id)}
								<button class="unfav-btn" onclick={() => confirmDelete(novel)}>✕</button>
							{:else}
								<button class="fav-btn" onclick={() => addFavorite(novel)}>☆</button>
							{/if}
						</div>
					</div>
				</div>
			{/each}
		</div>
	{:else if isSearchMode && searchResults}
		<p class="status">検索結果が見つかりませんでした</p>
	{/if}
</div>

{#if deleteTarget}
	<ConfirmModal
		message={`「${decodeHtml(deleteTarget.title)}」をお気に入りから削除しますか？`}
		onconfirm={executeDelete}
		oncancel={cancelDelete}
	/>
{/if}

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
	padding: var(--sp-4) var(--sp-4) 0

.toolbar
	display: flex
	justify-content: space-between
	align-items: center
	margin-bottom: var(--sp-4)
	gap: var(--sp-3)
	position: sticky
	top: var(--header-h)
	background: var(--c-bg)
	z-index: 50

.genre-select, .period-select
	padding: var(--sp-2) var(--sp-3)
	border: 1px solid var(--c-border)
	background: var(--c-surface)
	color: var(--c-text)
	border-radius: var(--radius-md)
	font-size: var(--fs-sm)
	cursor: pointer
	appearance: auto
	min-width: 0

.genre-select
	flex: 1
	max-width: 200px

.period-select
	flex-shrink: 0

.search-box
	display: flex
	flex: 1
	gap: var(--sp-1)
	min-width: 0

.search-input
	flex: 1
	min-width: 0
	padding: var(--sp-2) var(--sp-3)
	border: 1px solid var(--c-border)
	background: var(--c-surface)
	color: var(--c-text)
	border-radius: var(--radius-md)
	font-size: var(--fs-sm)

	&::placeholder
		color: var(--c-text-muted)

	&:focus
		outline: none
		border-color: var(--c-accent-active)

.search-btn
	flex-shrink: 0
	padding: var(--sp-2) var(--sp-3)
	border: 1px solid var(--c-border)
	background: var(--c-surface)
	color: var(--c-text)
	border-radius: var(--radius-md)
	cursor: pointer
	font-size: var(--fs-sm)

	&:hover
		background: var(--c-overlay-2)

	&:disabled
		opacity: 0.5
		cursor: not-allowed

.novel-grid
	display: flex
	flex-direction: column
	gap: var(--sp-3)

.novel-card-wrapper
	border: 1px solid var(--c-border)
	border-radius: var(--radius-md)

.novel-card
	display: flex

	&.is-fav
		border-left: 3px solid var(--c-fav-border)

.card-body
	flex: 1
	min-width: 0
	display: flex
	flex-direction: column
	gap: var(--sp-1)
	padding: var(--sp-3)

.card-header
	display: flex
	align-items: center
	gap: var(--sp-3)

.card-rank
	font-size: var(--fs-xs)
	font-weight: bold
	color: var(--c-text-muted)

.card-page
	font-size: var(--fs-xs)
	color: var(--c-text-muted)

	&.tanpen
		color: var(--c-text-faint)

.card-title
	line-height: 1.4

	a
		text-decoration: none
		color: inherit

		&:hover
			text-decoration: underline

.card-actions
	display: flex
	flex-direction: column
	flex-shrink: 0
	width: 40px
	border-left: 1px solid var(--c-border)

.detail-btn, .fav-btn, .unfav-btn
	flex: 1
	width: 100%
	border: none
	border-radius: 0
	background: transparent
	cursor: pointer
	display: flex
	align-items: center
	justify-content: center
	font-size: var(--fs-sm)

.detail-btn
	border-bottom: 1px solid var(--c-border)
	border-radius: 0 var(--radius-md) 0 0

	&:hover
		background: var(--c-accent-subtle)

.fav-btn
	border-radius: 0 0 var(--radius-md) 0
	color: var(--c-fav)

	&:hover
		background: var(--c-fav-hover)
		color: var(--c-fav-bright)

.unfav-btn
	border-radius: 0 0 var(--radius-md) 0
	color: var(--c-danger-dim)
	font-size: var(--fs-sm)

	&:hover
		background: var(--c-danger-hover)

// Desktop
@media (min-width: 800px)
	.toolbar
		padding-top: var(--sp-4)
		padding-bottom: var(--sp-4)
		margin-bottom: 0

	.novel-card-wrapper:hover .novel-card
		background-color: var(--c-overlay-2)

	.card-title
		font-size: var(--fs-md)

// Mobile
@media (max-width: 799px)
	.detail-btn
		border-bottom: none
		border-radius: 0 var(--radius-md) var(--radius-md) 0
</style>
