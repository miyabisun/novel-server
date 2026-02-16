<script>
	import 'normalize.css';
	import { router, navigate, getBasePath } from '$lib/router.svelte.js';
	import { auth, checkAuth } from '$lib/auth.svelte.js';
	import Header from '$lib/components/Header.svelte';
	import Ranking from './pages/Ranking.svelte';
	import Reader from './pages/Reader.svelte';
	import Login from './pages/Login.svelte';
	import Favorites from './pages/Favorites.svelte';

	checkAuth();

	$effect(() => {
		if (auth.loading) return;
		if (!auth.authenticated && router.index !== 1) {
			navigate('/login');
		} else if (auth.authenticated && router.index === 1) {
			navigate('/');
		}
	});

	$effect(() => {
		function handleClick(e) {
			const a = e.target.closest('a');
			if (!a) return;
			const href = a.getAttribute('href');
			if (!href || href.startsWith('http') || href.startsWith('//')) return;
			if (e.ctrlKey || e.metaKey || e.shiftKey || e.altKey) return;

			const base = getBasePath();
			let path = href;
			if (base && path.startsWith(base)) {
				path = path.slice(base.length) || '/';
			}

			e.preventDefault();
			navigate(path);
		}

		document.addEventListener('click', handleClick);
		return () => document.removeEventListener('click', handleClick);
	});
</script>

{#if auth.loading}
	<p class="loading">読み込み中...</p>
{:else if router.index === 1}
	<Login />
{:else}
	<Header />
	{#if router.index === 0}
		<Ranking />
	{:else if router.index === 2}
		<Favorites />
	{:else if router.index === 3}
		<Reader params={router.params} />
	{/if}
{/if}

<style lang="sass">
:global
	*,
	*::before,
	*::after
		box-sizing: border-box

	body
		background-color: #222
		color: rgba(255, 255, 255, 0.85)
		font-size: clamp(12px, 1vw, 18px)
		line-height: 1.5715

	h1, h2, h3, h4, h5, h6
		color: rgba(255, 255, 255, 0.85)

	a
		color: rgba(128, 192, 255, 0.85)
		text-decoration: none

		&:hover
			color: rgba(192, 222, 255, 0.85)

	table
		table-layout: fixed
		border-collapse: collapse
		width: 100%

		thead th
			padding: 2px 4px
			border-bottom: 3px gray double
			font-size: 0.9rem

		tbody td
			padding: 2px 4px
			vertical-align: top

		tr
			&:hover
				background-color: rgba(255, 255, 255, 0.2)

	ul
		margin: 0
		padding: 0
		list-style: none

	li
		margin: 0
		padding: 0

.loading
	text-align: center
	padding: 40px
	color: rgba(255, 255, 255, 0.6)
</style>
