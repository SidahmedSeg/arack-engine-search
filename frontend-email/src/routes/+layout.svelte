<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { getSession } from '$lib/auth/sso';
	import { isAuthenticated as hasOAuthTokens } from '$lib/auth/oauth';

	let { children } = $props();

	// Dark mode state
	let darkMode = $state(false);

	// Public routes that skip auth
	const publicRoutes = ['/oauth/callback', '/oauth/auto', '/consent'];

	// Login page URL
	const LOGIN_URL = 'https://arack.io/auth/login';

	onMount(async () => {
		// Load dark mode preference
		const stored = localStorage.getItem('darkMode');
		darkMode = stored === 'true';
		updateDarkMode();

		// Check if on public route
		const currentPath = $page.url.pathname;
		const isPublicRoute = publicRoutes.some((route) => currentPath.startsWith(route));

		if (isPublicRoute) {
			return; // Skip auth for public routes
		}

		// STEP 1: Check if we already have valid OAuth tokens in localStorage
		if (hasOAuthTokens()) {
			console.log('[Layout] Valid OAuth tokens found in localStorage');
			return; // Good to go, tokens will be used for JMAP API calls
		}

		// STEP 2: Check SSO session (this also stores tokens from session to localStorage)
		console.log('[Layout] No OAuth tokens, checking SSO session...');
		const session = await getSession();

		if (!session) {
			// No SSO session - redirect to login
			console.log('[Layout] No SSO session, redirecting to login');
			const returnUrl = encodeURIComponent(window.location.href);
			window.location.href = `${LOGIN_URL}?return_url=${returnUrl}`;
			return;
		}

		// SSO session found and tokens were stored by getSession()
		// Check if tokens are now available
		if (hasOAuthTokens()) {
			console.log('[Layout] OAuth tokens obtained from SSO session');
			return; // Tokens now stored, good to go
		}

		// SSO exists but no OAuth tokens in session (shouldn't happen with token exchange)
		console.log('[Layout] SSO session found but no OAuth tokens, redirecting to login');
		const returnUrl = encodeURIComponent(window.location.href);
		window.location.href = `${LOGIN_URL}?return_url=${returnUrl}`;
	});

	function updateDarkMode() {
		if (darkMode) {
			document.documentElement.classList.add('dark');
		} else {
			document.documentElement.classList.remove('dark');
		}
		localStorage.setItem('darkMode', darkMode.toString());
	}
</script>

{@render children()}
