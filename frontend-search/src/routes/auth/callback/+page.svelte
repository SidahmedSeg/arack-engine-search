<script lang="ts">
	import { onMount } from 'svelte';
	import { authStore } from '$lib/stores/auth.svelte';
	import * as Card from '$lib/components/ui/card';

	let isLoading = $state(true);
	let error = $state('');

	onMount(async () => {
		try {
			// Check if user is authenticated
			await authStore.checkSession();

			if (authStore.isAuthenticated) {
				// Redirect to home search page
				window.location.href = '/';
			} else {
				error = 'Authentication failed. Please try logging in again.';
				setTimeout(() => {
					window.location.href = '/auth/login';
				}, 2000);
			}
		} catch (err: any) {
			console.error('Callback error:', err);
			error = 'An error occurred. Redirecting to login...';
			setTimeout(() => {
				window.location.href = '/auth/login';
			}, 2000);
		}
	});
</script>

<div class="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900 px-4">
	<Card.Root class="w-full max-w-md">
		<Card.Content class="py-8">
			{#if error}
				<div class="text-center">
					<p class="text-red-600 dark:text-red-400">{error}</p>
				</div>
			{:else}
				<div class="text-center">
					<div class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-primary mb-4"></div>
					<h1 class="text-xl font-semibold text-gray-900 dark:text-white mb-2">
						Completing authentication...
					</h1>
					<p class="text-sm text-gray-600 dark:text-gray-400">
						Please wait while we log you in
					</p>
				</div>
			{/if}
		</Card.Content>
	</Card.Root>
</div>
