<script lang="ts">
	import { goto } from '$app/navigation';
	import { authStore } from '$lib/stores/auth.svelte';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import * as Card from '$lib/components/ui/card';
	import { getEmailSuggestions, checkEmailAvailability } from '$lib/auth/sso';
	import { onMount } from 'svelte';

	// Step tracking
	let currentStep = $state(1);
	const totalSteps = 3;

	// Step 1: Personal Information
	let firstName = $state('');
	let lastName = $state('');
	let dateOfBirth = $state('');
	let gender = $state('');

	// Step 2: Email Selection (changed from username)
	let selectedEmail = $state('');
	let customEmail = $state('');
	let suggestions = $state<string[]>([]);
	let checkingCustom = $state(false);
	let customAvailable = $state<boolean | null>(null);
	let useCustomEmail = $state(false);

	// Step 3: Password
	let password = $state('');
	let confirmPassword = $state('');

	// Common
	let error = $state('');
	let loading = $state(false);
	let registrationSuccess = $state(false);
	let registeredEmail = $state('');

	// Check if already authenticated
	onMount(async () => {
		if (authStore.isAuthenticated) {
			goto('/');
		}
	});

	// Password strength calculation
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

	const strengthLabel = $derived.by(() => {
		switch (passwordStrength) {
			case 1:
				return 'Weak';
			case 2:
				return 'Fair';
			case 3:
				return 'Good';
			case 4:
				return 'Strong';
			default:
				return '';
		}
	});

	const strengthColor = $derived.by(() => {
		switch (passwordStrength) {
			case 1:
				return 'bg-red-500';
			case 2:
				return 'bg-orange-500';
			case 3:
				return 'bg-yellow-500';
			case 4:
				return 'bg-green-500';
			default:
				return 'bg-gray-200';
		}
	});

	// Form validation
	const step1Valid = $derived(firstName.trim() !== '' && lastName.trim() !== '');

	const step2Valid = $derived(
		(useCustomEmail && customEmail.endsWith('@arack.io') && customAvailable === true) ||
			(!useCustomEmail && selectedEmail !== '')
	);

	const step3Valid = $derived(password.length >= 8 && password === confirmPassword);

	// Get final email
	const finalEmail = $derived(useCustomEmail ? customEmail : selectedEmail);

	// Load email suggestions when moving to step 2
	async function loadSuggestions() {
		if (!step1Valid) return;

		loading = true;
		error = '';

		try {
			const result = await getEmailSuggestions(firstName, lastName);
			suggestions = result;

			// Auto-select first suggestion
			if (result.length > 0) {
				selectedEmail = result[0];
				useCustomEmail = false;
			}

			currentStep = 2;
		} catch (err: any) {
			error = err.message || 'Failed to load suggestions';
		} finally {
			loading = false;
		}
	}

	// Check custom email availability (debounced)
	let debounceTimer: any;
	function checkCustomEmailAvailability() {
		clearTimeout(debounceTimer);

		if (!customEmail.endsWith('@arack.io')) {
			customAvailable = null;
			return;
		}

		checkingCustom = true;

		debounceTimer = setTimeout(async () => {
			try {
				const result = await checkEmailAvailability(customEmail);
				customAvailable = result.available;
			} catch (err) {
				customAvailable = false;
			} finally {
				checkingCustom = false;
			}
		}, 500);
	}

	$effect(() => {
		if (useCustomEmail && customEmail) {
			checkCustomEmailAvailability();
		}
	});

	// Submit registration
	async function submitRegistration() {
		if (!step3Valid) {
			error = 'Please complete all fields';
			return;
		}

		loading = true;
		error = '';

		try {
			await authStore.register({
				firstName,
				lastName,
				gender: gender || undefined,
				birthDate: dateOfBirth || undefined,
				email: finalEmail,
				password,
				confirmPassword
			});

			// Success - show success message
			registrationSuccess = true;
			registeredEmail = finalEmail;
		} catch (err: any) {
			error = err.message || 'Registration failed';
		} finally {
			loading = false;
		}
	}
</script>

<div class="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4">
	<Card.Root class="w-full max-w-md">
		{#if registrationSuccess}
			<!-- Success State -->
			<Card.Header>
				<Card.Title class="text-green-600">Account Created!</Card.Title>
				<Card.Description> Your Arack account has been successfully created. </Card.Description>
			</Card.Header>

			<Card.Content>
				<div class="space-y-4">
					<div class="p-4 bg-green-50 border border-green-200 rounded-lg">
						<p class="text-sm text-green-800">
							<strong>Your email address:</strong>
						</p>
						<p class="text-lg font-medium text-green-900 mt-1">
							{registeredEmail}
						</p>
					</div>

					<p class="text-sm text-gray-600">
						You can now sign in to access your email and all Arack services.
					</p>

					<Button onclick={() => goto('/')} class="w-full">Go to Home</Button>
				</div>
			</Card.Content>
		{:else}
			<Card.Header>
				<Card.Title>Create your Arack account</Card.Title>
				<Card.Description> Step {currentStep} of {totalSteps} </Card.Description>
			</Card.Header>

			<Card.Content>
				<!-- Progress Bar -->
				<div class="mb-6">
					<div class="flex gap-2">
						{#each Array(totalSteps) as _, i}
							<div
								class="h-2 flex-1 rounded-full transition-colors {i < currentStep
									? 'bg-blue-600'
									: 'bg-gray-200'}"
							></div>
						{/each}
					</div>
				</div>

				{#if error}
					<div class="mb-4 p-3 bg-red-50 border border-red-200 rounded-md text-red-600 text-sm">
						{error}
					</div>
				{/if}

				{#if currentStep === 1}
					<!-- Step 1: Personal Information -->
					<div class="space-y-4">
						<div class="grid grid-cols-2 gap-4">
							<div>
								<Label for="firstName">First Name</Label>
								<Input
									id="firstName"
									bind:value={firstName}
									placeholder="John"
									required
									disabled={loading}
								/>
							</div>
							<div>
								<Label for="lastName">Last Name</Label>
								<Input
									id="lastName"
									bind:value={lastName}
									placeholder="Doe"
									required
									disabled={loading}
								/>
							</div>
						</div>

						<div>
							<Label for="dateOfBirth">Date of Birth (optional)</Label>
							<Input
								id="dateOfBirth"
								type="date"
								bind:value={dateOfBirth}
								max={new Date().toISOString().split('T')[0]}
								disabled={loading}
							/>
						</div>

						<div>
							<Label>Gender (optional)</Label>
							<div class="flex gap-4 mt-2">
								<label class="flex items-center gap-2 cursor-pointer">
									<input
										type="radio"
										bind:group={gender}
										value="male"
										class="w-4 h-4"
										disabled={loading}
									/>
									<span>Male</span>
								</label>
								<label class="flex items-center gap-2 cursor-pointer">
									<input
										type="radio"
										bind:group={gender}
										value="female"
										class="w-4 h-4"
										disabled={loading}
									/>
									<span>Female</span>
								</label>
								<label class="flex items-center gap-2 cursor-pointer">
									<input
										type="radio"
										bind:group={gender}
										value="other"
										class="w-4 h-4"
										disabled={loading}
									/>
									<span>Other</span>
								</label>
							</div>
						</div>

						<Button onclick={loadSuggestions} disabled={!step1Valid || loading} class="w-full">
							{loading ? 'Loading...' : 'Continue'}
						</Button>
					</div>
				{:else if currentStep === 2}
					<!-- Step 2: Email Selection -->
					<div class="space-y-4">
						{#if suggestions.length > 0}
							<div>
								<Label>Choose your email address</Label>
								<div class="space-y-2 mt-2">
									{#each suggestions as suggestion}
										<label
											class="flex items-center gap-3 p-3 border rounded-lg cursor-pointer hover:bg-gray-50 transition-colors {selectedEmail ===
												suggestion && !useCustomEmail
												? 'border-blue-600 bg-blue-50'
												: 'border-gray-200'}"
										>
											<input
												type="radio"
												bind:group={selectedEmail}
												value={suggestion}
												onclick={() => (useCustomEmail = false)}
												class="w-4 h-4"
											/>
											<div class="flex-1">
												<div class="font-medium">{suggestion}</div>
												{#if selectedEmail === suggestion && !useCustomEmail}
													<div class="text-xs text-green-600">Selected</div>
												{/if}
											</div>
										</label>
									{/each}
								</div>
							</div>
						{/if}

						<div class="relative">
							<Label>Or create custom email</Label>
							<div class="flex items-center gap-2 mt-2">
								<Input
									bind:value={customEmail}
									placeholder="yourname@arack.io"
									onfocus={() => (useCustomEmail = true)}
								/>
							</div>
							{#if useCustomEmail}
								{#if checkingCustom}
									<p class="text-sm text-gray-500 mt-1">Checking availability...</p>
								{:else if customEmail.length >= 3}
									{#if !customEmail.endsWith('@arack.io')}
										<p class="text-sm text-orange-600 mt-1">Email must end with @arack.io</p>
									{:else if customAvailable}
										<p class="text-sm text-green-600 mt-1">Available!</p>
									{:else if customAvailable === false}
										<p class="text-sm text-red-600 mt-1">Already taken</p>
									{/if}
								{/if}
							{/if}
						</div>

						<div class="flex gap-2">
							<Button
								variant="outline"
								onclick={() => (currentStep = 1)}
								class="flex-1"
								disabled={loading}
							>
								Back
							</Button>
							<Button
								onclick={() => (currentStep = 3)}
								disabled={!step2Valid || loading}
								class="flex-1"
							>
								Continue
							</Button>
						</div>
					</div>
				{:else if currentStep === 3}
					<!-- Step 3: Password -->
					<div class="space-y-4">
						<div class="p-3 bg-blue-50 border border-blue-200 rounded-md">
							<p class="text-sm text-blue-800">
								Your email: <strong>{finalEmail}</strong>
							</p>
						</div>

						<div>
							<Label for="password">Password</Label>
							<Input
								id="password"
								type="password"
								bind:value={password}
								placeholder="Minimum 8 characters"
								required
								disabled={loading}
							/>
							{#if password.length > 0}
								<div class="mt-2 flex gap-1">
									{#each Array(4) as _, i}
										<div
											class="h-2 flex-1 rounded-full transition-colors {i < passwordStrength
												? strengthColor
												: 'bg-gray-200'}"
										></div>
									{/each}
								</div>
								<p class="text-xs text-gray-500 mt-1">
									{strengthLabel}
								</p>
							{/if}
						</div>

						<div>
							<Label for="confirmPassword">Confirm Password</Label>
							<Input
								id="confirmPassword"
								type="password"
								bind:value={confirmPassword}
								placeholder="Re-enter password"
								required
								disabled={loading}
							/>
							{#if confirmPassword.length > 0}
								{#if password === confirmPassword}
									<p class="text-sm text-green-600 mt-1">Passwords match</p>
								{:else}
									<p class="text-sm text-red-600 mt-1">Passwords don't match</p>
								{/if}
							{/if}
						</div>

						<div class="flex gap-2">
							<Button
								variant="outline"
								onclick={() => (currentStep = 2)}
								class="flex-1"
								disabled={loading}
							>
								Back
							</Button>
							<Button onclick={submitRegistration} disabled={!step3Valid || loading} class="flex-1">
								{loading ? 'Creating Account...' : 'Create Account'}
							</Button>
						</div>
					</div>
				{/if}
			</Card.Content>

			<Card.Footer>
				<p class="text-sm text-gray-500 text-center w-full">
					Already have an account?
					<a href="/auth/login" class="text-blue-600 hover:underline"> Sign in </a>
				</p>
			</Card.Footer>
		{/if}
	</Card.Root>
</div>
