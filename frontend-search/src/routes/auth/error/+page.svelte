<script lang="ts">
	import { onMount } from 'svelte';
	import * as Card from '$lib/components/ui/card';
	import Button from '$lib/components/ui/button/button.svelte';
	import { AlertCircle } from 'lucide-svelte';

	let errorMessage = $state('An authentication error occurred');
	let errorId = $state('');

	onMount(() => {
		const urlParams = new URLSearchParams(window.location.search);
		const error = urlParams.get('error');
		const id = urlParams.get('id');

		if (error) {
			errorMessage = decodeURIComponent(error);
		}
		if (id) {
			errorId = id;
		}
	});
</script>

<div class="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900 px-4">
	<Card.Root class="w-full max-w-md">
		<Card.Content class="py-8">
			<div class="text-center">
				<AlertCircle size={64} class="mx-auto text-red-600 mb-4" />
				<h1 class="text-2xl font-bold text-gray-900 dark:text-white mb-2">Authentication Error</h1>
				<p class="text-sm text-gray-600 dark:text-gray-400 mb-6">
					{errorMessage}
				</p>

				{#if errorId}
					<p class="text-xs text-gray-500 dark:text-gray-400 mb-6">
						Error ID: {errorId}
					</p>
				{/if}

				<div class="flex flex-col gap-2">
					<Button variant="default" onclick={() => window.location.href = '/auth/login'}>
						Try Login Again
					</Button>
					<Button variant="secondary" onclick={() => window.location.href = '/'}>
						Back to Home
					</Button>
				</div>
			</div>
		</Card.Content>
	</Card.Root>
</div>
