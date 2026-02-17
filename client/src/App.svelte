<script>
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
.loading
	text-align: center
	padding: 40px
	color: rgba(255, 255, 255, 0.6)
</style>
