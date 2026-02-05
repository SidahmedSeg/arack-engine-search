<script lang="ts">
	import { onMount } from 'svelte';
	import { authStore } from '$lib/stores/auth.svelte';
	import Card from '$lib/components/ui/Card.svelte';
	import Avatar from '$lib/components/ui/avatar/avatar.svelte';
	import { Mail, Hash } from 'lucide-svelte';
	import axios from 'axios';

	let savedSearchCount = $state(0);
	let historyCount = $state(0);
	let loading = $state(true);

	onMount(async () => {
		try {
			const [savedRes, historyRes] = await Promise.all([
				axios.get('https://api.arack.io/api/ory/saved-searches', { withCredentials: true }),
				axios.get('https://api.arack.io/api/user/search-history?limit=1', { withCredentials: true })
			]);

			savedSearchCount = savedRes.data.data.searches?.length || 0;
			historyCount = historyRes.data.data.history?.length || 0;
		} catch (err) {
			console.error('Failed to load stats:', err);
		} finally {
			loading = false;
		}
	});
</script>

<div class="space-y-6">
	<h1 class="text-3xl font-bold text-gray-900">My Profile</h1>

	<!-- Profile Card -->
	<Card>
		<div class="flex items-start gap-6">
			<Avatar user={authStore.user} size="xl" />

			<div class="flex-1 space-y-4">
				<div>
					<h2 class="text-2xl font-semibold text-gray-900">
						{authStore.user.firstName} {authStore.user.lastName}
					</h2>
					<p class="text-gray-500">{authStore.user.email}</p>
				</div>

				<div class="grid grid-cols-1 md:grid-cols-2 gap-4 pt-4 border-t border-gray-200">
					<div class="flex items-center gap-2 text-sm">
						<Mail size={16} class="text-gray-400" />
						<span class="text-gray-700">{authStore.user.email}</span>
					</div>

					{#if !loading}
						<div class="flex items-center gap-2 text-sm">
							<Hash size={16} class="text-gray-400" />
							<span class="text-gray-700">{savedSearchCount} saved searches</span>
						</div>
					{/if}
				</div>
			</div>
		</div>
	</Card>

	<!-- Account Stats -->
	<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
		<Card>
			<h3 class="text-lg font-semibold text-gray-900 mb-2">Saved Searches</h3>
			<p class="text-3xl font-bold text-blue-600">{savedSearchCount}</p>
			<p class="text-sm text-gray-500 mt-1">Quick access to your favorite searches</p>
		</Card>

		<Card>
			<h3 class="text-lg font-semibold text-gray-900 mb-2">Search History</h3>
			<p class="text-3xl font-bold text-blue-600">{historyCount}+</p>
			<p class="text-sm text-gray-500 mt-1">Total searches tracked</p>
		</Card>
	</div>
</div>
