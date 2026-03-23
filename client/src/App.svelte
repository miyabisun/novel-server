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

	// Tab swipe navigation
	const tabPaths = ['/', '/ranking/narou', '/ranking/kakuyomu', '/ranking/nocturne'];

	function getTabIndex() {
		if (router.index === 0) return 0;
		if (router.index === 1) {
			const idx = tabPaths.indexOf(`/ranking/${router.params.type}`);
			return idx >= 0 ? idx : 0;
		}
		return -1;
	}

	let touchStartX, touchStartY, locked, horizontal;

	function onTouchStart(e) {
		touchStartX = e.touches[0].clientX;
		touchStartY = e.touches[0].clientY;
		locked = false;
		horizontal = false;
	}

	function onTouchMove(e) {
		const dx = e.touches[0].clientX - touchStartX;
		const dy = e.touches[0].clientY - touchStartY;
		if (!locked) {
			if (Math.abs(dx) < 5 && Math.abs(dy) < 5) return;
			locked = true;
			horizontal = Math.abs(dx) > Math.abs(dy);
		}
	}

	function onTouchEnd(e) {
		if (!locked || !horizontal) return;
		const dx = e.changedTouches[0].clientX - touchStartX;
		const tabIdx = getTabIndex();
		if (tabIdx < 0) return;
		if (dx < -40 && tabIdx < tabPaths.length - 1) {
			navigate(tabPaths[tabIdx + 1]);
		} else if (dx > 40 && tabIdx > 0) {
			navigate(tabPaths[tabIdx - 1]);
		}
	}
</script>

<div class="app">
	<Header />
	<main
		ontouchstart={onTouchStart}
		ontouchmove={onTouchMove}
		ontouchend={onTouchEnd}
	>
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
	min-height: 100dvh
</style>
