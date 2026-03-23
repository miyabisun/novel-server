<script>
	import { router, navigate, getBasePath } from '$lib/router.svelte.js';
	import { navItems } from '$lib/constants.js';
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

	const tabPaths = navItems.map((item) => item.path);

	function getTabIndex() {
		if (router.index === 0) return 0;
		if (router.index === 1) {
			const idx = tabPaths.indexOf(`/ranking/${router.params.type}`);
			return idx >= 0 ? idx : 0;
		}
		return -1;
	}

	function swipeNav(node) {
		let startX, startY, locked, horizontal;

		function onStart(e) {
			startX = e.touches[0].clientX;
			startY = e.touches[0].clientY;
			locked = false;
			horizontal = false;
		}

		function onMove(e) {
			const dx = e.touches[0].clientX - startX;
			const dy = e.touches[0].clientY - startY;
			if (!locked) {
				if (Math.abs(dx) < 5 && Math.abs(dy) < 5) return;
				locked = true;
				horizontal = Math.abs(dx) > Math.abs(dy);
			}
			if (horizontal) e.preventDefault();
		}

		function onEnd(e) {
			if (!locked || !horizontal) return;
			const dx = e.changedTouches[0].clientX - startX;
			const tabIdx = getTabIndex();
			if (tabIdx < 0) return;
			if (dx < -40 && tabIdx < tabPaths.length - 1) {
				navigate(tabPaths[tabIdx + 1]);
			} else if (dx > 40 && tabIdx > 0) {
				navigate(tabPaths[tabIdx - 1]);
			}
		}

		node.addEventListener('touchstart', onStart, { passive: true });
		node.addEventListener('touchmove', onMove, { passive: false });
		node.addEventListener('touchend', onEnd, { passive: true });

		return {
			destroy() {
				node.removeEventListener('touchstart', onStart);
				node.removeEventListener('touchmove', onMove);
				node.removeEventListener('touchend', onEnd);
			},
		};
	}
</script>

<div class="app">
	<Header />
	<main use:swipeNav>
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
