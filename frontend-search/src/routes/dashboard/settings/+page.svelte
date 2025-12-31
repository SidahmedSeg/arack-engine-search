<script lang="ts">
	import { onMount } from 'svelte';
	import { authStore } from '$lib/stores/auth.svelte';
	import { preferencesStore } from '$lib/stores/preferences.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Card from '$lib/components/ui/Card.svelte';
	import { User, Palette, BarChart3, Save } from 'lucide-svelte';

	let resultsPerPage = $state(20);
	let analyticsOptOut = $state(false);
	let isLoading = $state(false);
	let success = $state('');
	let error = $state('');

	onMount(async () => {
		if (!authStore.isAuthenticated) {
			window.location.href = '/auth/login';
			return;
		}

		// Load current preferences
		await preferencesStore.load();
		resultsPerPage = preferencesStore.resultsPerPage;
		analyticsOptOut = preferencesStore.analyticsOptOut;
	});

	async function handleSave() {
		isLoading = true;
		error = '';
		success = '';

		const updated = await preferencesStore.update({
			results_per_page: resultsPerPage,
			analytics_opt_out: analyticsOptOut
		});

		isLoading = false;

		if (updated) {
			success = 'Settings saved successfully!';
			setTimeout(() => (success = ''), 3000);
		} else {
			error = preferencesStore.errorMessage || 'Failed to save settings';
		}
	}
</script>

<div class="space-y-6">
	<div class="mb-6">
		<h1 class="text-3xl font-bold text-gray-900 mb-2">settings</h1>
		<p class="text-gray-600">Manage your account and preferences</p>
	</div>

		{#if success}
			<div class="mb-4 p-3 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-md">
				<p class="text-sm text-green-600 dark:text-green-400">{success}</p>
			</div>
		{/if}

		{#if error}
			<div class="mb-4 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md">
				<p class="text-sm text-red-600 dark:text-red-400">{error}</p>
			</div>
		{/if}

		<div class="space-y-4">
			<!-- Profile Section -->
			<Card>
				<div class="flex items-center gap-3 mb-4">
					<User size={20} class="text-gray-700 dark:text-gray-300" />
					<h2 class="text-lg font-semibold text-gray-900 dark:text-white">Profile</h2>
				</div>

				{#if authStore.user}
					<div class="space-y-3 text-sm">
						<div>
							<span class="text-gray-600 dark:text-gray-400">Name:</span>
							<span class="ml-2 text-gray-900 dark:text-white font-medium">
								{authStore.user.firstName} {authStore.user.lastName}
							</span>
						</div>
						<div>
							<span class="text-gray-600 dark:text-gray-400">Email:</span>
							<span class="ml-2 text-gray-900 dark:text-white font-medium">
								{authStore.user.email}
							</span>
						</div>
					</div>
				{/if}
			</Card>

			<!-- Appearance Section -->
			<Card>
				<div class="flex items-center gap-3 mb-4">
					<Palette size={20} class="text-gray-700 dark:text-gray-300" />
					<h2 class="text-lg font-semibold text-gray-900 dark:text-white">Appearance</h2>
				</div>

				<div class="space-y-3">
					<div>
						<label class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2 block">
							Theme
						</label>
						<div class="flex gap-2">
							<Button
								variant={preferencesStore.theme === 'light' ? 'primary' : 'secondary'}
								size="sm"
								onclick={() => preferencesStore.update({ theme: 'light' })}
							>
								Light
							</Button>
							<Button
								variant={preferencesStore.theme === 'dark' ? 'primary' : 'secondary'}
								size="sm"
								onclick={() => preferencesStore.update({ theme: 'dark' })}
							>
								Dark
							</Button>
						</div>
					</div>

					<div>
						<label for="resultsPerPage" class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2 block">
							Results per page: {resultsPerPage}
						</label>
						<input
							id="resultsPerPage"
							type="range"
							min="10"
							max="100"
							step="10"
							bind:value={resultsPerPage}
							class="w-full h-2 bg-gray-200 dark:bg-gray-700 rounded-lg appearance-none cursor-pointer"
						/>
					</div>
				</div>
			</Card>

			<!-- Privacy Section -->
			<Card>
				<div class="flex items-center gap-3 mb-4">
					<BarChart3 size={20} class="text-gray-700 dark:text-gray-300" />
					<h2 class="text-lg font-semibold text-gray-900 dark:text-white">Privacy</h2>
				</div>

				<div class="flex items-center justify-between">
					<div>
						<p class="text-sm font-medium text-gray-900 dark:text-white">Analytics</p>
						<p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
							Help us improve by tracking your search patterns
						</p>
					</div>
					<label class="relative inline-flex items-center cursor-pointer">
						<input
							type="checkbox"
							bind:checked={analyticsOptOut}
							class="sr-only peer"
						/>
						<div class="w-11 h-6 bg-gray-200 peer-focus:ring-2 peer-focus:ring-blue-500 dark:bg-gray-700 rounded-full peer peer-checked:after:translate-x-full peer-checked:bg-blue-600 after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-5 after:w-5 after:transition-all"></div>
						<span class="ml-3 text-sm font-medium text-gray-900 dark:text-gray-300">
							{analyticsOptOut ? 'Opt Out' : 'Enabled'}
						</span>
					</label>
				</div>
			</Card>

			<!-- Save Button -->
			<div class="flex justify-end gap-2">
				<Button variant="secondary" onclick={() => window.history.back()}>
					Cancel
				</Button>
				<Button variant="primary" onclick={handleSave} disabled={isLoading}>
					<Save size={16} class="mr-2" />
					{isLoading ? 'Saving...' : 'Save Changes'}
				</Button>
			</div>
		</div>
</div>
