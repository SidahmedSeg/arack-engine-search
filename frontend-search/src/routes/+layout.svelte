<script lang="ts">
	import { onMount } from 'svelte';
	import '../app.css';
	import favicon from '$lib/assets/favicon.svg';
	import { authStore } from '$lib/stores/auth.svelte';
	import { preferencesStore } from '$lib/stores/preferences.svelte';

	let { children } = $props();

	// Phase 8.6: Check session and load preferences on app initialization
	onMount(async () => {
		// Check if user has an active Ory session
		await authStore.checkSession();

		// If authenticated, load user preferences
		if (authStore.isAuthenticated) {
			await preferencesStore.load();
		}
	});

	// Phase 8.6: Apply theme class to document based on preferences
	$effect(() => {
		const theme = preferencesStore.theme;
		document.documentElement.classList.toggle('dark', theme === 'dark');
	});
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

<div class="min-h-screen bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 transition-colors">
	{@render children()}
</div>
