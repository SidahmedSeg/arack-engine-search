<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { Check, X, Loader2 } from 'lucide-svelte';

	let status = $state<'processing' | 'success' | 'error'>('processing');
	let errorMessage = $state<string | null>(null);

	onMount(async () => {
		// Extract params from URL
		const params = new URLSearchParams(window.location.search);
		const error = params.get('error');
		const errorDescription = params.get('error_description');
		const code = params.get('code');
		const state = params.get('state');

		// Check for OAuth errors first
		if (error) {
			status = 'error';
			errorMessage = errorDescription || error;
			setTimeout(() => {
				goto('/oauth/auto');
			}, 3000);
			return;
		}

		// Validate we have the authorization code
		if (!code || !state) {
			status = 'error';
			errorMessage = 'Missing authorization code or state parameter';
			setTimeout(() => {
				goto('/oauth/auto');
			}, 3000);
			return;
		}

		try {
			// Call backend to exchange authorization code for tokens
			const response = await fetch(
				`https://api-mail.arack.io/api/mail/oauth/callback?code=${encodeURIComponent(
					code
				)}&state=${encodeURIComponent(state)}`,
				{
					method: 'GET',
					credentials: 'include', // Important: send cookies with request
					headers: {
						Accept: 'application/json'
					}
				}
			);

			if (!response.ok) {
				const errorData = await response.json().catch(() => ({}));
				throw new Error(errorData.error || `HTTP ${response.status}: ${response.statusText}`);
			}

			const data = await response.json();

			// Success - tokens stored in backend
			status = 'success';

			// Redirect to inbox after 2 seconds
			setTimeout(() => {
				goto('/inbox');
			}, 2000);
		} catch (err) {
			status = 'error';
			errorMessage = err instanceof Error ? err.message : 'Failed to complete OAuth authorization';
			console.error('OAuth callback error:', err);

			// Redirect to retry OAuth flow
			setTimeout(() => {
				goto('/oauth/auto');
			}, 3000);
		}
	});
</script>

<svelte:head>
	<title>OAuth Callback - Arack Mail</title>
</svelte:head>

<div class="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900 p-4">
	<div class="max-w-md w-full">
		<div class="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-8">
			{#if status === 'processing'}
				<div class="text-center">
					<Loader2 class="h-16 w-16 text-blue-600 dark:text-blue-400 mx-auto mb-4 animate-spin" />
					<h2 class="text-2xl font-semibold text-gray-900 dark:text-gray-100 mb-2">
						Connecting your account...
					</h2>
					<p class="text-gray-600 dark:text-gray-400">
						Please wait while we complete the OAuth authorization.
					</p>
				</div>
			{:else if status === 'success'}
				<div class="text-center">
					<div
						class="w-16 h-16 rounded-full bg-green-100 dark:bg-green-900 flex items-center justify-center mx-auto mb-4"
					>
						<Check class="h-10 w-10 text-green-600 dark:text-green-400" />
					</div>
					<h2 class="text-2xl font-semibold text-gray-900 dark:text-gray-100 mb-2">
						Account Connected!
					</h2>
					<p class="text-gray-600 dark:text-gray-400 mb-4">
						Your email account has been successfully connected via OAuth.
					</p>
					<p class="text-sm text-gray-500 dark:text-gray-500">Opening your inbox...</p>
				</div>
			{:else if status === 'error'}
				<div class="text-center">
					<div
						class="w-16 h-16 rounded-full bg-red-100 dark:bg-red-900 flex items-center justify-center mx-auto mb-4"
					>
						<X class="h-10 w-10 text-red-600 dark:text-red-400" />
					</div>
					<h2 class="text-2xl font-semibold text-gray-900 dark:text-gray-100 mb-2">
						Connection Failed
					</h2>
					<p class="text-gray-600 dark:text-gray-400 mb-4">
						{errorMessage || 'An error occurred during OAuth authorization.'}
					</p>
					<p class="text-sm text-gray-500 dark:text-gray-500">Redirecting to settings...</p>
				</div>
			{/if}
		</div>
	</div>
</div>
