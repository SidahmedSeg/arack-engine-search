<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { emailAPI } from '$lib/api/client';

	onMount(async () => {
		// Check OAuth status first
		try {
			const oauthStatus = await emailAPI.getOAuthStatus();
			if (!oauthStatus.connected) {
				goto('/oauth/auto');
				return;
			}
		} catch (err) {
			// If OAuth check fails, assume not connected
			goto('/oauth/auto');
			return;
		}

		// OAuth is connected, redirect to inbox
		goto('/inbox');
	});
</script>

<div class="flex items-center justify-center h-screen">
	<p class="text-gray-500">Checking authentication...</p>
</div>
