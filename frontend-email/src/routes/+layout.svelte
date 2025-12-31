<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { emailStore } from '$lib/stores/email.svelte';
	import { authStore } from '$lib/stores/auth.svelte';
	import { isAuthenticated } from '$lib/auth/oauth';

	let { children } = $props();

	// Dark mode state (persisted in localStorage)
	let darkMode = $state(false);

	// Pages that don't require authentication
	const publicRoutes = ['/oauth/callback', '/oauth/auto'];

	onMount(async () => {
		// Load dark mode preference from localStorage
		const stored = localStorage.getItem('darkMode');
		darkMode = stored === 'true';
		updateDarkMode();

		// Check if current route requires authentication
		const currentPath = $page.url.pathname;
		const isPublicRoute = publicRoutes.some((route) => currentPath.startsWith(route));

		if (isPublicRoute) {
			// Public route, skip auth check
			return;
		}

		// Check OAuth session
		if (!isAuthenticated()) {
			console.log('[Layout] No OAuth tokens, skipping initialization');
			return;
		}

		// Validate session and get user info
		await authStore.checkSession();

		if (!authStore.isAuthenticated) {
			console.log('[Layout] Session invalid, skipping initialization');
			return;
		}

		// Initialize email store with user account
		try {
			await emailStore.initialize();
			console.log('[Layout] Email store initialized with account:', emailStore.accountInfo?.email);
		} catch (err) {
			console.error('[Layout] Failed to initialize email store:', err);
			// API interceptor will redirect to login on 401
		}
	});

	function updateDarkMode() {
		if (darkMode) {
			document.documentElement.classList.add('dark');
		} else {
			document.documentElement.classList.remove('dark');
		}
		localStorage.setItem('darkMode', darkMode.toString());
	}

	function toggleDarkMode() {
		darkMode = !darkMode;
		updateDarkMode();
	}

	// Expose toggle function globally for components
	if (typeof window !== 'undefined') {
		(window as any).toggleDarkMode = toggleDarkMode;
	}
</script>

{@render children()}
