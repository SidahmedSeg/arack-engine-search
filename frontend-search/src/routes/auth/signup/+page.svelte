<script lang="ts">
	import { onMount } from 'svelte';
	import { ory } from '$lib/stores/auth.svelte';
	import type { RegistrationFlow } from '@ory/client';
	import Button from '$lib/components/ui/button/button.svelte';
	import Input from '$lib/components/ui/input/input.svelte';
	import * as Card from '$lib/components/ui/card';

	let flow: RegistrationFlow | null = $state(null);
	let email = $state('');
	let password = $state('');
	let firstName = $state('');
	let lastName = $state('');
	let csrfToken = $state('');
	let actionUrl = $state('');
	let isLoading = $state(false);
	let error = $state('');

	onMount(async () => {
		const urlParams = new URLSearchParams(window.location.search);
		const flowId = urlParams.get('flow');

		try {
			if (flowId) {
				const { data } = await ory.getRegistrationFlow({ id: flowId });
				flow = data;
				actionUrl = data.ui.action;
			} else {
				// Redirect to Kratos to create flow
				window.location.href = 'http://127.0.0.1:4433/self-service/registration/browser';
				return;
			}

			// Extract CSRF token
			const csrfNode = flow?.ui.nodes.find(
				(node) => 'name' in node.attributes && node.attributes.name === 'csrf_token'
			);
			if (csrfNode && 'value' in csrfNode.attributes) {
				csrfToken = csrfNode.attributes.value as string;
			}
		} catch (err: any) {
			console.error('Failed to initialize registration flow:', err);
			error = 'Failed to initialize registration. Please try again.';
		}
	});
</script>

<div class="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900 px-4">
	<Card.Root class="w-full max-w-md">
		<Card.Content class="py-8">
			<div class="text-center mb-6">
				<h1 class="text-2xl font-bold text-gray-900 dark:text-white mb-2">Create Account</h1>
				<p class="text-sm text-gray-600 dark:text-gray-400">
					Sign up to save searches and track your history
				</p>
			</div>

			{#if error}
				<div class="mb-4 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md">
					<p class="text-sm text-red-600 dark:text-red-400">{error}</p>
				</div>
			{/if}

			{#if flow}
				<!-- Native HTML form that submits directly to Kratos -->
				<form method="POST" action={actionUrl} class="space-y-4">
					<!-- Hidden CSRF token -->
					<input type="hidden" name="csrf_token" value={csrfToken} />
					<input type="hidden" name="method" value="password" />

					<Input
						type="text"
						label="First Name"
						bind:value={firstName}
						placeholder="John"
						required
						name="traits.first_name"
					/>

					<Input
						type="text"
						label="Last Name"
						bind:value={lastName}
						placeholder="Doe"
						required
						name="traits.last_name"
					/>

					<Input
						type="email"
						label="Email"
						bind:value={email}
						placeholder="you@example.com"
						required
						name="traits.email"
					/>

					<Input
						type="password"
						label="Password"
						bind:value={password}
						placeholder="••••••••"
						required
						name="password"
					/>

					<p class="text-xs text-gray-500 dark:text-gray-400">
						Password must be at least 8 characters long
					</p>

					<Button type="submit" variant="default" class="w-full" disabled={isLoading}>
						{isLoading ? 'Creating account...' : 'Sign Up'}
					</Button>
				</form>
			{:else}
				<div class="text-center py-8">
					<div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
					<p class="mt-2 text-sm text-gray-600 dark:text-gray-400">Initializing...</p>
				</div>
			{/if}

			<div class="mt-6 text-center text-sm">
				<span class="text-gray-600 dark:text-gray-400">Already have an account?</span>
				<a href="/auth/login" class="ml-1 text-primary hover:underline font-medium">
					Sign in
				</a>
			</div>

			<div class="mt-4 text-center">
				<a href="/" class="text-sm text-gray-600 hover:text-gray-700 dark:text-gray-400">
					← Back to home
				</a>
			</div>
		</Card.Content>
	</Card.Root>
</div>
