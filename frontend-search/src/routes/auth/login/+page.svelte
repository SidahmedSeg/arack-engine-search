<script lang="ts">
	import { authStore } from '$lib/stores/auth.svelte';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';

	// Two-step flow like Google
	let step = $state<'email' | 'password'>('email');
	let email = $state('');
	let password = $state('');
	let showPassword = $state(false);
	let isLoading = $state(false);
	let error = $state('');

	// Get return_url for redirect after login
	let returnUrl = $derived($page.url.searchParams.get('return_url'));

	// Helper to redirect
	function redirectTo(url: string | null) {
		const target = url || '/';
		if (target.startsWith('http://') || target.startsWith('https://')) {
			window.location.href = target;
		} else {
			goto(target);
		}
	}

	// Check if already authenticated
	onMount(async () => {
		if (authStore.isAuthenticated) {
			redirectTo(returnUrl);
		}
	});

	function handleEmailNext(e: Event) {
		e.preventDefault();
		error = '';

		if (!email || !email.includes('@')) {
			error = 'Enter a valid email address';
			return;
		}

		step = 'password';
	}

	function handleBack() {
		step = 'email';
		password = '';
		error = '';
	}

	async function handleLogin(e: Event) {
		e.preventDefault();
		isLoading = true;
		error = '';

		try {
			await authStore.loginWithPassword(email, password);
			redirectTo(returnUrl);
		} catch (err: any) {
			error = err.message || 'Wrong password. Try again or click Forgot password to reset it.';
		} finally {
			isLoading = false;
		}
	}
</script>

<svelte:head>
	<title>Sign in - Arack</title>
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

			{#if step === 'email'}
				<!-- Email Step -->
				<div class="text-center mb-8">
					<h1 class="text-2xl font-normal text-gray-900 mb-2">Sign in</h1>
					<p class="text-base text-gray-600">Use your Arack Account</p>
				</div>

				{#if error}
					<div class="mb-4 flex items-start gap-2 text-sm text-red-600">
						<svg class="w-5 h-5 flex-shrink-0 mt-0.5" fill="currentColor" viewBox="0 0 20 20">
							<path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
						</svg>
						<span>{error}</span>
					</div>
				{/if}

				<form onsubmit={handleEmailNext}>
					<div class="mb-6">
						<div class="relative">
							<input
								type="email"
								id="email"
								bind:value={email}
								class="peer w-full px-4 py-4 border border-gray-300 rounded outline-none focus:border-blue-600 focus:ring-1 focus:ring-blue-600 text-base placeholder-transparent"
								placeholder="Email"
								required
							/>
							<label
								for="email"
								class="absolute left-3 -top-2.5 bg-white px-1 text-sm text-gray-500 transition-all
									peer-placeholder-shown:top-4 peer-placeholder-shown:left-4 peer-placeholder-shown:text-base peer-placeholder-shown:text-gray-500
									peer-focus:-top-2.5 peer-focus:left-3 peer-focus:text-sm peer-focus:text-blue-600"
							>
								Email or phone
							</label>
						</div>
					</div>

					<div class="mb-8">
						<a href="/auth/recovery" class="text-sm font-medium text-blue-600 hover:text-blue-700">
							Forgot email?
						</a>
					</div>

					<div class="flex items-center justify-between">
						<a
							href="/auth/register"
							class="text-sm font-medium text-blue-600 hover:text-blue-700"
						>
							Create account
						</a>
						<button
							type="submit"
							class="px-6 py-2.5 bg-blue-600 text-white text-sm font-medium rounded hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-600 focus:ring-offset-2 transition-colors"
						>
							Next
						</button>
					</div>
				</form>

			{:else}
				<!-- Password Step -->
				<div class="mb-6">
					<button
						type="button"
						onclick={handleBack}
						class="inline-flex items-center gap-2 px-3 py-1.5 -ml-3 text-sm text-gray-700 hover:bg-gray-100 rounded-full transition-colors"
					>
						<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
						</svg>
						<span class="max-w-[200px] truncate">{email}</span>
						<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
						</svg>
					</button>
				</div>

				<div class="text-center mb-8">
					<h1 class="text-2xl font-normal text-gray-900 mb-2">Welcome</h1>
					<div class="flex items-center justify-center gap-2">
						<div class="w-6 h-6 rounded-full bg-blue-100 flex items-center justify-center">
							<svg class="w-4 h-4 text-blue-600" fill="currentColor" viewBox="0 0 20 20">
								<path fill-rule="evenodd" d="M10 9a3 3 0 100-6 3 3 0 000 6zm-7 9a7 7 0 1114 0H3z" clip-rule="evenodd" />
							</svg>
						</div>
						<span class="text-sm text-gray-600 truncate max-w-[200px]">{email}</span>
					</div>
				</div>

				{#if error}
					<div class="mb-4 flex items-start gap-2 text-sm text-red-600">
						<svg class="w-5 h-5 flex-shrink-0 mt-0.5" fill="currentColor" viewBox="0 0 20 20">
							<path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
						</svg>
						<span>{error}</span>
					</div>
				{/if}

				<form onsubmit={handleLogin}>
					<div class="mb-2">
						<div class="relative">
							<input
								type={showPassword ? 'text' : 'password'}
								id="password"
								bind:value={password}
								class="peer w-full px-4 py-4 pr-12 border border-gray-300 rounded outline-none focus:border-blue-600 focus:ring-1 focus:ring-blue-600 text-base placeholder-transparent"
								placeholder="Password"
								required
								disabled={isLoading}
							/>
							<label
								for="password"
								class="absolute left-3 -top-2.5 bg-white px-1 text-sm text-gray-500 transition-all
									peer-placeholder-shown:top-4 peer-placeholder-shown:left-4 peer-placeholder-shown:text-base peer-placeholder-shown:text-gray-500
									peer-focus:-top-2.5 peer-focus:left-3 peer-focus:text-sm peer-focus:text-blue-600"
							>
								Enter your password
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
					</div>

					<div class="mb-8">
						<label class="flex items-center gap-2 cursor-pointer">
							<input type="checkbox" bind:checked={showPassword} class="w-4 h-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500" />
							<span class="text-sm text-gray-600">Show password</span>
						</label>
					</div>

					<div class="flex items-center justify-between">
						<a href="/auth/recovery" class="text-sm font-medium text-blue-600 hover:text-blue-700">
							Forgot password?
						</a>
						<button
							type="submit"
							disabled={isLoading || !password}
							class="px-6 py-2.5 bg-blue-600 text-white text-sm font-medium rounded hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-600 focus:ring-offset-2 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
						>
							{#if isLoading}
								<span class="flex items-center gap-2">
									<svg class="animate-spin h-4 w-4" viewBox="0 0 24 24">
										<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none"/>
										<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"/>
									</svg>
									Signing in...
								</span>
							{:else}
								Next
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
