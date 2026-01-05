<script lang="ts">
	import { goto } from '$app/navigation';
	import { authStore } from '$lib/stores/auth.svelte';
	import { onMount } from 'svelte';

	const API_URL = import.meta.env.VITE_ACCOUNT_API_URL || 'https://account.arack.io';

	// Helper functions for API calls
	async function getEmailSuggestions(firstName: string, lastName: string): Promise<string[]> {
		const response = await fetch(
			`${API_URL}/api/register/suggestions?firstName=${encodeURIComponent(firstName)}&lastName=${encodeURIComponent(lastName)}`,
			{ credentials: 'include' }
		);
		if (!response.ok) throw new Error('Failed to get suggestions');
		const data = await response.json();
		return data.suggestions || [];
	}

	async function checkEmailAvailabilityAPI(email: string): Promise<{ available: boolean }> {
		const response = await fetch(`${API_URL}/api/register/check-email`, {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			credentials: 'include',
			body: JSON.stringify({ email })
		});
		if (!response.ok) throw new Error('Failed to check availability');
		return response.json();
	}

	// Step tracking: 'name' | 'email' | 'password' | 'success'
	type Step = 'name' | 'email' | 'password' | 'success';
	let currentStep = $state<Step>('name');

	// Step 1: Name
	let firstName = $state('');
	let lastName = $state('');

	// Step 2: Email
	let selectedEmail = $state('');
	let customEmail = $state('');
	let suggestions = $state<string[]>([]);
	let checkingEmail = $state(false);
	let emailAvailable = $state<boolean | null>(null);
	let showCustomInput = $state(false);

	// Step 3: Password
	let password = $state('');
	let confirmPassword = $state('');
	let showPassword = $state(false);

	// Common
	let error = $state('');
	let loading = $state(false);
	let registeredEmail = $state('');

	// Check if already authenticated
	onMount(async () => {
		if (authStore.isAuthenticated) {
			goto('/');
		}
	});

	// Password strength
	const passwordStrength = $derived.by(() => {
		if (password.length === 0) return 0;
		let strength = 0;
		if (password.length >= 8) strength++;
		if (password.length >= 12) strength++;
		if (/[a-z]/.test(password) && /[A-Z]/.test(password)) strength++;
		if (/\d/.test(password)) strength++;
		if (/[^a-zA-Z0-9]/.test(password)) strength++;
		return Math.min(4, strength);
	});

	const strengthLabel = $derived(['', 'Weak', 'Fair', 'Good', 'Strong'][passwordStrength]);
	const strengthColor = $derived(['bg-gray-200', 'bg-red-500', 'bg-orange-500', 'bg-yellow-500', 'bg-green-500'][passwordStrength]);

	// Validation
	const step1Valid = $derived(firstName.trim().length >= 1 && lastName.trim().length >= 1);
	const step2Valid = $derived(
		(showCustomInput && customEmail.endsWith('@arack.io') && emailAvailable === true) ||
		(!showCustomInput && selectedEmail !== '')
	);
	const step3Valid = $derived(password.length >= 8 && password === confirmPassword);
	const finalEmail = $derived(showCustomInput ? customEmail : selectedEmail);

	// Step 1 -> Step 2
	async function handleNameNext() {
		if (!step1Valid) return;
		loading = true;
		error = '';

		try {
			const result = await getEmailSuggestions(firstName, lastName);
			suggestions = result;
			if (result.length > 0) {
				selectedEmail = result[0];
			}
			currentStep = 'email';
		} catch (err: any) {
			error = err.message || 'Failed to load email suggestions';
		} finally {
			loading = false;
		}
	}

	// Check custom email availability
	let debounceTimer: any;
	function checkCustomEmail() {
		clearTimeout(debounceTimer);
		emailAvailable = null;

		if (!customEmail.endsWith('@arack.io') || customEmail.length < 5) {
			return;
		}

		checkingEmail = true;
		debounceTimer = setTimeout(async () => {
			try {
				const result = await checkEmailAvailabilityAPI(customEmail);
				emailAvailable = result.available;
			} catch {
				emailAvailable = false;
			} finally {
				checkingEmail = false;
			}
		}, 500);
	}

	$effect(() => {
		if (showCustomInput && customEmail) {
			checkCustomEmail();
		}
	});

	// Submit registration
	async function submitRegistration() {
		if (!step3Valid) {
			error = 'Please complete all fields correctly';
			return;
		}

		loading = true;
		error = '';

		try {
			const response = await fetch(`${API_URL}/api/register`, {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				credentials: 'include',
				body: JSON.stringify({
					firstName,
					lastName,
					email: finalEmail,
					password,
					confirmPassword
				})
			});

			const data = await response.json();

			if (!response.ok) {
				throw new Error(data.error || 'Registration failed');
			}

			registeredEmail = finalEmail;
			currentStep = 'success';
		} catch (err: any) {
			error = err.message || 'Registration failed';
		} finally {
			loading = false;
		}
	}

	function goBack() {
		error = '';
		if (currentStep === 'email') {
			currentStep = 'name';
		} else if (currentStep === 'password') {
			currentStep = 'email';
		}
	}
</script>

<svelte:head>
	<title>Create your Arack Account</title>
</svelte:head>

<div class="min-h-screen flex items-center justify-center bg-white px-4 py-12">
	<div class="w-full max-w-[450px]">
		<!-- Card -->
		<div class="border border-gray-200 rounded-lg px-10 py-12 sm:px-12">
			<!-- Logo -->
			<div class="flex justify-center mb-4">
				<svg class="h-10 w-auto" viewBox="0 0 120 40" fill="none" xmlns="http://www.w3.org/2000/svg">
					<text x="0" y="32" font-family="Product Sans, Arial, sans-serif" font-size="32" font-weight="500">
						<tspan fill="#4285F4">A</tspan>
						<tspan fill="#EA4335">r</tspan>
						<tspan fill="#FBBC04">a</tspan>
						<tspan fill="#4285F4">c</tspan>
						<tspan fill="#34A853">k</tspan>
					</text>
				</svg>
			</div>

			{#if currentStep === 'success'}
				<!-- Success State -->
				<div class="text-center">
					<div class="mb-6">
						<div class="w-16 h-16 mx-auto bg-green-100 rounded-full flex items-center justify-center">
							<svg class="w-8 h-8 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
							</svg>
						</div>
					</div>
					<h1 class="text-2xl font-normal text-gray-900 mb-2">Welcome to Arack</h1>
					<p class="text-base text-gray-600 mb-6">Your account has been created</p>

					<div class="bg-gray-50 rounded-lg p-4 mb-8">
						<p class="text-sm text-gray-500 mb-1">Your email address</p>
						<p class="text-lg font-medium text-gray-900">{registeredEmail}</p>
					</div>

					<button
						onclick={() => goto('/')}
						class="w-full px-6 py-3 bg-blue-600 text-white text-sm font-medium rounded hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-600 focus:ring-offset-2 transition-colors"
					>
						Continue to Arack
					</button>
				</div>

			{:else if currentStep === 'name'}
				<!-- Step 1: Name -->
				<div class="text-center mb-8">
					<h1 class="text-2xl font-normal text-gray-900 mb-2">Create your Arack Account</h1>
					<p class="text-base text-gray-600">Enter your name</p>
				</div>

				{#if error}
					<div class="mb-4 flex items-start gap-2 text-sm text-red-600">
						<svg class="w-5 h-5 flex-shrink-0 mt-0.5" fill="currentColor" viewBox="0 0 20 20">
							<path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
						</svg>
						<span>{error}</span>
					</div>
				{/if}

				<form onsubmit={(e) => { e.preventDefault(); handleNameNext(); }}>
					<div class="grid grid-cols-2 gap-4 mb-6">
						<div class="relative">
							<input
								type="text"
								id="firstName"
								bind:value={firstName}
								class="peer w-full px-4 py-4 border border-gray-300 rounded outline-none focus:border-blue-600 focus:ring-1 focus:ring-blue-600 text-base placeholder-transparent"
								placeholder="First name"
								required
								disabled={loading}
							/>
							<label
								for="firstName"
								class="absolute left-3 -top-2.5 bg-white px-1 text-sm text-gray-500 transition-all
									peer-placeholder-shown:top-4 peer-placeholder-shown:left-4 peer-placeholder-shown:text-base peer-placeholder-shown:text-gray-500
									peer-focus:-top-2.5 peer-focus:left-3 peer-focus:text-sm peer-focus:text-blue-600"
							>
								First name
							</label>
						</div>
						<div class="relative">
							<input
								type="text"
								id="lastName"
								bind:value={lastName}
								class="peer w-full px-4 py-4 border border-gray-300 rounded outline-none focus:border-blue-600 focus:ring-1 focus:ring-blue-600 text-base placeholder-transparent"
								placeholder="Last name"
								required
								disabled={loading}
							/>
							<label
								for="lastName"
								class="absolute left-3 -top-2.5 bg-white px-1 text-sm text-gray-500 transition-all
									peer-placeholder-shown:top-4 peer-placeholder-shown:left-4 peer-placeholder-shown:text-base peer-placeholder-shown:text-gray-500
									peer-focus:-top-2.5 peer-focus:left-3 peer-focus:text-sm peer-focus:text-blue-600"
							>
								Last name
							</label>
						</div>
					</div>

					<div class="flex items-center justify-between">
						<a href="/auth/login" class="text-sm font-medium text-blue-600 hover:text-blue-700">
							Sign in instead
						</a>
						<button
							type="submit"
							disabled={!step1Valid || loading}
							class="px-6 py-2.5 bg-blue-600 text-white text-sm font-medium rounded hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-600 focus:ring-offset-2 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
						>
							{loading ? 'Loading...' : 'Next'}
						</button>
					</div>
				</form>

			{:else if currentStep === 'email'}
				<!-- Step 2: Email Selection -->
				<div class="text-center mb-8">
					<h1 class="text-2xl font-normal text-gray-900 mb-2">Choose your email address</h1>
					<p class="text-base text-gray-600">Select or create your @arack.io address</p>
				</div>

				{#if error}
					<div class="mb-4 flex items-start gap-2 text-sm text-red-600">
						<svg class="w-5 h-5 flex-shrink-0 mt-0.5" fill="currentColor" viewBox="0 0 20 20">
							<path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
						</svg>
						<span>{error}</span>
					</div>
				{/if}

				<div class="mb-6">
					<!-- Email Suggestions -->
					{#if suggestions.length > 0 && !showCustomInput}
						<div class="space-y-2 mb-4">
							{#each suggestions as suggestion}
								<label
									class="flex items-center gap-3 p-4 border rounded-lg cursor-pointer hover:bg-gray-50 transition-colors {selectedEmail === suggestion ? 'border-blue-600 bg-blue-50' : 'border-gray-300'}"
								>
									<input
										type="radio"
										bind:group={selectedEmail}
										value={suggestion}
										class="w-4 h-4 text-blue-600 focus:ring-blue-500"
									/>
									<div class="flex-1">
										<span class="text-base text-gray-900">{suggestion}</span>
									</div>
									{#if selectedEmail === suggestion}
										<svg class="w-5 h-5 text-blue-600" fill="currentColor" viewBox="0 0 20 20">
											<path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
										</svg>
									{/if}
								</label>
							{/each}
						</div>
					{/if}

					<!-- Custom email toggle/input -->
					{#if !showCustomInput}
						<button
							type="button"
							onclick={() => { showCustomInput = true; selectedEmail = ''; }}
							class="flex items-center gap-2 text-sm font-medium text-blue-600 hover:text-blue-700"
						>
							<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
							</svg>
							Create a custom email address
						</button>
					{:else}
						<div>
							<button
								type="button"
								onclick={() => { showCustomInput = false; customEmail = ''; emailAvailable = null; if (suggestions.length > 0) selectedEmail = suggestions[0]; }}
								class="flex items-center gap-1 text-sm text-gray-600 hover:text-gray-800 mb-3"
							>
								<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
								</svg>
								Back to suggestions
							</button>

							<div class="relative">
								<input
									type="text"
									id="customEmail"
									bind:value={customEmail}
									class="peer w-full px-4 py-4 border border-gray-300 rounded outline-none focus:border-blue-600 focus:ring-1 focus:ring-blue-600 text-base placeholder-transparent"
									placeholder="username@arack.io"
								/>
								<label
									for="customEmail"
									class="absolute left-3 -top-2.5 bg-white px-1 text-sm text-gray-500 transition-all
										peer-placeholder-shown:top-4 peer-placeholder-shown:left-4 peer-placeholder-shown:text-base peer-placeholder-shown:text-gray-500
										peer-focus:-top-2.5 peer-focus:left-3 peer-focus:text-sm peer-focus:text-blue-600"
								>
									Create your email
								</label>
							</div>

							<div class="mt-2 text-sm">
								{#if checkingEmail}
									<span class="text-gray-500">Checking availability...</span>
								{:else if customEmail.length >= 3}
									{#if !customEmail.endsWith('@arack.io')}
										<span class="text-orange-600">Email must end with @arack.io</span>
									{:else if emailAvailable === true}
										<span class="text-green-600 flex items-center gap-1">
											<svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
												<path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
											</svg>
											Available
										</span>
									{:else if emailAvailable === false}
										<span class="text-red-600">This email is already taken</span>
									{/if}
								{/if}
							</div>
						</div>
					{/if}
				</div>

				<div class="flex items-center justify-between">
					<button
						type="button"
						onclick={goBack}
						class="px-6 py-2.5 text-blue-600 text-sm font-medium rounded hover:bg-blue-50 focus:outline-none focus:ring-2 focus:ring-blue-600 focus:ring-offset-2 transition-colors"
					>
						Back
					</button>
					<button
						type="button"
						onclick={() => currentStep = 'password'}
						disabled={!step2Valid}
						class="px-6 py-2.5 bg-blue-600 text-white text-sm font-medium rounded hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-600 focus:ring-offset-2 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
					>
						Next
					</button>
				</div>

			{:else if currentStep === 'password'}
				<!-- Step 3: Password -->
				<div class="text-center mb-6">
					<h1 class="text-2xl font-normal text-gray-900 mb-2">Create a strong password</h1>
					<p class="text-base text-gray-600">Use 8 or more characters with a mix of letters, numbers & symbols</p>
				</div>

				<!-- Selected email display -->
				<div class="mb-6 p-3 bg-gray-50 rounded-lg flex items-center gap-2">
					<svg class="w-5 h-5 text-gray-500" fill="currentColor" viewBox="0 0 20 20">
						<path d="M2.003 5.884L10 9.882l7.997-3.998A2 2 0 0016 4H4a2 2 0 00-1.997 1.884z" />
						<path d="M18 8.118l-8 4-8-4V14a2 2 0 002 2h12a2 2 0 002-2V8.118z" />
					</svg>
					<span class="text-sm text-gray-700">{finalEmail}</span>
				</div>

				{#if error}
					<div class="mb-4 flex items-start gap-2 text-sm text-red-600">
						<svg class="w-5 h-5 flex-shrink-0 mt-0.5" fill="currentColor" viewBox="0 0 20 20">
							<path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
						</svg>
						<span>{error}</span>
					</div>
				{/if}

				<form onsubmit={(e) => { e.preventDefault(); submitRegistration(); }}>
					<div class="space-y-4 mb-4">
						<div class="relative">
							<input
								type={showPassword ? 'text' : 'password'}
								id="password"
								bind:value={password}
								class="peer w-full px-4 py-4 pr-12 border border-gray-300 rounded outline-none focus:border-blue-600 focus:ring-1 focus:ring-blue-600 text-base placeholder-transparent"
								placeholder="Password"
								required
								disabled={loading}
							/>
							<label
								for="password"
								class="absolute left-3 -top-2.5 bg-white px-1 text-sm text-gray-500 transition-all
									peer-placeholder-shown:top-4 peer-placeholder-shown:left-4 peer-placeholder-shown:text-base peer-placeholder-shown:text-gray-500
									peer-focus:-top-2.5 peer-focus:left-3 peer-focus:text-sm peer-focus:text-blue-600"
							>
								Password
							</label>
							<button
								type="button"
								onclick={() => showPassword = !showPassword}
								class="absolute right-3 top-1/2 -translate-y-1/2 p-1 text-gray-500 hover:text-gray-700"
							>
								{#if showPassword}
									<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" />
									</svg>
								{:else}
									<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
									</svg>
								{/if}
							</button>
						</div>

						<div class="relative">
							<input
								type={showPassword ? 'text' : 'password'}
								id="confirmPassword"
								bind:value={confirmPassword}
								class="peer w-full px-4 py-4 border border-gray-300 rounded outline-none focus:border-blue-600 focus:ring-1 focus:ring-blue-600 text-base placeholder-transparent"
								placeholder="Confirm"
								required
								disabled={loading}
							/>
							<label
								for="confirmPassword"
								class="absolute left-3 -top-2.5 bg-white px-1 text-sm text-gray-500 transition-all
									peer-placeholder-shown:top-4 peer-placeholder-shown:left-4 peer-placeholder-shown:text-base peer-placeholder-shown:text-gray-500
									peer-focus:-top-2.5 peer-focus:left-3 peer-focus:text-sm peer-focus:text-blue-600"
							>
								Confirm password
							</label>
						</div>
					</div>

					<!-- Password strength indicator -->
					{#if password.length > 0}
						<div class="mb-2">
							<div class="flex gap-1 mb-1">
								{#each Array(4) as _, i}
									<div class="h-1 flex-1 rounded-full transition-colors {i < passwordStrength ? strengthColor : 'bg-gray-200'}"></div>
								{/each}
							</div>
							<p class="text-xs text-gray-500">{strengthLabel}</p>
						</div>
					{/if}

					<!-- Password match indicator -->
					{#if confirmPassword.length > 0}
						<div class="mb-4 text-sm">
							{#if password === confirmPassword}
								<span class="text-green-600 flex items-center gap-1">
									<svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
										<path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
									</svg>
									Passwords match
								</span>
							{:else}
								<span class="text-red-600">Passwords don't match</span>
							{/if}
						</div>
					{/if}

					<div class="mb-4">
						<label class="flex items-center gap-2 cursor-pointer">
							<input type="checkbox" bind:checked={showPassword} class="w-4 h-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500" />
							<span class="text-sm text-gray-600">Show password</span>
						</label>
					</div>

					<div class="flex items-center justify-between">
						<button
							type="button"
							onclick={goBack}
							disabled={loading}
							class="px-6 py-2.5 text-blue-600 text-sm font-medium rounded hover:bg-blue-50 focus:outline-none focus:ring-2 focus:ring-blue-600 focus:ring-offset-2 transition-colors disabled:opacity-50"
						>
							Back
						</button>
						<button
							type="submit"
							disabled={!step3Valid || loading}
							class="px-6 py-2.5 bg-blue-600 text-white text-sm font-medium rounded hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-600 focus:ring-offset-2 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
						>
							{#if loading}
								<span class="flex items-center gap-2">
									<svg class="animate-spin h-4 w-4" viewBox="0 0 24 24">
										<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none"/>
										<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"/>
									</svg>
									Creating...
								</span>
							{:else}
								Create account
							{/if}
						</button>
					</div>
				</form>
			{/if}
		</div>

		<!-- Footer -->
		<div class="mt-6 flex items-center justify-between text-sm text-gray-600">
			<div class="flex items-center gap-4">
				<span>English (United States)</span>
			</div>
			<div class="flex items-center gap-4">
				<a href="/help" class="hover:text-gray-800">Help</a>
				<a href="/privacy" class="hover:text-gray-800">Privacy</a>
				<a href="/terms" class="hover:text-gray-800">Terms</a>
			</div>
		</div>
	</div>
</div>
