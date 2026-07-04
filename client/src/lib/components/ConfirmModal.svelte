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
	padding: var(--sp-xl)

.modal
	background: var(--c-surface)
	border: 1px solid var(--c-border)
	border-radius: var(--radius-lg)
	padding: var(--sp-lg)
	max-width: 360px
	width: 100%
	box-shadow: 0 8px 32px rgba(0, 0, 0, 0.25)

.modal-message
	margin: 0 0 var(--sp-lg)
	font-size: var(--fs-body)
	color: var(--c-text)
	line-height: 1.6
	overflow-wrap: break-word

.modal-actions
	display: flex
	gap: var(--sp-sm)
	justify-content: flex-end

.btn
	padding: var(--sp-sm) var(--sp-lg)
	border: 1px solid var(--c-border)
	border-radius: var(--radius-sm)
	cursor: pointer
	font-size: var(--fs-label)
	font-weight: 500

.btn-cancel
	background: transparent
	color: var(--c-text-muted)

	&:hover
		background: var(--c-border)

.btn-delete
	background: transparent
	color: var(--c-danger)

	&:hover
		background: var(--c-danger-subtle)
</style>
