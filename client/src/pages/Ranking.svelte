<script>
	import config from '$lib/config.js';
	import fetcher from '$lib/fetcher.js';
	import { navigate } from '$lib/router.svelte.js';

	const types = ['narou', 'kakuyomu', 'nocturne'];
	let activeType = $state('narou');
	let ranking = $state(null);
	let loading = $state(false);
	let error = $state(null);

	async function loadRanking(type) {
		loading = true;
		error = null;
		try {
			ranking = await fetcher(`${config.path.api}/novel/${type}/ranking`);
		} catch (e) {
			error = e.message;
			ranking = null;
		} finally {
			loading = false;
		}
	}

	function selectType(type) {
		activeType = type;
		loadRanking(type);
	}

	function goToReader(type, id, num = 1) {
		navigate(`/novel/${type}/${id}/${num}`);
	}

	async function addFavorite(e, novel) {
		e.stopPropagation();
		const btn = e.currentTarget;
		try {
			await fetcher(`${config.path.api}/favorites/${activeType}/${novel.id}`, {
				method: 'PUT',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ title: novel.title, page: novel.page }),
			});
			btn.textContent = '★';
		} catch (err) {
			alert(err.message);
		}
	}

	// Initial load only
	loadRanking('narou');
</script>

<div class="ranking">
	<div class="tabs">
		{#each types as type}
			<button
				class="tab"
				class:active={activeType === type}
				onclick={() => selectType(type)}
			>{type}</button>
		{/each}
	</div>

	{#if loading}
		<p class="status">読み込み中...</p>
	{:else if error}
		<p class="status error">{error}</p>
	{:else if ranking}
		{#each Object.entries(ranking) as [genre, novels]}
			<section class="genre">
				<h2>{genre}</h2>
				<table>
					<thead>
						<tr>
							<th class="col-fav"></th>
							<th class="col-rank">#</th>
							<th class="col-title">タイトル</th>
							<th class="col-page">話数</th>
						</tr>
					</thead>
					<tbody>
						{#each novels as novel, i}
							<tr onclick={() => goToReader(activeType, novel.id)} class="clickable">
								<td class="col-fav">
									<button class="fav-btn" onclick={(e) => addFavorite(e, novel)}>☆</button>
								</td>
								<td class="col-rank">{i + 1}</td>
								<td class="col-title">{novel.title}</td>
								<td class="col-page">{novel.page}</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</section>
		{/each}
	{/if}
</div>

<style lang="sass">
.ranking
	padding: 0 15px

.tabs
	display: flex
	gap: 4px
	margin-bottom: 16px

.tab
	padding: 6px 16px
	border: 1px solid #555
	background: transparent
	color: rgba(255, 255, 255, 0.7)
	cursor: pointer
	border-radius: 4px

	&:hover
		background: rgba(255, 255, 255, 0.1)

	&.active
		background: rgba(255, 255, 255, 0.2)
		color: white
		border-color: rgba(128, 192, 255, 0.6)

.status
	text-align: center
	padding: 20px
	color: rgba(255, 255, 255, 0.6)

	&.error
		color: #ff6b6b

.genre
	margin-bottom: 24px

	h2
		font-size: 1.1rem
		margin: 0 0 8px 0
		border-bottom: 1px solid #555
		padding-bottom: 4px

.col-fav
	width: 30px
	text-align: center

.fav-btn
	padding: 0
	border: none
	background: transparent
	color: rgba(255, 200, 50, 0.8)
	cursor: pointer
	font-size: 1.1rem

	&:hover
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
</style>
