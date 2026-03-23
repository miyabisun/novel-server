<script>
	import config from '$lib/config.js';
	import fetcher from '$lib/fetcher.js';
	import { link } from '$lib/router.svelte.js';
	import { decodeHtml } from '$lib/decode.js';
	import ConfirmModal from '$lib/components/ConfirmModal.svelte';
	import { typeColors } from '$lib/constants.js';

	let favorites = $state([]);
	let loading = $state(false);
	let error = $state(null);
	let deleteTarget = $state(null);

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

	function confirmDelete(fav) {
		deleteTarget = fav;
	}

	function cancelDelete() {
		deleteTarget = null;
	}

	async function executeDelete() {
		if (!deleteTarget) return;
		try {
			await removeFavorite(deleteTarget.type, deleteTarget.id);
		} finally {
			cancelDelete();
		}
	}

	function formatDate(dateStr) {
		if (!dateStr) return null;
		return dateStr.replace(/:\d{2}$/, '');
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
		<div class="fav-grid">
			{#each favorites as fav (fav.type + ':' + fav.id)}
				<div class="fav-wrapper">
					<div class="fav-card">
						<div class="card-body">
							<div class="card-header">
								<span class="card-info">{fav.read} / {fav.page}{#if fav.novelupdated_at} <span class="card-updated">{formatDate(fav.novelupdated_at)}</span>{/if}</span>
								<span class="card-type" style:--type-color={typeColors[fav.type]}>{fav.type}</span>
							</div>
							<div class="card-title"><a href={link(`/novel/${fav.type}/${fav.id}/${Math.min((fav.read || 0) + 1, fav.page || 1)}`)}>{decodeHtml(fav.title)}</a></div>
						</div>
						<div class="card-actions">
							<button class="delete-btn" onclick={() => confirmDelete(fav)}>✕</button>
						</div>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

{#if deleteTarget}
	<ConfirmModal
		message={`「${decodeHtml(deleteTarget.title)}」を削除しますか？`}
		onconfirm={executeDelete}
		oncancel={cancelDelete}
	/>
{/if}

<style lang="sass">
.favorites
	padding: 0 var(--sp-4)

.fav-grid
	display: flex
	flex-direction: column
	gap: var(--sp-3)

.fav-wrapper
	border-radius: var(--radius-md)
	border: 1px solid var(--c-border)

.fav-card
	display: flex
	color: inherit

@media (min-width: 800px)
	.fav-wrapper:hover .fav-card
		background-color: var(--c-surface-hover)

.card-body
	flex: 1
	min-width: 0
	display: flex
	flex-direction: column
	gap: var(--sp-1)
	padding: var(--sp-3)

.card-header
	display: flex
	justify-content: space-between
	align-items: center
	gap: var(--sp-3)

.card-info
	font-size: var(--fs-xs)
	color: var(--c-text-muted)

.card-updated
	color: var(--c-text-faint)
	margin-left: var(--sp-2)

.card-type
	font-size: var(--fs-xs)
	color: var(--type-color, rgba(255, 255, 255, 0.4))
	border: 1px solid var(--type-color, rgba(255, 255, 255, 0.2))
	border-radius: var(--radius-sm)
	padding: 1px var(--sp-2)
	flex-shrink: 0

.card-title
	line-height: 1.4

	a
		text-decoration: none
		color: inherit

		&:hover
			text-decoration: underline

	@media (min-width: 769px)
		font-size: var(--fs-md)

.card-actions
	display: flex
	align-items: center
	flex-shrink: 0
	border-left: 1px solid var(--c-border)

.delete-btn
	padding: 0 var(--sp-4)
	border: none
	background: transparent
	color: var(--c-danger-dim)
	cursor: pointer
	font-size: var(--fs-sm)
	height: 100%

	&:hover
		background: var(--c-danger-hover)

@media (max-width: 799px)
	.card-actions
		display: none
</style>
