<script>
	import { router, link } from '$lib/router.svelte.js';

	const navItems = [
		{ label: 'favorite', href: '/', color: 'rgba(220, 180, 50, 0.7)' },
		{ label: 'narou', href: '/ranking/narou', color: 'rgba(100, 190, 120, 0.7)' },
		{ label: 'kakuyomu', href: '/ranking/kakuyomu', color: 'rgba(100, 160, 220, 0.7)' },
		{ label: 'nocturne', href: '/ranking/nocturne', color: 'rgba(200, 110, 110, 0.7)' },
	];

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
</header>

<style lang="sass">
header
	padding: 0 15px
	display: flex
	align-items: center
	justify-content: space-between
	border-bottom: 1px solid #444
	background: #222

.nav-left
	display: flex
	align-items: center
	gap: 0

.title
	color: rgba(255, 255, 255, 0.5)
	font-size: 0.85rem
	padding: 10px 12px 10px 0
	margin-right: 4px
	border-right: 1px solid #444
	user-select: none
	pointer-events: none

	@media (max-width: 799px)
		display: none

.nav-item
	padding: 10px 12px
	color: rgba(255, 255, 255, 0.6)
	text-decoration: none
	font-size: 0.85rem
	border-bottom: 2px solid transparent
	margin-bottom: -1px

	&:hover
		color: rgba(255, 255, 255, 0.85)
		background: rgba(255, 255, 255, 0.05)

	&.active
		color: white
		border-bottom-color: var(--tab-color)
</style>
