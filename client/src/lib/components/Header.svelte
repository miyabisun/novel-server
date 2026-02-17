<script>
	import { router, link } from '$lib/router.svelte.js';
	import { logout } from '$lib/auth.svelte.js';

	const navItems = [
		{ label: 'favorite', href: '/' },
		{ label: 'narou', href: '/ranking/narou' },
		{ label: 'kakuyomu', href: '/ranking/kakuyomu' },
		{ label: 'nocturne', href: '/ranking/nocturne' },
	];

	function isActive(item) {
		if (item.href === '/') return router.index === 0;
		return router.index === 2 && router.params.type === item.label;
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
	<button class="logout-btn" onclick={logout}>logout</button>
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

.logout-btn
	padding: 4px 12px
	border: 1px solid #555
	background: transparent
	color: rgba(255, 255, 255, 0.7)
	cursor: pointer
	border-radius: 4px
	font-size: 0.85rem

	&:hover
		background: rgba(255, 255, 255, 0.1)
</style>
