<script>
	import { onMount } from 'svelte';
	import { router, link } from '$lib/router.svelte.js';
	import { typeColors } from '$lib/constants.js';
	import fetcher from '$lib/fetcher.js';

	const navItems = [
		{ label: 'favorite', href: '/', color: 'rgba(220, 180, 50, 0.7)' },
		{ label: 'narou', href: '/ranking/narou', color: typeColors.narou },
		{ label: 'kakuyomu', href: '/ranking/kakuyomu', color: typeColors.kakuyomu },
		{ label: 'nocturne', href: '/ranking/nocturne', color: typeColors.nocturne },
	];

	let email = $state(null);

	onMount(async () => {
		const res = await fetcher('/api/auth/me').catch(() => null);
		if (res?.email && res.email !== 'guest') {
			email = res.email;
		}
	});

	function isActive(item) {
		if (item.href === '/') return router.index === 0;
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
				href={link(item.href)}
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
	padding: 0 var(--sp-4)
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
	font-size: var(--fs-sm)
	padding: var(--sp-3) 0
	white-space: nowrap
	overflow: hidden
	text-overflow: ellipsis
	max-width: 200px

	@media (max-width: 799px)
		display: none

.title
	color: var(--c-text-muted)
	font-size: var(--fs-sm)
	padding: var(--sp-3) var(--sp-4) var(--sp-3) 0
	margin-right: var(--sp-1)
	border-right: 1px solid var(--c-border)
	user-select: none
	pointer-events: none

	@media (max-width: 799px)
		display: none

.nav-item
	padding: var(--sp-3) var(--sp-4)
	color: var(--c-text-sub)
	text-decoration: none
	font-size: var(--fs-sm)
	border-bottom: 2px solid transparent
	margin-bottom: -1px

	&:hover
		color: var(--c-text)
		background: var(--c-overlay-1)

	&.active
		color: white
		border-bottom-color: var(--tab-color)
</style>
