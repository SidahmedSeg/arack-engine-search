<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { initLoginFlow, submitLogin, isFlowExpired } from '$lib/api/kratos';
	import { authStore } from '$lib/stores/auth.svelte';
	import Button from '$lib/components/ui/button/button.svelte';
	import Input from '$lib/components/ui/input/input.svelte';
	import * as Card from '$lib/components/ui/card';

	let flowId = $state('');
	let email = $state('');
	let password = $state('');
	let isLoading = $state(false);
	let error = $state('');
	let flowInitialized = $state(false);

	onMount(async () => {
		try {
			// Initialize login flow
			const flow = await initLoginFlow();
			flowId = flow.id;
			flowInitialized = true;
		} catch (err: any) {
			console.error('Failed to initialize login flow:', err);
			error = 'Failed to initialize login. Please try again.';
		}
	});

	async function handleSubmit(event: Event) {
		event.preventDefault();

		if (!flowId) {
			error = 'Login flow not initialized. Please refresh the page.';
			return;
		}

		if (!email || !password) {
			error = 'Please enter your email and password.';
			return;
		}

		isLoading = true;
		error = '';

		try {
			// Submit login credentials
			await submitLogin(flowId, {
				identifier: email,
				password: password
			});

			// Update auth store
			await authStore.setAuthenticated();

			// Redirect to home page
			goto('/');
		} catch (err: any) {
			console.error('Login failed:', err);
			error = err.message || 'Invalid email or password. Please try again.';

			// If flow expired, reinitialize
			if (err.message.includes('expired') || err.message.includes('not found')) {
				try {
					const flow = await initLoginFlow();
					flowId = flow.id;
					error = 'Login session expired. Please try again.';
				} catch (reinitErr) {
					error = 'Failed to refresh login. Please reload the page.';
				}
			}

			isLoading = false;
		}
	}
</script>

<div class="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900 px-4">
	<Card.Root class="w-full max-w-md">
		<Card.Content class="py-8">
			<div class="text-center mb-6">
				<h1 class="text-2xl font-bold text-gray-900 dark:text-white mb-2">Welcome Back</h1>
				<p class="text-sm text-gray-600 dark:text-gray-400">
					Sign in to access your search history and saved searches
				</p>
			</div>

			{#if error}
				<div
					class="mb-4 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md"
				>
					<p class="text-sm text-red-600 dark:text-red-400">{error}</p>
				</div>
			{/if}

			{#if flowInitialized}
				<form onsubmit={handleSubmit} class="space-y-4">
					<Input
						type="email"
						label="Email"
						bind:value={email}
						placeholder="you@example.com"
						required
						disabled={isLoading}
					/>

					<Input
						type="password"
						label="Password"
						bind:value={password}
						placeholder="••••••••"
						required
						disabled={isLoading}
					/>

					<div class="flex items-center justify-between text-sm">
						<a href="/auth/recovery" class="text-primary hover:underline">
							Forgot password?
						</a>
					</div>

					<Button type="submit" variant="default" class="w-full" disabled={isLoading}>
						{isLoading ? 'Signing in...' : 'Sign In'}
					</Button>
				</form>
			{:else}
				<div class="text-center py-8">
					<div
						class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-primary"
					></div>
					<p class="mt-2 text-sm text-gray-600 dark:text-gray-400">Initializing...</p>
				</div>
			{/if}

			<div class="mt-6 text-center text-sm">
				<span class="text-gray-600 dark:text-gray-400">Don't have an account?</span>
				<a href="/auth/signup" class="ml-1 text-primary hover:underline font-medium"> Sign up </a>
			</div>

			<div class="mt-4 text-center">
				<a href="/" class="text-sm text-gray-600 hover:text-gray-700 dark:text-gray-400">
					← Back to home
				</a>
			</div>
		</Card.Content>
	</Card.Root>
</div>
