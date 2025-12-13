<script lang="ts">
	import { goto } from '$app/navigation';
	import { auth } from '$lib/stores/auth';

	let email = $state('');
	let password = $state('');
	let error = $state('');
	let loading = $state(false);

	async function handleSubmit(e: Event) {
		e.preventDefault();
		error = '';
		loading = true;

		const result = await auth.login(email, password);

		loading = false;

		if (result.success) {
			// Redirect to dashboard on successful login
			goto('/');
		} else {
			error = result.error || 'Login failed';
		}
	}
</script>

<svelte:head>
	<title>Login - Search Engine Admin</title>
</svelte:head>

<div class="min-h-screen bg-gradient-to-br from-blue-50 via-white to-purple-50 flex items-center justify-center p-4">
	<div class="w-full max-w-md">
		<!-- Logo/Header -->
		<div class="text-center mb-8">
			<div class="inline-block p-3 bg-blue-600 rounded-2xl mb-4">
				<svg class="w-8 h-8 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
				</svg>
			</div>
			<h1 class="text-3xl font-bold text-gray-900">Welcome back</h1>
			<p class="text-gray-600 mt-2">Sign in to your admin account</p>
		</div>

		<!-- Login Card -->
		<div class="bg-white rounded-2xl shadow-xl p-8">
			<form onsubmit={handleSubmit} class="space-y-6">
				<!-- Email Field -->
				<div>
					<label for="email" class="block text-sm font-medium text-gray-700 mb-2">
						Email address
					</label>
					<input
						type="email"
						id="email"
						bind:value={email}
						required
						autocomplete="email"
						class="w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all outline-none"
						placeholder="you@example.com"
						disabled={loading}
					/>
				</div>

				<!-- Password Field -->
				<div>
					<label for="password" class="block text-sm font-medium text-gray-700 mb-2">
						Password
					</label>
					<input
						type="password"
						id="password"
						bind:value={password}
						required
						autocomplete="current-password"
						class="w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all outline-none"
						placeholder="••••••••"
						disabled={loading}
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
					disabled={loading}
					class="w-full bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 text-white font-medium py-3 px-4 rounded-lg transition-colors duration-200 flex items-center justify-center"
				>
					{#if loading}
						<svg class="animate-spin h-5 w-5 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
							<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
							<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
						</svg>
						<span class="ml-2">Signing in...</span>
					{:else}
						Sign in
					{/if}
				</button>
			</form>
		</div>

		<!-- Footer Text -->
		<div class="text-center mt-6 text-sm text-gray-600">
			<p>Don't have an account? Contact your administrator for an invitation.</p>
		</div>
	</div>
</div>
