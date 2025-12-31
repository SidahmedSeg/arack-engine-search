<script lang="ts">
	import { onMount } from 'svelte';
	import { Loader2 } from 'lucide-svelte';

	let status = $state<'processing' | 'redirecting' | 'error'>('processing');
	let errorMessage = $state<string | null>(null);

	onMount(async () => {
		// Extract consent_challenge from URL
		const params = new URLSearchParams(window.location.search);
		const consentChallenge = params.get('consent_challenge');

		if (!consentChallenge) {
			status = 'error';
			errorMessage = 'Missing consent_challenge parameter';
			return;
		}

		try {
			// Call the backend consent endpoint
			const response = await fetch(
				`https://api.arack.io/api/hydra/consent?consent_challenge=${consentChallenge}`,
				{
					method: 'GET',
					credentials: 'include', // Send Kratos session cookie
					headers: {
						Accept: 'application/json' // Important: tells backend to return JSON, not HTTP redirect
					}
				}
			);

			if (!response.ok) {
				throw new Error(`HTTP ${response.status}: ${response.statusText}`);
			}

			const data = await response.json();

			// Extract redirect_to from the response
			const redirectTo = data.data?.redirect_to || data.redirect_to;

			if (!redirectTo) {
				throw new Error('No redirect_to URL in response');
			}

			// Follow the redirect
			status = 'redirecting';
			window.location.href = redirectTo;
		} catch (error) {
			status = 'error';
			errorMessage = error instanceof Error ? error.message : 'Unknown error';
			console.error('Consent error:', error);
		}
	});
</script>

<svelte:head>
	<title>OAuth Consent - Arack Mail</title>
</svelte:head>

<div class="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900 p-4">
	<div class="max-w-md w-full">
		<div class="bg-white dark:bg-gray-800 rounded-lg shadow-lg p-8">
			{#if status === 'processing' || status === 'redirecting'}
				<div class="text-center">
					<Loader2 class="h-16 w-16 text-blue-600 dark:text-blue-400 mx-auto mb-4 animate-spin" />
					<h2 class="text-2xl font-semibold text-gray-900 dark:text-gray-100 mb-2">
						{status === 'processing' ? 'Processing consent...' : 'Redirecting...'}
					</h2>
					<p class="text-gray-600 dark:text-gray-400">
						Please wait while we complete the authorization.
					</p>
				</div>
			{:else if status === 'error'}
				<div class="text-center">
					<div
						class="w-16 h-16 rounded-full bg-red-100 dark:bg-red-900 flex items-center justify-center mx-auto mb-4"
					>
						<svg
							class="h-10 w-10 text-red-600 dark:text-red-400"
							fill="none"
							viewBox="0 0 24 24"
							stroke="currentColor"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M6 18L18 6M6 6l12 12"
							/>
						</svg>
					</div>
					<h2 class="text-2xl font-semibold text-gray-900 dark:text-gray-100 mb-2">
						Authorization Failed
					</h2>
					<p class="text-gray-600 dark:text-gray-400 mb-4">
						{errorMessage || 'An error occurred during authorization.'}
					</p>
					<button
						onclick={() => (window.location.href = '/oauth/auto')}
						class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
					>
						Try Again
					</button>
				</div>
			{/if}
		</div>
	</div>
</div>
