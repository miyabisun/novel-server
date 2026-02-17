<script>
	import config from '$lib/config.js';
	import fetcher from '$lib/fetcher.js';
	import { navigate } from '$lib/router.svelte.js';

	let { type, novel, isFav = false, onToggleFav, onclose } = $props();

	let synopsis = $state('');
	let loading = $state(true);
	let error = $state(null);
	let saving = $state(false);

	$effect(() => {
		document.body.style.overflow = 'hidden';
		return () => { document.body.style.overflow = ''; };
	});

	$effect(() => {
		loading = true;
		error = null;
		fetcher(`${config.path.api}/novel/${type}/${novel.id}/detail`)
			.then((data) => { synopsis = data.synopsis ?? ''; })
			.catch((e) => { error = e.message; })
			.finally(() => { loading = false; });
	});

	function handleKeydown(e) {
		if (e.key === 'Escape') onclose();
	}

	function handleBackdrop(e) {
		if (e.target === e.currentTarget) onclose();
	}

	function goRead() {
		const path = `/novel/${type}/${novel.id}/1`;
		onclose();
		navigate(path);
	}

	async function toggleFavorite() {
		if (saving) return;
		saving = true;
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
			onToggleFav?.(novel.id);
		} catch (err) {
			alert(err.message);
		} finally {
			saving = false;
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="backdrop" onclick={handleBackdrop}>
	<div class="modal">
		<button class="close-btn" onclick={onclose}>&times;</button>
		<h2 class="title">{novel.title}</h2>

		{#if loading}
			<p class="status">読み込み中...</p>
		{:else if error}
			<p class="status error">{error}</p>
		{:else}
			<div class="synopsis">{synopsis || 'あらすじなし'}</div>
		{/if}

		<div class="actions">
			<button class="btn btn-primary" onclick={goRead}>第1話を読む</button>
			<button
				class="btn btn-fav"
				onclick={toggleFavorite}
				disabled={saving}
			>
				{#if saving}
					保存中...
				{:else if isFav}
					★ お気に入り解除
				{:else}
					☆ お気に入り追加
				{/if}
			</button>
		</div>
	</div>
</div>

<style lang="sass">
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
	max-width: 540px
	width: 100%
	max-height: 80vh
	overflow-y: auto
	position: relative

.close-btn
	position: absolute
	top: 12px
	right: 12px
	background: transparent
	border: none
	color: rgba(255, 255, 255, 0.6)
	font-size: 1.5rem
	cursor: pointer
	line-height: 1

	&:hover
		color: white

.title
	margin: 0 0 16px
	font-size: 1.1rem
	color: white
	padding-right: 30px

.status
	color: rgba(255, 255, 255, 0.6)
	text-align: center
	padding: 20px 0

	&.error
		color: #ff6b6b

.synopsis
	white-space: pre-wrap
	color: rgba(255, 255, 255, 0.85)
	font-size: 0.9rem
	line-height: 1.7
	max-height: 40vh
	overflow-y: auto
	margin-bottom: 20px

.actions
	display: flex
	gap: 8px

.btn
	padding: 8px 16px
	border: 1px solid #555
	border-radius: 4px
	cursor: pointer
	font-size: 0.85rem

.btn-primary
	background: rgba(128, 192, 255, 0.2)
	color: rgba(128, 192, 255, 0.9)
	border-color: rgba(128, 192, 255, 0.4)

	&:hover
		background: rgba(128, 192, 255, 0.3)

.btn-fav
	background: transparent
	color: rgba(255, 200, 50, 0.8)
	border-color: rgba(255, 200, 50, 0.3)

	&:hover:not(:disabled)
		background: rgba(255, 200, 50, 0.1)

	&:disabled
		cursor: default
		opacity: 0.7
</style>
