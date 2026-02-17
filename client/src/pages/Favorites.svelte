<script>
	import config from '$lib/config.js';
	import fetcher from '$lib/fetcher.js';
	import { navigate } from '$lib/router.svelte.js';

	let favorites = $state([]);
	let loading = $state(false);
	let error = $state(null);

	async function loadFavorites() {
		loading = true;
		error = null;
		try {
			favorites = await fetcher(`${config.path.api}/favorites`);
		} catch (e) {
			error = e.message;
		} finally {
			loading = false;
		}
	}

	async function removeFavorite(type, id) {
		try {
			await fetcher(`${config.path.api}/favorites/${type}/${id}`, { method: 'DELETE' });
			favorites = favorites.filter((f) => !(f.type === type && f.id === id));
		} catch (e) {
			alert(e.message);
		}
	}

	function goToReader(fav) {
		const num = fav.read || 1;
		navigate(`/novel/${fav.type}/${fav.id}/${num}`);
	}

	loadFavorites();
</script>

<div class="favorites">
	{#if loading}
		<p class="status">読み込み中...</p>
	{:else if error}
		<p class="status error">{error}</p>
	{:else if favorites.length === 0}
		<p class="status">お気に入りはまだありません</p>
	{:else}
		<table>
			<thead>
				<tr>
					<th class="col-type">サイト</th>
					<th class="col-title">タイトル</th>
					<th class="col-progress">進捗</th>
					<th class="col-action"></th>
				</tr>
			</thead>
			<tbody>
				{#each favorites as fav}
					<tr class="clickable" onclick={() => goToReader(fav)}>
						<td class="col-type">{fav.type}</td>
						<td class="col-title">{fav.title}</td>
						<td class="col-progress">{fav.read} / {fav.page}</td>
						<td class="col-action">
							<button class="delete-btn" onclick={(e) => { e.stopPropagation(); removeFavorite(fav.type, fav.id); }}>
								✕
							</button>
						</td>
					</tr>
				{/each}
			</tbody>
		</table>
	{/if}
</div>

<style lang="sass">
.favorites
	padding: 0 15px

.col-type
	width: 80px

.col-title
	text-align: left

.col-progress
	width: 100px
	text-align: center

.col-action
	width: 50px
	text-align: center

.clickable
	cursor: pointer

.delete-btn
	padding: 2px 8px
	border: 1px solid #555
	background: transparent
	color: rgba(255, 100, 100, 0.8)
	cursor: pointer
	border-radius: 4px
	font-size: 0.8rem

	&:hover
		background: rgba(255, 100, 100, 0.15)

.status
	text-align: center
	padding: 20px
	color: rgba(255, 255, 255, 0.6)

	&.error
		color: #ff6b6b
</style>
