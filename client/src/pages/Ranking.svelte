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
		const keys = Object.keys(ranking);
		if (keys.length > 1) {
			return ['総合', ...keys, '検索'];
		}
		return ['総合', '検索'];
	});

	let displayNovels = $derived.by(() => {
		if (isSearchMode) return searchResults ?? [];
		if (!ranking) return [];
		if (activeGenre === '総合') {
			return Object.values(ranking).flat();
		}
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

	function selectPeriod(period) {
		activePeriod = period;
		loadRanking(type, period);
	}

	function selectGenre(value) {
		activeGenre = value;
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
		favIds.has(id) ? favIds.delete(id) : favIds.add(id);
		favIds = new Set(favIds);
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

	function handleKeydown(e) {
		if (deleteTarget && e.key === 'Escape') cancelDelete();
	}

	function handleBackdrop(e) {
		if (e.target === e.currentTarget) cancelDelete();
	}

	// Bidirectional swipe action for touch devices
	function swipeable(node, opts) {
		let startX, startY, offsetX, locked, horizontal;
		let { isFav, novel } = opts;
		let bgAdd, bgDelete;

		function preventClick(e) {
			e.stopPropagation();
			e.preventDefault();
		}

		function onStart(e) {
			const touch = e.touches[0];
			startX = touch.clientX;
			startY = touch.clientY;
			offsetX = 0;
			locked = false;
			horizontal = false;
			bgAdd = node.parentElement.querySelector('.swipe-bg-add');
			bgDelete = node.parentElement.querySelector('.swipe-bg-delete');
			node.style.transition = 'none';
			if (bgAdd) bgAdd.style.transition = 'none';
			if (bgDelete) bgDelete.style.transition = 'none';
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
			// Left swipe (delete): only if fav; Right swipe (add): only if not fav
			if (dx < 0 && isFav) {
				offsetX = Math.max(-80, dx);
			} else if (dx > 0 && !isFav) {
				offsetX = Math.min(80, dx);
			} else {
				offsetX = 0;
			}
			node.style.transform = `translateX(${offsetX}px)`;
			if (bgDelete) bgDelete.style.opacity = Math.min(1, Math.max(0, -offsetX) / 40);
			if (bgAdd) bgAdd.style.opacity = Math.min(1, Math.max(0, offsetX) / 40);
		}

		function onEnd() {
			if (!locked) return;
			if (horizontal) {
				node.addEventListener('click', preventClick, { once: true, capture: true });
				if (offsetX < -40 && isFav) confirmDelete(novel);
				if (offsetX > 40 && !isFav) addFavorite(novel);
			}
			node.style.transition = 'transform 0.2s ease';
			node.style.transform = 'translateX(0)';
			if (bgDelete) {
				bgDelete.style.transition = 'opacity 0.2s ease';
				bgDelete.style.opacity = 0;
			}
			if (bgAdd) {
				bgAdd.style.transition = 'opacity 0.2s ease';
				bgAdd.style.opacity = 0;
			}
			offsetX = 0;
		}

		node.addEventListener('touchstart', onStart, { passive: true });
		node.addEventListener('touchmove', onMove, { passive: false });
		node.addEventListener('touchend', onEnd, { passive: true });
		node.addEventListener('touchcancel', onEnd, { passive: true });

		return {
			update(newOpts) {
				isFav = newOpts.isFav;
				novel = newOpts.novel;
			},
			destroy() {
				node.removeEventListener('touchstart', onStart);
				node.removeEventListener('touchmove', onMove);
				node.removeEventListener('touchend', onEnd);
				node.removeEventListener('touchcancel', onEnd);
			},
		};
	}

	$effect(() => {
		activePeriod = 'daily';
		activeGenre = '総合';
		searchQuery = '';
		searchResults = null;
		loadRanking(type, 'daily');
	});
</script>

<svelte:window onkeydown={handleKeydown} />

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
					<div class="swipe-bg-add">追加</div>
					<div class="swipe-bg-delete">削除</div>
					<div
						class="novel-card"
						class:is-fav={favIds.has(novel.id)}
						use:swipeable={{ isFav: favIds.has(novel.id), novel }}
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
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="backdrop" onclick={handleBackdrop}>
		<div class="modal">
			<p class="modal-message">「{decodeHtml(deleteTarget.title)}」をお気に入りから削除しますか？</p>
			<div class="modal-actions">
				<button class="btn btn-cancel" onclick={cancelDelete}>キャンセル</button>
				<button class="btn btn-delete" onclick={executeDelete}>削除</button>
			</div>
		</div>
	</div>
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

.status
	text-align: center
	padding: var(--sp-5)
	color: var(--c-text-sub)

	&.error
		color: #ff6b6b

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

.swipe-bg-add, .swipe-bg-delete
	display: none

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

// Delete confirmation modal
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

// Desktop
@media (min-width: 800px)
	.toolbar
		position: sticky
		top: 0
		background: var(--c-bg)
		z-index: 50
		padding-top: var(--sp-4)
		padding-bottom: var(--sp-4)
		margin-bottom: 0

	.novel-card-wrapper:hover .novel-card
		background-color: var(--c-overlay-2)

	.card-title
		font-size: var(--fs-md)

// Mobile: swipe actions
@media (max-width: 799px)
	.novel-card-wrapper
		position: relative
		overflow: hidden

	.novel-card
		background: var(--c-bg)
		position: relative
		z-index: 1

	.fav-btn, .unfav-btn
		display: none

	.detail-btn
		border-bottom: none
		border-radius: 0 var(--radius-md) var(--radius-md) 0

	.swipe-bg-add, .swipe-bg-delete
		display: flex
		align-items: center
		position: absolute
		top: 0
		bottom: 0
		width: 80px
		font-weight: bold
		font-size: var(--fs-sm)
		opacity: 0

	.swipe-bg-add
		left: 0
		justify-content: flex-start
		padding-left: var(--sp-5)
		color: var(--c-fav)

	.swipe-bg-delete
		right: 0
		justify-content: flex-end
		padding-right: var(--sp-5)
		color: var(--c-danger)
</style>
