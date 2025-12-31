<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { auth, isAuthenticated } from '$lib/stores/auth';

	let loading = $state(false);

	// Check if already authenticated and redirect
	onMount(() => {
		const unsubscribe = isAuthenticated.subscribe((authenticated) => {
			if (authenticated) {
				goto('/');
			}
		});
		return unsubscribe;
	});

	function handleLogin() {
		loading = true;
		// Redirect to SSO login, return to admin dashboard after
		auth.login(window.location.origin);
	}
</script>

<svelte:head>
	<title>Login - Search Engine Admin</title>
</svelte:head>

<div class="min-h-screen bg-gradient-to-br from-blue-50 via-white to-purple-50 flex items-center justify-center p-4">
	<div class="w-full max-w-md">
		<!-- Logo/Header -->
		<div class="text-center mb-8">
			<div class="inline-block p-3 bg-blue-600 rounded-2xl mb-4">
				<svg class="w-8 h-8 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
				</svg>
			</div>
			<h1 class="text-3xl font-bold text-gray-900">Welcome back</h1>
			<p class="text-gray-600 mt-2">Sign in to your admin account</p>
		</div>

		<!-- Login Card -->
		<div class="bg-white rounded-2xl shadow-xl p-8">
			<div class="space-y-6">
				<!-- SSO Login Button -->
				<button
					type="button"
					onclick={handleLogin}
					disabled={loading}
					class="w-full bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 text-white font-medium py-3 px-4 rounded-lg transition-colors duration-200 flex items-center justify-center"
				>
					{#if loading}
						<svg class="animate-spin h-5 w-5 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
							<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
							<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
						</svg>
						<span class="ml-2">Redirecting...</span>
					{:else}
						Sign in with Arack SSO
					{/if}
				</button>

				<div class="relative">
					<div class="absolute inset-0 flex items-center">
						<div class="w-full border-t border-gray-200"></div>
					</div>
					<div class="relative flex justify-center text-xs uppercase">
						<span class="bg-white px-2 text-gray-500">
							Single Sign-On
						</span>
					</div>
				</div>

				<p class="text-center text-sm text-gray-600">
					Use your Arack account to access Search, Mail, and Admin
				</p>
			</div>
		</div>

		<!-- Footer Text -->
		<div class="text-center mt-6 text-sm text-gray-600">
			<p>Don't have an account? Contact your administrator.</p>
		</div>
	</div>
</div>
