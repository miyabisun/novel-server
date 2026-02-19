<script>
	import config from '$lib/config.js';
	import fetcher from '$lib/fetcher.js';
	import { link } from '$lib/router.svelte.js';
	import { decodeHtml } from '$lib/decode.js';

	const typeColors = {
		narou: 'rgba(100, 190, 120, 0.7)',
		kakuyomu: 'rgba(100, 160, 220, 0.7)',
		nocturne: 'rgba(200, 110, 110, 0.7)',
	};

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

	function handleKeydown(e) {
		if (deleteTarget && e.key === 'Escape') cancelDelete();
	}

	function handleBackdrop(e) {
		if (e.target === e.currentTarget) cancelDelete();
	}

	// Swipe action for touch devices
	function swipeable(node, opts) {
		let startX, startY, offsetX, locked, horizontal;
		const { onConfirmDelete } = opts;

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
			node.style.transition = 'none';
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
			offsetX = Math.max(-80, Math.min(0, dx));
			node.style.transform = `translateX(${offsetX}px)`;
		}

		function onEnd() {
			if (!locked) return;
			if (horizontal) {
				node.addEventListener('click', preventClick, { once: true, capture: true });
				if (offsetX < -40) onConfirmDelete();
			}
			node.style.transition = 'transform 0.2s ease';
			node.style.transform = 'translateX(0)';
			offsetX = 0;
		}

		node.addEventListener('touchstart', onStart, { passive: true });
		node.addEventListener('touchmove', onMove, { passive: false });
		node.addEventListener('touchend', onEnd, { passive: true });

		return {
			destroy() {
				node.removeEventListener('touchstart', onStart);
				node.removeEventListener('touchmove', onMove);
				node.removeEventListener('touchend', onEnd);
			},
		};
	}

	function formatDate(dateStr) {
		if (!dateStr) return null;
		return dateStr.replace(/:\d{2}$/, '');
	}

	loadFavorites();
</script>

<svelte:window onkeydown={handleKeydown} />

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
					<div class="swipe-bg">削除</div>
					<div
						class="fav-card"
						use:swipeable={{ onConfirmDelete: () => confirmDelete(fav) }}
					>
						<div class="card-body">
							<div class="card-header">
								<span class="card-info">{fav.read} / {fav.page}{#if fav.novelupdated_at} <span class="card-updated">{formatDate(fav.novelupdated_at)}</span>{/if}</span>
								<span class="card-type" style:--type-color={typeColors[fav.type]}>{fav.type}</span>
							</div>
							<div class="card-title"><a href={link(`/novel/${fav.type}/${fav.id}/${fav.read || 1}`)}>{decodeHtml(fav.title)}</a></div>
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
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="backdrop" onclick={handleBackdrop}>
		<div class="modal">
			<p class="modal-message">「{decodeHtml(deleteTarget.title)}」を削除しますか？</p>
			<div class="modal-actions">
				<button class="btn btn-cancel" onclick={cancelDelete}>キャンセル</button>
				<button class="btn btn-delete" onclick={executeDelete}>削除</button>
			</div>
		</div>
	</div>
{/if}

<style lang="sass">
.favorites
	padding: 0 15px

.fav-grid
	display: flex
	flex-direction: column
	gap: 8px

.fav-wrapper
	border-radius: 6px
	border: 1px solid #444

.fav-card
	display: flex
	color: inherit

.fav-wrapper:hover .fav-card
	background-color: rgba(255, 255, 255, 0.08)

.card-body
	flex: 1
	min-width: 0
	display: flex
	flex-direction: column
	gap: 4px
	padding: 10px

.card-header
	display: flex
	justify-content: space-between
	align-items: center
	gap: 8px

.card-info
	font-size: 0.8rem
	color: rgba(255, 255, 255, 0.5)

.card-updated
	color: rgba(255, 255, 255, 0.35)
	margin-left: 6px

.card-type
	font-size: 0.7rem
	color: var(--type-color, rgba(255, 255, 255, 0.4))
	border: 1px solid var(--type-color, rgba(255, 255, 255, 0.2))
	border-radius: 3px
	padding: 1px 5px
	flex-shrink: 0

.card-title
	line-height: 1.4

	a
		text-decoration: none
		color: inherit

		&:hover
			text-decoration: underline

	@media (min-width: 769px)
		font-size: 1rem

.card-actions
	display: flex
	align-items: center
	flex-shrink: 0
	border-left: 1px solid #444

.swipe-bg
	display: none

.delete-btn
	padding: 0 12px
	border: none
	background: transparent
	color: rgba(255, 100, 100, 0.8)
	cursor: pointer
	font-size: 0.85rem
	height: 100%

	&:hover
		background: rgba(255, 100, 100, 0.15)

.status
	text-align: center
	padding: 20px
	color: rgba(255, 255, 255, 0.6)

	&.error
		color: #ff6b6b

// Delete confirmation modal
.backdrop
	position: fixed
	inset: 0
	background: rgba(0, 0, 0, 0.6)
	z-index: 200
	display: flex
	align-items: center
	justify-content: center
	padding: 20px

.modal
	background: #2a2a2a
	border: 1px solid #555
	border-radius: 8px
	padding: 24px
	max-width: 360px
	width: 100%

.modal-message
	margin: 0 0 20px
	font-size: 1rem
	color: rgba(255, 255, 255, 0.9)
	line-height: 1.6
	overflow-wrap: break-word

.modal-actions
	display: flex
	gap: 8px
	justify-content: flex-end

.btn
	padding: 8px 16px
	border: 1px solid #555
	border-radius: 4px
	cursor: pointer
	font-size: 0.85rem

.btn-cancel
	background: transparent
	color: rgba(255, 255, 255, 0.7)

	&:hover
		background: rgba(255, 255, 255, 0.08)

.btn-delete
	background: rgba(255, 100, 100, 0.2)
	color: rgba(255, 100, 100, 0.9)
	border-color: rgba(255, 100, 100, 0.4)

	&:hover
		background: rgba(255, 100, 100, 0.3)

// Mobile: swipe-to-delete
@media (max-width: 799px)
	.fav-wrapper
		position: relative
		overflow: hidden

	.fav-card
		background: #222
		position: relative
		z-index: 1

	.card-actions
		display: none

	.swipe-bg
		display: flex
		align-items: center
		justify-content: flex-end
		padding-right: 20px
		position: absolute
		right: 0
		top: 0
		bottom: 0
		width: 80px
		color: rgba(255, 100, 100, 0.9)
		font-weight: bold
		font-size: 0.9rem
</style>
