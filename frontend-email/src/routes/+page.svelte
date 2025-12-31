<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { authStore } from '$lib/stores/auth.svelte';
	import { login, isAuthenticated } from '$lib/auth/oauth';

	onMount(async () => {
		// Check if user has valid OAuth session
		if (isAuthenticated()) {
			// Check session and validate tokens
			await authStore.checkSession();

			if (authStore.isAuthenticated) {
				// Session is valid, redirect to inbox
				goto('/inbox');
				return;
			}
		}

		// No valid session, redirect to Zitadel login
		console.log('[Root] No valid session, redirecting to Zitadel login');
		await login();
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
