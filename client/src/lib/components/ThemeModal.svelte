<script>
	import Icon from '$lib/components/Icon.svelte';
	import Modal from '$lib/components/Modal.svelte';
	import { loadTheme, saveTheme } from '$lib/theme.js';

	let { onclose } = $props();

	let choice = $state(loadTheme());

	const options = [
		{ value: 'system', label: '自動', icon: 'monitor' },
		{ value: 'light', label: 'ライト', icon: 'sun' },
		{ value: 'dark', label: 'ダーク', icon: 'moon' },
		{ value: 'e-paper', label: '電子ペーパー', icon: 'book' },
	];

	// Keep the modal open so the user can see the theme change live.
	function choose(value) {
		choice = value;
		saveTheme(value);
	}
</script>

<Modal title="テーマ設定" {onclose}>
	<div class="options" role="radiogroup" aria-label="テーマ">
		{#each options as option (option.value)}
			<button
				class="option"
				class:selected={choice === option.value}
				type="button"
				role="radio"
				aria-checked={choice === option.value}
				data-autofocus={choice === option.value ? '' : undefined}
				onclick={() => choose(option.value)}
			>
				<Icon name={option.icon} />
				<span>{option.label}</span>
			</button>
		{/each}
	</div>
</Modal>

<style lang="sass">
.options
	display: flex
	flex-direction: column
	gap: var(--sp-sm)

.option
	display: flex
	align-items: center
	gap: var(--sp-sm)
	min-height: 44px
	padding: var(--sp-sm) var(--sp-md)
	border: 1px solid var(--c-border)
	border-radius: var(--radius-sm)
	background: var(--c-surface)
	color: var(--c-text)
	font-size: var(--fs-label)
	font-weight: 500
	cursor: pointer

	&:hover
		background: var(--c-surface-hover)

	&.selected
		border-color: var(--c-accent)
		background: var(--c-accent-subtle)
</style>
