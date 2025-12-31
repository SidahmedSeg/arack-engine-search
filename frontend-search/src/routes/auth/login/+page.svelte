<script lang="ts">
	import { authStore } from '$lib/stores/auth.svelte';
	import Button from '$lib/components/ui/button/button.svelte';
	import Input from '$lib/components/ui/input/input.svelte';
	import Label from '$lib/components/ui/label/label.svelte';
	import * as Card from '$lib/components/ui/card';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';

	let email = $state('');
	let password = $state('');
	let isLoading = $state(false);
	let error = $state('');

	// Check if already authenticated and redirect
	onMount(async () => {
		if (authStore.isAuthenticated) {
			goto('/');
		}
	});

	async function handleLogin(e: Event) {
		e.preventDefault();
		isLoading = true;
		error = '';

		try {
			await authStore.loginWithPassword(email, password);
			// Redirect to home on success
			goto('/');
		} catch (err: any) {
			error = err.message || 'Invalid email or password';
		} finally {
			isLoading = false;
		}
	}
</script>

<div class="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900 px-4">
	<Card.Root class="w-full max-w-md">
		<Card.Content class="py-8">
			<div class="text-center mb-8">
				<h1 class="text-2xl font-bold text-gray-900 dark:text-white mb-2">Welcome Back</h1>
				<p class="text-sm text-gray-600 dark:text-gray-400">
					Sign in to access Arack Search, Mail, Drive, and all your apps
				</p>
			</div>

			{#if error}
				<div
					class="mb-6 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md"
				>
					<p class="text-sm text-red-600 dark:text-red-400">{error}</p>
				</div>
			{/if}

			<form onsubmit={handleLogin} class="space-y-4">
				<div class="space-y-2">
					<Label for="email">Email</Label>
					<Input
						id="email"
						type="email"
						placeholder="you@arack.io"
						bind:value={email}
						required
						disabled={isLoading}
					/>
				</div>

				<div class="space-y-2">
					<div class="flex items-center justify-between">
						<Label for="password">Password</Label>
						<a
							href="/auth/recovery"
							class="text-xs text-primary hover:underline"
						>
							Forgot password?
						</a>
					</div>
					<Input
						id="password"
						type="password"
						placeholder="Enter your password"
						bind:value={password}
						required
						disabled={isLoading}
					/>
				</div>

				<Button
					type="submit"
					variant="default"
					class="w-full"
					disabled={isLoading || !email || !password}
				>
					{#if isLoading}
						<span class="flex items-center justify-center gap-2">
							<div class="inline-block animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
							Signing in...
						</span>
					{:else}
						Sign In
					{/if}
				</Button>
			</form>

			<div class="mt-6 text-center text-sm">
				<span class="text-gray-600 dark:text-gray-400">Don't have an account?</span>
				<a href="/auth/register" class="ml-1 text-primary hover:underline font-medium">
					Create account
				</a>
			</div>

			<div class="mt-4 text-center">
				<a href="/" class="text-sm text-gray-600 hover:text-gray-700 dark:text-gray-400">
					Back to home
				</a>
			</div>
		</Card.Content>
	</Card.Root>
</div>
