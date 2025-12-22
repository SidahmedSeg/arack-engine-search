<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import Button from '$lib/components/ui/button/button.svelte';
	import axios from 'axios';

	const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'https://api.arack.io';

	let consentChallenge = $state('');
	let clientName = $state('');
	let requestedScope = $state<string[]>([]);
	let loading = $state(false);
	let error = $state('');
	let submitting = $state(false); // Prevent double submission

	onMount(async () => {
		consentChallenge = $page.url.searchParams.get('consent_challenge') || '';

		if (!consentChallenge) {
			error = 'Missing consent challenge';
			return;
		}

		try {
			// Get consent request details from backend
			const response = await axios.get(
				`${API_BASE_URL}/api/hydra/consent?consent_challenge=${consentChallenge}`,
				{
					withCredentials: true
				}
			);

			const data = response.data.data;

			if (data.redirect_to) {
				// Auto-accepted (user already consented before), redirect
				window.location.href = data.redirect_to;
			} else {
				// Show consent UI
				clientName = data.client_name;
				requestedScope = Array.isArray(data.requested_scope)
					? data.requested_scope
					: [];
			}
		} catch (err: any) {
			error = err.response?.data?.error || 'Failed to load consent request';
		}
	});

	async function handleAccept() {
		// Prevent double submission
		if (submitting) return;
		submitting = true;

		loading = true;
		error = '';

		try {
			const response = await axios.post(
				`${API_BASE_URL}/api/hydra/consent/accept`,
				{
					consent_challenge: consentChallenge
				},
				{
					withCredentials: true
				}
			);

			// Redirect to Hydra's redirect URL
			if (response.data.data.redirect_to) {
				window.location.href = response.data.data.redirect_to;
			}
		} catch (err: any) {
			error = err.response?.data?.error || 'Failed to grant consent';
			loading = false;
			submitting = false; // Allow retry on error
		}
	}

	async function handleReject() {
		loading = true;
		error = '';

		try {
			const response = await axios.post(
				`${API_BASE_URL}/api/hydra/consent/reject`,
				{
					consent_challenge: consentChallenge
				},
				{
					withCredentials: true
				}
			);

			// Redirect back to client
			if (response.data.data.redirect_to) {
				window.location.href = response.data.data.redirect_to;
			}
		} catch (err: any) {
			error = err.response?.data?.error || 'Failed to reject consent';
			loading = false;
		}
	}

	function getScopeDescription(scope: string): string {
		const descriptions: Record<string, string> = {
			openid: 'Verify your identity',
			profile: 'View your profile information (name)',
			email: 'View your email address',
			offline_access: 'Access your data while you\'re offline'
		};
		return descriptions[scope] || scope;
	}
</script>

<svelte:head>
	<title>Grant Access - 2arak Search</title>
</svelte:head>

<div class="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900 px-4 py-8">
	<div class="max-w-md w-full space-y-8 p-8 bg-white dark:bg-gray-800 rounded-lg shadow-lg">
		<div class="text-center">
			<h2 class="text-2xl font-bold text-gray-900 dark:text-white">Grant Access</h2>
			{#if clientName}
				<p class="mt-2 text-gray-600 dark:text-gray-300">
					<strong class="text-blue-600 dark:text-blue-400">{clientName}</strong> wants to access your 2arak account
				</p>
			{/if}
		</div>

		{#if error}
			<div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 text-red-800 dark:text-red-300 px-4 py-3 rounded">
				{error}
			</div>
		{/if}

		{#if requestedScope.length > 0}
			<div class="space-y-4">
				<div class="border dark:border-gray-700 rounded-lg p-4 bg-gray-50 dark:bg-gray-700">
					<h3 class="font-semibold text-gray-900 dark:text-white mb-3">
						This will allow {clientName || 'the application'} to:
					</h3>
					<ul class="space-y-2 text-sm text-gray-700 dark:text-gray-300">
						{#each requestedScope as scope}
							<li class="flex items-center gap-2">
								<svg
									class="w-5 h-5 text-green-500 flex-shrink-0"
									fill="currentColor"
									viewBox="0 0 20 20"
								>
									<path
										fill-rule="evenodd"
										d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
										clip-rule="evenodd"
									/>
								</svg>
								<span>{getScopeDescription(scope)}</span>
							</li>
						{/each}
					</ul>
				</div>

				<div class="flex gap-3">
					<Button
						variant="outline"
						onclick={handleReject}
						disabled={loading}
						class="flex-1"
					>
						Deny
					</Button>
					<Button
						onclick={handleAccept}
						disabled={loading}
						class="flex-1"
					>
						{loading ? 'Processing...' : 'Allow'}
					</Button>
				</div>

				<p class="text-xs text-gray-500 dark:text-gray-400 text-center">
					You can revoke access at any time from your account settings
				</p>
			</div>
		{:else if !error}
			<div class="text-center text-gray-600 dark:text-gray-300">
				<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto"></div>
				<p class="mt-2">Loading...</p>
			</div>
		{/if}
	</div>
</div>
