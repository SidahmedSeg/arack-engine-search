<script lang="ts">
	import { authStore } from '$lib/stores/auth.svelte';
	import Button from '$lib/components/ui/button/button.svelte';
	import * as Card from '$lib/components/ui/card';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';

	let isLoading = $state(false);
	let error = $state('');

	// Check if already authenticated and redirect
	onMount(async () => {
		if (authStore.isAuthenticated) {
			goto('/');
		}
	});

	function handleSignup() {
		isLoading = true;
		error = '';

		// Redirect to account.arack.io SSO login (registration is handled there)
		authStore.login(window.location.origin);
	}
</script>

<div class="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900 px-4">
	<Card.Root class="w-full max-w-md">
		<Card.Content class="py-8">
			<div class="text-center mb-8">
				<h1 class="text-2xl font-bold text-gray-900 dark:text-white mb-2">Join Arack</h1>
				<p class="text-sm text-gray-600 dark:text-gray-400">
					Create an account to access Search, Mail, Drive, and all Arack apps
				</p>
			</div>

			{#if error}
				<div
					class="mb-6 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md"
				>
					<p class="text-sm text-red-600 dark:text-red-400">{error}</p>
				</div>
			{/if}

			<div class="space-y-4">
				<Button
					type="button"
					variant="default"
					class="w-full"
					disabled={isLoading}
					onclick={handleSignup}
				>
					{#if isLoading}
						<span class="flex items-center justify-center gap-2">
							<div class="inline-block animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
							Redirecting...
						</span>
					{:else}
						Create Arack Account
					{/if}
				</Button>

				<div class="relative">
					<div class="absolute inset-0 flex items-center">
						<div class="w-full border-t border-gray-300 dark:border-gray-600"></div>
					</div>
					<div class="relative flex justify-center text-xs uppercase">
						<span class="bg-gray-50 dark:bg-gray-900 px-2 text-gray-500">
							One account for everything
						</span>
					</div>
				</div>

				<div class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-md p-4">
					<p class="text-sm text-blue-800 dark:text-blue-200">
						Your Arack account gives you access to:
					</p>
					<ul class="mt-2 space-y-1 text-xs text-blue-700 dark:text-blue-300">
						<li>• Arack Search - Save searches and view history</li>
						<li>• Arack Mail - Professional email (@arack.io)</li>
						<li>• Arack Drive - Cloud storage and collaboration</li>
						<li>• Future apps - One login for all services</li>
					</ul>
				</div>
			</div>

			<div class="mt-6 text-center text-sm">
				<span class="text-gray-600 dark:text-gray-400">Already have an account?</span>
				<a href="/auth/login" class="ml-1 text-primary hover:underline font-medium"> Sign in </a>
			</div>

			<div class="mt-4 text-center">
				<a href="/" class="text-sm text-gray-600 hover:text-gray-700 dark:text-gray-400">
					← Back to home
				</a>
			</div>
		</Card.Content>
	</Card.Root>
</div>
