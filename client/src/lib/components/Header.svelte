<script>
	import { link } from '$lib/router.svelte.js';
	import { logout } from '$lib/auth.svelte.js';
</script>

<header>
	<ul class="breadcrumbs">
		<li><a href="{link('/')}">novel-server</a></li>
		{#each ['narou', 'kakuyomu', 'nocturne'] as type}
			<li><a href="{link('/')}" data-type={type}>{type}</a></li>
		{/each}
	</ul>
	<nav class="header-nav">
		<a href="{link('/favorites')}">お気に入り</a>
		<button class="logout-btn" onclick={logout}>ログアウト</button>
	</nav>
</header>

<style lang="sass">
header
	padding: 0 15px
	display: flex
	align-items: center
	justify-content: space-between

	.breadcrumbs
		display: flex
		list-style: none
		margin: 0
		padding: 0

		li
			&::before
				content: ">"
				margin: 0 0.3rem
			&:first-of-type
				&::before
					content: ""
					margin: 0
			&:nth-of-type(2)
				&::before
					content: "|"

	.header-nav
		display: flex
		align-items: center
		gap: 12px

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
