<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { getSession } from '$lib/auth/sso';
	import { isAuthenticated as hasOAuthTokens } from '$lib/auth/oauth';

	const LOGIN_URL = 'https://arack.io/auth/login';

	onMount(async () => {
		console.log('[Root] Checking authentication...');

		// If we have OAuth tokens, go to inbox
		if (hasOAuthTokens()) {
			console.log('[Root] OAuth tokens found, redirecting to inbox');
			goto('/inbox');
			return;
		}

		// Check SSO session (this also stores tokens from session to localStorage)
		console.log('[Root] No OAuth tokens, checking SSO session...');
		const session = await getSession();

		if (session) {
			// SSO session found and tokens were stored by getSession()
			// Check if tokens are now available
			if (hasOAuthTokens()) {
				console.log('[Root] OAuth tokens obtained from SSO, redirecting to inbox');
				goto('/inbox');
				return;
			}

			// SSO exists but no OAuth tokens (shouldn't happen with token exchange)
			console.log('[Root] SSO session found but no OAuth tokens');
		}

		// No session or no tokens - redirect to login
		console.log('[Root] No valid session, redirecting to login');
		const returnUrl = encodeURIComponent(window.location.origin + '/inbox');
		window.location.href = `${LOGIN_URL}?return_url=${returnUrl}`;
	});
</script>

<div class="flex items-center justify-center h-screen bg-gray-50 dark:bg-gray-900">
	<div class="text-center">
		<div
			class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mb-4"
		></div>
		<p class="text-gray-500 dark:text-gray-400">Checking authentication...</p>
	</div>
</div>
