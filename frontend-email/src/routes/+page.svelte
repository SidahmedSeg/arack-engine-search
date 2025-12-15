<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';

	onMount(() => {
		// Redirect to inbox
		goto('/inbox');
	});
</script>

<div class="flex items-center justify-center h-screen">
	<p class="text-gray-500">Redirecting to inbox...</p>
</div>
