<script>
	import { onMount } from 'svelte';
	import { router, link } from '$lib/router.svelte.js';
	import { navItems } from '$lib/constants.js';
	import fetcher from '$lib/fetcher.js';
	import Icon from '$lib/components/Icon.svelte';
	import ThemeModal from '$lib/components/ThemeModal.svelte';
	import { readerChrome } from '$lib/readerChrome.svelte.js';

	let email = $state(null);
	let menuOpen = $state(false);
	let themeOpen = $state(false);
	let menuButton = $state(null);
	let isCompact = $state(false);

	onMount(() => {
		const mq = window.matchMedia('(max-width: 799px)');
		const syncCompact = () => {
			isCompact = mq.matches;
		};
		syncCompact();
		mq.addEventListener('change', syncCompact);

		// Keep onMount synchronous so Svelte registers this cleanup (async returns a Promise).
		void (async () => {
			const res = await fetcher('/api/auth/me').catch(() => null);
			if (res?.email && res.email !== 'guest') {
				email = res.email;
			}
		})();

		return () => mq.removeEventListener('change', syncCompact);
	});

	function isActive(item) {
		if (item.path === '/') return router.index === 0;
		return router.index === 1 && router.params.type === item.label;
	}

	function closeMenu() {
		menuOpen = false;
		menuButton?.focus();
	}

	function openTheme() {
		menuOpen = false;
		themeOpen = true;
	}

	function closeTheme() {
		themeOpen = false;
		menuButton?.focus();
	}

	function onReaderToc() {
		closeMenu();
		readerChrome.goToc?.();
	}

	function onReaderUnfav() {
		closeMenu();
		readerChrome.requestUnfav?.();
	}

	function onkeydown(event) {
		if (menuOpen && event.key === 'Escape') {
			event.preventDefault();
			closeMenu();
		}
	}

	let showReaderMenu = $derived(
		isCompact && readerChrome.active && (readerChrome.showToc || readerChrome.showUnfav),
	);
</script>

<svelte:window {onkeydown} />

<header>
	<nav class="nav-left">
		<span class="title">novel-server</span>
		{#each navItems as item}
			<a
				class="nav-item"
				class:active={isActive(item)}
				href={link(item.path)}
				style:--tab-color={item.color}
			>
				{#if item.short}
					<span class="label-full">{item.label}</span>
					<span class="label-short">{item.short}</span>
				{:else}
					{item.label}
				{/if}
			</a>
		{/each}
	</nav>
	<div class="menu-wrapper">
		<button
			class="icon-btn"
			type="button"
			aria-label="メニュー"
			aria-expanded={menuOpen}
			bind:this={menuButton}
			onclick={() => (menuOpen = !menuOpen)}
		>
			<Icon name="menu" />
		</button>
		{#if menuOpen}
			<button
				class="menu-overlay"
				type="button"
				tabindex="-1"
				aria-label="メニューを閉じる"
				onclick={closeMenu}
			></button>
			<nav class="menu" aria-label="アプリメニュー">
				{#if email}
					<div class="menu-email">{email}</div>
				{/if}
				<button class="menu-item" type="button" onclick={openTheme}>テーマ設定</button>
				{#if showReaderMenu}
					{#if readerChrome.showToc}
						<button class="menu-item" type="button" onclick={onReaderToc}>目次</button>
					{/if}
					{#if readerChrome.showUnfav}
						<button class="menu-item danger" type="button" onclick={onReaderUnfav}>
							お気に入りから削除
						</button>
					{/if}
				{/if}
			</nav>
		{/if}
	</div>
</header>

{#if themeOpen}
	<ThemeModal onclose={closeTheme} />
{/if}

<style lang="sass">
header
	position: sticky
	top: 0
	z-index: 100
	height: var(--header-h)
	padding: 0 var(--sp-lg)
	display: flex
	align-items: center
	justify-content: space-between
	border-bottom: 1px solid var(--c-border)
	background: var(--c-bg)

.nav-left
	display: flex
	align-items: center
	gap: 0
	min-width: 0
	flex: 1

.title
	color: var(--c-text-muted)
	font-size: var(--fs-label)
	padding: 0 var(--sp-lg) 0 0
	margin-right: var(--sp-xs)
	border-right: 1px solid var(--c-border)
	user-select: none
	pointer-events: none
	white-space: nowrap

	@media (max-width: 799px)
		display: none

.nav-item
	padding: 0 var(--sp-md)
	height: var(--header-h)
	display: inline-flex
	align-items: center
	color: var(--c-text-muted)
	text-decoration: none
	font-size: var(--fs-label)
	font-weight: 500
	border-bottom: 2px solid transparent
	margin-bottom: -1px
	white-space: nowrap

	@media (max-width: 799px)
		padding: 0 var(--sp-sm)

	// Tabs never change background (template rule): only color + underline.
	&:hover
		color: var(--c-text)

	&.active
		color: var(--c-text)
		border-bottom-color: var(--tab-color)

.label-full
	@media (max-width: 799px)
		display: none

.label-short
	display: none

	@media (max-width: 799px)
		display: inline

.menu-wrapper
	position: relative
	display: flex
	align-items: center
	align-self: stretch
	flex-shrink: 0

.menu-overlay
	position: fixed
	inset: 0
	z-index: 109
	padding: 0
	border: none
	background: transparent
	cursor: default

.menu
	position: absolute
	top: 100%
	right: 0
	z-index: 110
	display: flex
	flex-direction: column
	min-width: 180px
	overflow: hidden
	border: 1px solid var(--c-border)
	border-radius: var(--radius-lg)
	background: var(--c-surface)
	box-shadow: 0 8px 32px rgba(0, 0, 0, 0.25)

.menu-email
	padding: var(--sp-sm) var(--sp-md)
	border-bottom: 1px solid var(--c-border)
	color: var(--c-text-muted)
	font-size: var(--fs-caption)
	white-space: nowrap
	overflow: hidden
	text-overflow: ellipsis
	max-width: 240px

.menu-item
	display: block
	width: 100%
	padding: var(--sp-sm) var(--sp-md)
	border: none
	background: transparent
	color: var(--c-text)
	font-size: var(--fs-label)
	font-weight: 500
	text-align: left
	cursor: pointer

	&:hover
		background: var(--c-surface-hover)

	&.danger
		color: var(--c-danger)

		&:hover
			background: var(--c-danger-subtle)
</style>
