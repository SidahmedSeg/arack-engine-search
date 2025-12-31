<script lang="ts">
	import { onMount } from 'svelte';
	import { ory } from '$lib/api/ory';
	import type { RecoveryFlow } from '@ory/client';
	import Button from '$lib/components/ui/button/button.svelte';
	import Input from '$lib/components/ui/input/input.svelte';
	import OTPInput from '$lib/components/ui/otp-input/otp-input.svelte';
	import * as Card from '$lib/components/ui/card';
	import { KeyRound, Mail, CheckCircle } from 'lucide-svelte';

	let flow: RecoveryFlow | null = $state(null);
	let email = $state('');
	let code = $state('');
	let password = $state('');
	let csrfToken = $state('');
	let isLoading = $state(false);
	let error = $state('');
	let success = $state(false);
	let currentStep: 'email' | 'code' | 'success' = $state('email');

	onMount(async () => {
		await initializeFlow();
	});

	async function initializeFlow() {
		const urlParams = new URLSearchParams(window.location.search);
		const flowId = urlParams.get('flow');

		try {
			if (flowId) {
				const { data } = await ory.getRecoveryFlow({ id: flowId });
				flow = data;
			} else {
				const { data } = await ory.createBrowserRecoveryFlow();
				flow = data;
				window.history.replaceState({}, '', `/auth/recovery?flow=${data.id}`);
			}

			// Extract CSRF token
			const csrfNode = flow?.ui.nodes.find(
				(node) => 'name' in node.attributes && node.attributes.name === 'csrf_token'
			);
			if (csrfNode && 'value' in csrfNode.attributes) {
				csrfToken = csrfNode.attributes.value as string;
			}

			// Determine current step based on flow nodes
			const hasCodeField = flow?.ui.nodes.some(
				(node) => 'name' in node.attributes && node.attributes.name === 'code'
			);
			const hasPasswordField = flow?.ui.nodes.some(
				(node) => 'name' in node.attributes && node.attributes.name === 'password'
			);

			if (hasCodeField && hasPasswordField) {
				currentStep = 'code';
				// Extract email from flow if available
				const emailNode = flow?.ui.nodes.find(
					(node) => 'name' in node.attributes && node.attributes.name === 'email'
				);
				if (emailNode && 'value' in emailNode.attributes) {
					email = emailNode.attributes.value as string;
				}
			} else {
				currentStep = 'email';
			}
		} catch (err: any) {
			console.error('Failed to initialize recovery flow:', err);
			error = 'Failed to initialize password recovery. Please try again.';
		}
	}

	async function handleEmailSubmit() {
		if (!flow || !email) return;

		isLoading = true;
		error = '';

		try {
			const { data } = await ory.updateRecoveryFlow({
				flow: flow.id,
				updateRecoveryFlowBody: {
					method: 'code',
					csrf_token: csrfToken,
					email: email
				}
			});

			// Update flow with response
			flow = data;

			// Extract new CSRF token
			const csrfNode = flow?.ui.nodes.find(
				(node) => 'name' in node.attributes && node.attributes.name === 'csrf_token'
			);
			if (csrfNode && 'value' in csrfNode.attributes) {
				csrfToken = csrfNode.attributes.value as string;
			}

			// Transition to code entry step
			currentStep = 'code';
			isLoading = false;
		} catch (err: any) {
			isLoading = false;

			if (err.response?.data?.ui) {
				const messages = err.response.data.ui.messages || [];
				error = messages.map((m: any) => m.text).join('. ') || 'Failed to send recovery code.';
			} else {
				error = 'Failed to send recovery email. Please try again.';
			}
		}
	}

	async function handleCodeSubmit() {
		if (!flow || code.length !== 6 || !password) return;

		isLoading = true;
		error = '';

		try {
			await ory.updateRecoveryFlow({
				flow: flow.id,
				updateRecoveryFlowBody: {
					method: 'code',
					csrf_token: csrfToken,
					code: code,
					password: password
				}
			});

			success = true;
			setTimeout(() => {
				window.location.href = '/auth/login';
			}, 2000);
		} catch (err: any) {
			// Check if it's a redirect response (expected after successful password reset)
			if (err.response?.data?.redirect_browser_to) {
				success = true;
				setTimeout(() => {
					window.location.href = '/auth/login';
				}, 2000);
				return;
			}

			isLoading = false;

			if (err.response?.data?.ui) {
				const messages = err.response.data.ui.messages || [];
				error = messages.map((m: any) => m.text).join('. ') || 'Password reset failed.';
			} else {
				error = 'Failed to reset password. Please check your code.';
			}
		}
	}

	function handleOTPChange(value: string) {
		code = value;
	}
</script>

<div class="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900 px-4">
	<Card.Root class="w-full max-w-md">
		<Card.Content class="py-8">
			{#if success}
				<!-- Success State -->
				<div class="text-center">
					<CheckCircle size={64} class="mx-auto text-green-600 mb-4" />
					<h1 class="text-2xl font-bold text-gray-900 dark:text-white mb-2">Password Reset!</h1>
					<p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
						Your password has been successfully reset. Redirecting to login...
					</p>
				</div>
			{:else if currentStep === 'email'}
				<!-- Step 1: Request Recovery Code -->
				<div class="text-center mb-6">
					<KeyRound size={48} class="mx-auto text-primary mb-3" />
					<h1 class="text-2xl font-bold text-gray-900 dark:text-white mb-2">Reset Password</h1>
					<p class="text-sm text-gray-600 dark:text-gray-400">
						Enter your email and we'll send you a code to reset your password
					</p>
				</div>

				{#if error}
					<div class="mb-4 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md">
						<p class="text-sm text-red-600 dark:text-red-400">{error}</p>
					</div>
				{/if}

				<form onsubmit={(e) => { e.preventDefault(); handleEmailSubmit(); }} class="space-y-4">
					<Input
						type="email"
						label="Email"
						bind:value={email}
						placeholder="you@example.com"
						required
					/>

					<Button type="submit" variant="default" class="w-full" disabled={isLoading}>
						{isLoading ? 'Sending...' : 'Send Recovery Code'}
					</Button>
				</form>

				<div class="mt-6 text-center text-sm">
					<a href="/auth/login" class="text-primary hover:underline font-medium">
						← Back to login
					</a>
				</div>
			{:else if currentStep === 'code'}
				<!-- Step 2: Enter Code & New Password -->
				<div class="text-center mb-8">
					<Mail size={48} class="mx-auto text-primary mb-4" />
					<h1 class="text-2xl font-bold text-gray-900 dark:text-white mb-3">Enter Recovery Code</h1>
					<p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
						We've sent a 6-digit code to
					</p>
					{#if email}
						<p class="text-base font-semibold text-gray-900 dark:text-white mb-2">
							{email}
						</p>
					{/if}
					<p class="text-xs text-gray-500 dark:text-gray-400">
						Enter the code and choose a new password
					</p>
				</div>

				{#if error}
					<div class="mb-6 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md">
						<p class="text-sm text-red-600 dark:text-red-400 text-center">{error}</p>
					</div>
				{/if}

				<form onsubmit={(e) => { e.preventDefault(); handleCodeSubmit(); }} class="space-y-6">
					<div>
						<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-3 text-center">
							Recovery Code
						</label>
						<OTPInput
							bind:value={code}
							length={6}
							disabled={isLoading}
							oninput={handleOTPChange}
						/>
					</div>

					<Input
						type="password"
						label="New Password"
						bind:value={password}
						placeholder="Enter new password"
						required
					/>

					<p class="text-xs text-gray-500 dark:text-gray-400">
						Password must be at least 8 characters long
					</p>

					<Button type="submit" variant="default" class="w-full" disabled={isLoading || code.length !== 6 || !password}>
						{isLoading ? 'Resetting Password...' : 'Reset Password'}
					</Button>
				</form>

				<div class="mt-6 text-center text-sm">
					<span class="text-gray-600 dark:text-gray-400">Didn't receive the code?</span>
					<button
						class="ml-1 text-primary hover:underline font-medium"
						onclick={() => { currentStep = 'email'; initializeFlow(); }}
					>
						Try again
					</button>
				</div>
			{/if}

			<div class="mt-4 text-center">
				<a href="/" class="text-sm text-gray-600 hover:text-gray-700 dark:text-gray-400">
					← Back to home
				</a>
			</div>
		</Card.Content>
	</Card.Root>
</div>
