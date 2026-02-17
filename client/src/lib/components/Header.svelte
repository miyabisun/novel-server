<script>
	import { router, link } from '$lib/router.svelte.js';

	const navItems = [
		{ label: 'favorite', href: '/' },
		{ label: 'narou', href: '/ranking/narou' },
		{ label: 'kakuyomu', href: '/ranking/kakuyomu' },
		{ label: 'nocturne', href: '/ranking/nocturne' },
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
	margin-bottom: 16px
	position: sticky
	top: 0
	background: #222
	z-index: 100

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
		border-bottom-color: rgba(128, 192, 255, 0.7)
</style>
