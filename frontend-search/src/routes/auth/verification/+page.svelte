<script lang="ts">
	import { onMount } from 'svelte';
	import { ory } from '$lib/api/ory';
	import type { VerificationFlow } from '@ory/client';
	import Button from '$lib/components/ui/button/button.svelte';
	import OTPInput from '$lib/components/ui/otp-input/otp-input.svelte';
	import * as Card from '$lib/components/ui/card';
	import { Mail, CheckCircle } from 'lucide-svelte';

	let flow: VerificationFlow | null = $state(null);
	let userEmail = $state('');
	let code = $state('');
	let csrfToken = $state('');
	let isLoading = $state(false);
	let error = $state('');
	let success = $state(false);

	onMount(async () => {
		const urlParams = new URLSearchParams(window.location.search);
		const flowId = urlParams.get('flow');
		const codeParam = urlParams.get('code');

		if (codeParam) {
			code = codeParam;
		}

		try {
			if (flowId) {
				const { data } = await ory.getVerificationFlow({ id: flowId });
				flow = data;

				// Extract email from flow
				const emailNode = flow?.ui.nodes.find(
					(node) => 'name' in node.attributes && node.attributes.name === 'email'
				);
				if (emailNode && 'value' in emailNode.attributes) {
					userEmail = emailNode.attributes.value as string;
				}
			} else {
				const { data } = await ory.createBrowserVerificationFlow();
				flow = data;
				window.history.replaceState({}, '', `/auth/verification?flow=${data.id}`);
			}

			const csrfNode = flow?.ui.nodes.find(
				(node) => 'name' in node.attributes && node.attributes.name === 'csrf_token'
			);
			if (csrfNode && 'value' in csrfNode.attributes) {
				csrfToken = csrfNode.attributes.value as string;
			}
		} catch (err: any) {
			console.error('Failed to initialize verification flow:', err);
			error = 'Failed to initialize verification. Please try again.';
		}
	});

	async function handleSubmit() {
		if (!flow || code.length !== 6) return;

		isLoading = true;
		error = '';

		try {
			await ory.updateVerificationFlow({
				flow: flow.id,
				updateVerificationFlowBody: {
					method: 'code',
					csrf_token: csrfToken,
					email: userEmail,
					code: code
				}
			});

			success = true;
			setTimeout(() => {
				window.location.href = '/auth/login';
			}, 2000);
		} catch (err: any) {
			isLoading = false;

			if (err.response?.data?.ui) {
				const messages = err.response.data.ui.messages || [];
				error = messages.map((m: any) => m.text).join('. ') || 'Verification failed.';
			} else {
				error = 'Verification failed. Please check your code.';
			}
		}
	}

	// Auto-submit when all 6 digits are entered
	function handleOTPChange(value: string) {
		code = value;
		if (value.length === 6) {
			handleSubmit();
		}
	}
</script>

<div class="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900 px-4">
	<Card.Root class="w-full max-w-md">
		<Card.Content class="py-8">
			{#if success}
				<div class="text-center">
					<CheckCircle size={64} class="mx-auto text-green-600 mb-4" />
					<h1 class="text-2xl font-bold text-gray-900 dark:text-white mb-2">Email Verified!</h1>
					<p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
						Your email has been successfully verified. Redirecting to login...
					</p>
				</div>
			{:else}
				<div class="text-center mb-8">
					<Mail size={48} class="mx-auto text-primary mb-4" />
					<h1 class="text-2xl font-bold text-gray-900 dark:text-white mb-3">Verify Your Email</h1>
					<p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
						We've sent a 6-digit verification code to
					</p>
					{#if userEmail}
						<p class="text-base font-semibold text-gray-900 dark:text-white mb-2">
							{userEmail}
						</p>
					{/if}
					<p class="text-xs text-gray-500 dark:text-gray-400">
						Enter the code below to activate your account
					</p>
				</div>

				{#if error}
					<div class="mb-6 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md">
						<p class="text-sm text-red-600 dark:text-red-400 text-center">{error}</p>
					</div>
				{/if}

				<form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }} class="space-y-6">
					<div>
						<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-3 text-center">
							Verification Code
						</label>
						<OTPInput
							bind:value={code}
							length={6}
							disabled={isLoading}
							oninput={handleOTPChange}
						/>
					</div>

					<Button type="submit" variant="default" class="w-full" disabled={isLoading || code.length !== 6}>
						{isLoading ? 'Verifying...' : 'Verify Email'}
					</Button>
				</form>

				<div class="mt-6 text-center text-sm">
					<span class="text-gray-600 dark:text-gray-400">Didn't receive the code?</span>
					<button class="ml-1 text-primary hover:underline font-medium">
						Resend
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
