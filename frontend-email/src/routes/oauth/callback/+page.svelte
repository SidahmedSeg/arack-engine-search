<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { handleCallback, getUserInfo, clearOAuthPending, login } from '$lib/auth/oauth';
	import { authStore } from '$lib/stores/auth.svelte';
	import * as Card from '$lib/components/ui/card';

	let isLoading = $state(true);
	let error = $state('');
	let status = $state('Processing authorization...');

	// Login page URL
	const LOGIN_URL = 'https://arack.io/auth/login';

	onMount(async () => {
		// Clear the pending flag since we're in the callback
		clearOAuthPending();

		try {
			// FIRST: Check for OAuth errors in URL (from prompt=none silent auth)
			const urlError = $page.url.searchParams.get('error');
			const errorDescription = $page.url.searchParams.get('error_description');

			if (urlError) {
				console.log('[OAuth Callback] OAuth error:', urlError, errorDescription);

				// Handle silent auth failures (prompt=none)
				if (urlError === 'login_required' || urlError === 'interaction_required') {
					// No Zitadel session exists - need full login
					console.log('[OAuth Callback] Silent auth failed - no Zitadel session');
					console.log('[OAuth Callback] Redirecting to full login...');
					const returnUrl = encodeURIComponent(window.location.origin + '/inbox');
					window.location.href = `${LOGIN_URL}?return_url=${returnUrl}`;
					return;
				}

				if (urlError === 'consent_required') {
					// Need to show consent - retry without prompt=none
					console.log('[OAuth Callback] Consent required - retrying with full OAuth');
					login(); // This will do full OAuth without prompt=none
					return;
				}

				if (urlError === 'access_denied') {
					throw new Error('Access denied. You declined the authorization request.');
				}

				// Other errors - show error message
				throw new Error(`OAuth error: ${urlError} - ${errorDescription || 'Unknown error'}`);
			}

			console.log('[OAuth Callback] Processing OAuth callback');
			status = 'Exchanging authorization code for tokens...';

			// Get current URL with authorization code
			const callbackUrl = window.location.href;

			// Exchange authorization code for tokens
			const tokens = await handleCallback(callbackUrl);

			console.log('[OAuth Callback] Tokens received successfully');
			status = 'Fetching user information...';

			// Get user info using the access token
			const userInfo = await getUserInfo();

			console.log('[OAuth Callback] User info retrieved:', userInfo);
			status = 'Setting up session...';

			// Update auth store with user info
			authStore.setUser({
				id: userInfo.sub,
				email: userInfo.email || '',
				firstName: userInfo.given_name || '',
				lastName: userInfo.family_name || ''
			});

			console.log('[OAuth Callback] Authentication complete, redirecting to inbox');
			status = 'Authentication complete!';

			// Redirect to inbox
			setTimeout(() => {
				goto('/inbox');
			}, 300);
		} catch (err: any) {
			console.error('[OAuth Callback] Error:', err);
			error = err.message || 'Failed to complete authentication';
			isLoading = false;

			// Check for specific OAuth errors
			if (err.message?.includes('state')) {
				error = 'Security validation failed. Please try logging in again.';
			} else if (err.message?.includes('code_verifier')) {
				error = 'Session expired. Please try logging in again.';
			}
		}
	});
</script>

<svelte:head>
	<title>OAuth Callback - Arack Mail</title>
</svelte:head>

<div class="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900 px-4">
	<Card.Root class="w-full max-w-md">
		<Card.Content class="py-8">
			{#if error}
				<div class="text-center space-y-4">
					<div class="text-red-600 dark:text-red-400">
						<svg
							class="mx-auto h-12 w-12 mb-4"
							fill="none"
							stroke="currentColor"
							viewBox="0 0 24 24"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
							/>
						</svg>
						<p class="text-lg font-semibold">Authentication Failed</p>
						<p class="text-sm mt-2">{error}</p>
					</div>

					<div class="text-left bg-gray-100 dark:bg-gray-800 p-4 rounded text-xs space-y-1">
						<p class="font-semibold">Troubleshooting:</p>
						<ul class="list-disc pl-5 space-y-1">
							<li>Try clearing your browser cookies and cache</li>
							<li>Make sure cookies are enabled</li>
							<li>Try using a different browser or incognito mode</li>
						</ul>
					</div>

					<div class="space-y-2">
						<button
							onclick={() => {
								import('$lib/auth/oauth').then(({ login }) => login());
							}}
							class="block w-full bg-blue-600 text-white px-4 py-2 rounded-md hover:bg-blue-700 transition-colors text-center"
						>
							Try Again
						</button>
						<a
							href="/"
							class="block text-sm text-gray-600 hover:text-gray-700 dark:text-gray-400 text-center"
						>
							Back to Home
						</a>
					</div>
				</div>
			{:else}
				<div class="text-center space-y-4">
					<div
						class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mb-4"
					></div>
					<h1 class="text-xl font-semibold text-gray-900 dark:text-white">
						{status}
					</h1>
					<p class="text-sm text-gray-600 dark:text-gray-400">
						Please wait while we complete your authentication
					</p>

					<!-- Progress indicator -->
					<div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2 mt-4">
						<div
							class="bg-blue-600 h-2 rounded-full transition-all duration-500"
							style="width: {status.includes('complete')
								? '100%'
								: status.includes('user')
									? '75%'
									: status.includes('tokens')
										? '50%'
										: '25%'}"
						></div>
					</div>
				</div>
			{/if}
		</Card.Content>
	</Card.Root>
</div>
