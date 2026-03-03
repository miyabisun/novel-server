<script>
	let { message, onconfirm, oncancel } = $props();

	function handleKeydown(e) {
		if (e.key === 'Escape') oncancel();
	}

	function handleBackdrop(e) {
		if (e.target === e.currentTarget) oncancel();
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="backdrop" onclick={handleBackdrop}>
	<div class="modal">
		<p class="modal-message">{message}</p>
		<div class="modal-actions">
			<button class="btn btn-cancel" onclick={oncancel}>キャンセル</button>
			<button class="btn btn-delete" onclick={onconfirm}>削除</button>
		</div>
	</div>
</div>

<style lang="sass">
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
</style>
