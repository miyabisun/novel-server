<script>
	import Icon from '$lib/components/Icon.svelte';

	let { title, onclose, children } = $props();

	let dialog = $state(null);

	$effect(() => {
		const autofocus = dialog?.querySelector('[data-autofocus]');
		(autofocus ?? dialog)?.focus();
	});

	function onkeydown(event) {
		if (event.key === 'Escape') {
			event.preventDefault();
			onclose();
			return;
		}
		if (event.key !== 'Tab' || !dialog) return;

		const focusables = dialog.querySelectorAll(
			'button, a[href], input, select, textarea',
		);
		if (focusables.length === 0) return;

		const first = focusables[0];
		const last = focusables[focusables.length - 1];
		if (event.shiftKey && document.activeElement === first) {
			event.preventDefault();
			last.focus();
		} else if (!event.shiftKey && document.activeElement === last) {
			event.preventDefault();
			first.focus();
		}
	}
</script>

<svelte:window {onkeydown} />

<button class="scrim" type="button" aria-label="閉じる" onclick={onclose}></button>
<div
	class="modal"
	role="dialog"
	aria-modal="true"
	aria-label={title}
	tabindex="-1"
	bind:this={dialog}
>
	<div class="modal-head">
		<h2>{title}</h2>
		<button class="icon-btn" type="button" aria-label="閉じる" onclick={onclose}>
			<Icon name="x" />
		</button>
	</div>
	{@render children()}
</div>

<style lang="sass">
.scrim
	position: fixed
	inset: 0
	z-index: 200
	padding: 0
	border: none
	background: var(--c-backdrop)
	cursor: default

.modal
	position: fixed
	z-index: 201
	top: 50%
	left: 50%
	transform: translate(-50%, -50%)
	width: min(360px, calc(100vw - 32px))
	max-height: 80dvh
	overflow-y: auto
	padding: var(--sp-lg)
	border: 1px solid var(--c-border)
	border-radius: var(--radius-lg)
	background: var(--c-surface)
	box-shadow: 0 8px 32px rgba(0, 0, 0, 0.25)

.modal-head
	display: flex
	align-items: center
	justify-content: space-between
	margin-bottom: var(--sp-md)

	h2
		margin: 0
		font-size: var(--fs-title)
		font-weight: 600
		line-height: 1.3
		color: var(--c-text)
</style>
