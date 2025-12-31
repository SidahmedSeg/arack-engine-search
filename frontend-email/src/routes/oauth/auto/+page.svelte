<script lang="ts">
	import { onMount } from 'svelte';
	import { Loader2, Mail } from 'lucide-svelte';
	import { login } from '$lib/auth/oauth';

	let status = $state<'checking' | 'redirecting'>('checking');

	onMount(() => {
		// Wait a moment to show the loading screen, then redirect to Zitadel
		setTimeout(async () => {
			status = 'redirecting';
			// Redirect to Zitadel OAuth login
			await login();
		}, 800);
	});
</script>

<svelte:head>
	<title>Connecting Email - Arack Mail</title>
</svelte:head>

<div
	class="min-h-screen flex items-center justify-center bg-gradient-to-br from-blue-50 to-indigo-100 dark:from-gray-900 dark:to-gray-800 p-4"
>
	<div class="max-w-md w-full">
		<div class="bg-white dark:bg-gray-800 rounded-2xl shadow-2xl p-8 text-center">
			<!-- Logo/Icon -->
			<div class="mb-6 flex justify-center">
				<div
					class="w-20 h-20 rounded-full bg-gradient-to-br from-blue-500 to-indigo-600 flex items-center justify-center shadow-lg"
				>
					<Mail class="h-10 w-10 text-white" />
				</div>
			</div>

			{#if status === 'checking'}
				<!-- Checking status -->
				<div class="space-y-4">
					<Loader2 class="h-12 w-12 text-blue-600 dark:text-blue-400 mx-auto animate-spin" />
					<h2 class="text-2xl font-bold text-gray-900 dark:text-gray-100">
						Setting up your email...
					</h2>
					<p class="text-gray-600 dark:text-gray-400">Preparing your secure connection</p>
				</div>
			{:else if status === 'redirecting'}
				<!-- Redirecting to OAuth -->
				<div class="space-y-4">
					<div class="relative">
						<Loader2 class="h-12 w-12 text-blue-600 dark:text-blue-400 mx-auto animate-spin" />
						<div class="absolute inset-0 flex items-center justify-center">
							<div
								class="w-16 h-16 border-4 border-blue-200 dark:border-blue-800 rounded-full animate-pulse"
							></div>
						</div>
					</div>
					<h2 class="text-2xl font-bold text-gray-900 dark:text-gray-100">
						Connecting your account...
					</h2>
					<p class="text-gray-600 dark:text-gray-400">
						You'll be redirected to sign in with your Arack account
					</p>
					<div class="mt-6 pt-6 border-t border-gray-200 dark:border-gray-700">
						<div class="space-y-2 text-sm text-gray-500 dark:text-gray-400">
							<p class="flex items-center justify-center gap-2">
								<svg class="w-4 h-4 text-green-500" fill="currentColor" viewBox="0 0 20 20">
									<path
										fill-rule="evenodd"
										d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
										clip-rule="evenodd"
									/>
								</svg>
								Single Sign-On with Arack Search
							</p>
							<p class="flex items-center justify-center gap-2">
								<svg class="w-4 h-4 text-green-500" fill="currentColor" viewBox="0 0 20 20">
									<path
										fill-rule="evenodd"
										d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
										clip-rule="evenodd"
									/>
								</svg>
								Secure OAuth 2.0 with PKCE
							</p>
							<p class="flex items-center justify-center gap-2">
								<svg class="w-4 h-4 text-green-500" fill="currentColor" viewBox="0 0 20 20">
									<path
										fill-rule="evenodd"
										d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
										clip-rule="evenodd"
									/>
								</svg>
								Automatic token refresh
							</p>
						</div>
					</div>
				</div>
			{/if}
		</div>
	</div>
</div>

<style>
	@keyframes pulse {
		0%,
		100% {
			opacity: 1;
			transform: scale(1);
		}
		50% {
			opacity: 0.5;
			transform: scale(1.05);
		}
	}

	.animate-pulse {
		animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
	}
</style>
