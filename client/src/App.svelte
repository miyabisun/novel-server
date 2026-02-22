<script>
	import { router, navigate, getBasePath } from '$lib/router.svelte.js';
	import Header from '$lib/components/Header.svelte';
	import Ranking from './pages/Ranking.svelte';
	import Reader from './pages/Reader.svelte';
	import Favorites from './pages/Favorites.svelte';
	import TableOfContents from './pages/TableOfContents.svelte';

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

<div class="app">
	<Header />
	<main>
		{#if router.index === 0}
			<Favorites />
		{:else if router.index === 1}
			<Ranking type={router.params.type} />
		{:else if router.index === 2}
			<TableOfContents params={router.params} />
		{:else if router.index === 3}
			<Reader params={router.params} />
		{/if}
	</main>
</div>

<style lang="sass">
.app
	display: grid
	grid-template-rows: auto 1fr
	height: 100dvh

main
	overflow-y: auto
</style>
