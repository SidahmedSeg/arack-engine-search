<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import { api } from '$lib/stores/api';
	import { auth } from '$lib/stores/auth';
	import type { InvitationVerifyResponse } from '$shared/types';

	let token = $derived($page.params.token);

	let invitation = $state<InvitationVerifyResponse | null>(null);
	let verifying = $state(true);
	let verifyError = $state('');

	let password = $state('');
	let confirmPassword = $state('');
	let firstName = $state('');
	let lastName = $state('');
	let error = $state('');
	let submitting = $state(false);

	onMount(async () => {
		try {
			invitation = await api.verifyInvitation(token);
			verifying = false;
		} catch (err: any) {
			verifyError = err.message || 'Invalid or expired invitation';
			verifying = false;
		}
	});

	async function handleSubmit(e: Event) {
		e.preventDefault();
		error = '';

		// Validate passwords match
		if (password !== confirmPassword) {
			error = 'Passwords do not match';
			return;
		}

		// Validate password length
		if (password.length < 8) {
			error = 'Password must be at least 8 characters';
			return;
		}

		submitting = true;

		const result = await auth.acceptInvitation(token, password, firstName, lastName);

		submitting = false;

		if (result.success) {
			goto('/');
		} else {
			error = result.error || 'Failed to accept invitation';
		}
	}
</script>

<svelte:head>
	<title>Accept Invitation - Search Engine Admin</title>
</svelte:head>

<div class="min-h-screen bg-gradient-to-br from-blue-50 via-white to-purple-50 flex items-center justify-center p-4">
	<div class="w-full max-w-md">
		<!-- Logo/Header -->
		<div class="text-center mb-8">
			<div class="inline-block p-3 bg-blue-600 rounded-2xl mb-4">
				<svg class="w-8 h-8 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 19v-8.93a2 2 0 01.89-1.664l7-4.666a2 2 0 012.22 0l7 4.666A2 2 0 0121 10.07V19M3 19a2 2 0 002 2h14a2 2 0 002-2M3 19l6.75-4.5M21 19l-6.75-4.5M3 10l6.75 4.5M21 10l-6.75 4.5m0 0l-1.14.76a2 2 0 01-2.22 0l-1.14-.76" />
				</svg>
			</div>
			<h1 class="text-3xl font-bold text-gray-900">You're invited!</h1>
			<p class="text-gray-600 mt-2">Complete your account setup to get started</p>
		</div>

		<!-- Content Card -->
		<div class="bg-white rounded-2xl shadow-xl p-8">
			{#if verifying}
				<!-- Loading State -->
				<div class="text-center py-12">
					<svg class="animate-spin h-10 w-10 text-blue-600 mx-auto mb-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
						<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
						<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
					</svg>
					<p class="text-gray-600">Verifying invitation...</p>
				</div>
			{:else if verifyError}
				<!-- Error State -->
				<div class="text-center py-12">
					<div class="inline-block p-3 bg-red-100 rounded-2xl mb-4">
						<svg class="w-8 h-8 text-red-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
						</svg>
					</div>
					<h2 class="text-xl font-semibold text-gray-900 mb-2">Invalid Invitation</h2>
					<p class="text-gray-600 mb-6">{verifyError}</p>
					<a
						href="/login"
						class="inline-block bg-blue-600 hover:bg-blue-700 text-white font-medium py-2 px-6 rounded-lg transition-colors"
					>
						Go to Login
					</a>
				</div>
			{:else if invitation}
				<!-- Invitation Info -->
				<div class="mb-6 p-4 bg-blue-50 border border-blue-200 rounded-lg">
					<div class="flex items-start">
						<svg class="w-5 h-5 text-blue-600 mt-0.5 mr-3" fill="currentColor" viewBox="0 0 20 20">
							<path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd" />
						</svg>
						<div>
							<p class="text-sm text-blue-900 font-medium">Creating account for</p>
							<p class="text-sm text-blue-700">{invitation.email}</p>
							<p class="text-xs text-blue-600 mt-1">Role: {invitation.role}</p>
						</div>
					</div>
				</div>

				<!-- Registration Form -->
				<form onsubmit={handleSubmit} class="space-y-5">
					<!-- First Name -->
					<div>
						<label for="firstName" class="block text-sm font-medium text-gray-700 mb-2">
							First Name
						</label>
						<input
							type="text"
							id="firstName"
							bind:value={firstName}
							required
							autocomplete="given-name"
							class="w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all outline-none"
							placeholder="John"
							disabled={submitting}
						/>
					</div>

					<!-- Last Name -->
					<div>
						<label for="lastName" class="block text-sm font-medium text-gray-700 mb-2">
							Last Name
						</label>
						<input
							type="text"
							id="lastName"
							bind:value={lastName}
							required
							autocomplete="family-name"
							class="w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all outline-none"
							placeholder="Doe"
							disabled={submitting}
						/>
					</div>

					<!-- Password -->
					<div>
						<label for="password" class="block text-sm font-medium text-gray-700 mb-2">
							Password
						</label>
						<input
							type="password"
							id="password"
							bind:value={password}
							required
							autocomplete="new-password"
							class="w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all outline-none"
							placeholder="At least 8 characters"
							disabled={submitting}
						/>
					</div>

					<!-- Confirm Password -->
					<div>
						<label for="confirmPassword" class="block text-sm font-medium text-gray-700 mb-2">
							Confirm Password
						</label>
						<input
							type="password"
							id="confirmPassword"
							bind:value={confirmPassword}
							required
							autocomplete="new-password"
							class="w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all outline-none"
							placeholder="Repeat your password"
							disabled={submitting}
						/>
					</div>

					<!-- Error Message -->
					{#if error}
						<div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg text-sm">
							{error}
						</div>
					{/if}

					<!-- Submit Button -->
					<button
						type="submit"
						disabled={submitting}
						class="w-full bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 text-white font-medium py-3 px-4 rounded-lg transition-colors duration-200 flex items-center justify-center"
					>
						{#if submitting}
							<svg class="animate-spin h-5 w-5 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
								<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
								<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
							</svg>
							<span class="ml-2">Creating account...</span>
						{:else}
							Create Account
						{/if}
					</button>
				</form>
			{/if}
		</div>

		<!-- Footer Text -->
		<div class="text-center mt-6 text-sm text-gray-600">
			<p>Already have an account? <a href="/login" class="text-blue-600 hover:text-blue-700 font-medium">Sign in</a></p>
		</div>
	</div>
</div>
