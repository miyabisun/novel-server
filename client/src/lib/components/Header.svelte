<script>
	import { onMount } from 'svelte';
	import { router, link } from '$lib/router.svelte.js';
	import { navItems } from '$lib/constants.js';
	import fetcher from '$lib/fetcher.js';

	let email = $state(null);

	onMount(async () => {
		const res = await fetcher('/api/auth/me').catch(() => null);
		if (res?.email && res.email !== 'guest') {
			email = res.email;
		}
	});

	function isActive(item) {
		if (item.path === '/') return router.index === 0;
		return router.index === 1 && router.params.type === item.label;
	}
</script>

<header>
	<nav class="nav-left">
		<span class="title">novel-server</span>
		{#each navItems as item}
			<a
				class="nav-item"
				class:active={isActive(item)}
				href={link(item.path)}
				style:--tab-color={item.color}
			>{item.label}</a>
		{/each}
	</nav>
	{#if email}
		<span class="nav-right">{email}</span>
	{/if}
</header>

<style lang="sass">
header
	position: sticky
	top: 0
	z-index: 100
	padding: 0 var(--sp-lg)
	display: flex
	align-items: center
	justify-content: space-between
	border-bottom: 1px solid var(--c-border)
	background: var(--c-bg)

.nav-left
	display: flex
	align-items: center
	gap: 0

.nav-right
	color: var(--c-text-muted)
	font-size: var(--fs-caption)
	padding: var(--sp-md) 0
	white-space: nowrap
	overflow: hidden
	text-overflow: ellipsis
	max-width: 200px

	@media (max-width: 799px)
		display: none

.title
	color: var(--c-text-muted)
	font-size: var(--fs-label)
	padding: var(--sp-md) var(--sp-lg) var(--sp-md) 0
	margin-right: var(--sp-xs)
	border-right: 1px solid var(--c-border)
	user-select: none
	pointer-events: none

	@media (max-width: 799px)
		display: none

.nav-item
	padding: var(--sp-md) var(--sp-lg)
	color: var(--c-text-muted)
	text-decoration: none
	font-size: var(--fs-label)
	font-weight: 500
	border-bottom: 2px solid transparent
	margin-bottom: -1px

	// Tabs never change background (template rule): only color + underline.
	&:hover
		color: var(--c-text)

	&.active
		color: var(--c-text)
		border-bottom-color: var(--tab-color)
</style>
