<script>
	import config from '$lib/config.js';
	import fetcher from '$lib/fetcher.js';
	import { link } from '$lib/router.svelte.js';
	import NovelDetailModal from '$lib/components/NovelDetailModal.svelte';
	import ConfirmModal from '$lib/components/ConfirmModal.svelte';
	import Icon from '$lib/components/Icon.svelte';
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
		history.pushState({}, '', url);
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

	$effect(() => {
		function onPopState() {
			const saved = readUrlParams();
			if (saved.genre !== activeGenre) activeGenre = saved.genre;
			if (saved.period !== activePeriod) {
				activePeriod = saved.period;
				loadRanking(type, saved.period);
			}
		}
		window.addEventListener('popstate', onPopState);
		return () => window.removeEventListener('popstate', onPopState);
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
				<button class="search-btn" onclick={executeSearch} disabled={searchLoading} aria-label="検索"><Icon name="search" /></button>
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
								<span class="card-page">{novel.noveltype === 2 ? '短編' : `${novel.page}話`}</span>
							</div>
							<div class="card-title"><a href={link(`/novel/${type}/${novel.id}/1`)}>{decodeHtml(novel.title)}</a></div>
						</div>
						<div class="card-actions">
							<button class="detail-btn" onclick={() => selectedNovel = novel} aria-label="詳細"><Icon name="book" /></button>
							{#if favIds.has(novel.id)}
								<button class="unfav-btn" onclick={() => confirmDelete(novel)} aria-label="お気に入りから削除"><Icon name="x" /></button>
							{:else}
								<button class="fav-btn" onclick={() => addFavorite(novel)} aria-label="お気に入りに追加"><Icon name="star-outline" /></button>
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
	padding: var(--sp-lg) var(--sp-lg) 0

.toolbar
	display: flex
	justify-content: space-between
	align-items: center
	margin-bottom: var(--sp-lg)
	gap: var(--sp-sm)
	position: sticky
	top: var(--header-h)
	background: var(--c-bg)
	z-index: 50

.genre-select, .period-select
	padding: var(--sp-sm) var(--sp-md)
	border: 1px solid var(--c-border)
	background: var(--c-surface)
	color: var(--c-text)
	border-radius: var(--radius-sm)
	font-size: var(--fs-label)
	font-weight: 500
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
	gap: var(--sp-sm)
	min-width: 0

.search-input
	flex: 1
	min-width: 0
	padding: var(--sp-sm) var(--sp-md)
	border: 1px solid var(--c-border)
	background: var(--c-surface)
	color: var(--c-text)
	border-radius: var(--radius-sm)
	font-size: var(--fs-body)

	&::placeholder
		color: var(--c-text-muted)

	&:focus
		border-color: var(--c-accent)

.search-btn
	flex-shrink: 0
	padding: var(--sp-sm) var(--sp-md)
	border: 1px solid var(--c-border)
	background: var(--c-surface)
	color: var(--c-text)
	border-radius: var(--radius-sm)
	font-size: var(--fs-label)
	font-weight: 500
	cursor: pointer
	display: inline-flex
	align-items: center
	justify-content: center

	&:hover
		background: var(--c-border)

	&:disabled
		opacity: 0.5
		cursor: not-allowed

.novel-grid
	display: flex
	flex-direction: column
	gap: var(--sp-sm)

.novel-card-wrapper
	border: 1px solid var(--c-border)
	border-radius: var(--radius-md)
	background: var(--c-surface)

.novel-card
	display: flex

	&.is-fav
		border-left: 3px solid var(--c-fav-border)

.card-body
	flex: 1
	min-width: 0
	display: flex
	flex-direction: column
	gap: var(--sp-xs)
	padding: 10px

.card-header
	display: flex
	align-items: center
	gap: var(--sp-md)

.card-rank
	font-size: var(--fs-caption)
	font-weight: bold
	color: var(--c-text-muted)

.card-page
	font-size: var(--fs-caption)
	color: var(--c-text-muted)

.card-title
	font-size: var(--fs-label)
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

.detail-btn
	border-bottom: 1px solid var(--c-border)
	border-radius: 0 var(--radius-md) 0 0
	color: var(--c-text-muted)

	&:hover
		background: var(--c-accent-subtle)
		color: var(--c-accent)

.fav-btn
	border-radius: 0 0 var(--radius-md) 0
	color: var(--c-fav)

	&:hover
		background: var(--c-fav-hover)
		color: var(--c-fav-bright)

.unfav-btn
	border-radius: 0 0 var(--radius-md) 0
	color: var(--c-danger)

	&:hover
		background: var(--c-danger-subtle)

// Desktop
@media (min-width: 800px)
	.toolbar
		padding-top: var(--sp-lg)
		padding-bottom: var(--sp-lg)
		margin-bottom: 0

	.novel-card-wrapper:hover .novel-card
		background-color: var(--c-surface-hover)

	.card-title
		font-size: var(--fs-title)

// Mobile
@media (max-width: 799px)
	.fav-btn, .unfav-btn
		display: none

	.detail-btn
		border-bottom: none
		border-radius: 0 var(--radius-md) var(--radius-md) 0
</style>
