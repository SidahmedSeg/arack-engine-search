<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { authStore } from '$lib/stores/auth.svelte';
	import axios from 'axios';
	import Button from '$lib/components/ui/Button.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Card from '$lib/components/ui/Card.svelte';
	import { Bookmark, Plus, Trash2, Search as SearchIcon } from 'lucide-svelte';
	import { formatDistanceToNow } from 'date-fns';

	interface SavedSearch {
		id: string;
		kratos_identity_id: string;
		name: string;
		query: string;
		filters: any;
		created_at: string;
		updated_at: string;
	}

	let searches = $state<SavedSearch[]>([]);
	let isLoading = $state(true);
	let error = $state('');
	let showCreateModal = $state(false);
	let newSearchName = $state('');
	let newSearchQuery = $state('');

	onMount(async () => {
		if (!authStore.isAuthenticated) {
			goto('/auth/login');
			return;
		}

		await loadSearches();
	});

	async function loadSearches() {
		isLoading = true;
		error = '';

		try {
			const response = await axios.get('https://api.arack.io/api/ory/saved-searches', {
				withCredentials: true
			});

			searches = response.data.data.searches || [];
		} catch (err: any) {
			console.error('Failed to load saved searches:', err);
			error = 'Failed to load saved searches';
		} finally {
			isLoading = false;
		}
	}

	async function createSearch() {
		if (!newSearchName || !newSearchQuery) return;

		try {
			await axios.post(
				'https://api.arack.io/api/ory/saved-searches',
				{ name: newSearchName, query: newSearchQuery, filters: null },
				{ withCredentials: true }
			);

			newSearchName = '';
			newSearchQuery = '';
			showCreateModal = false;
			await loadSearches();
		} catch (err) {
			error = 'Failed to create saved search';
		}
	}

	async function deleteSearch(id: string) {
		if (!confirm('Are you sure you want to delete this saved search?')) return;

		try {
			await axios.delete(`https://api.arack.io/api/ory/saved-searches/${id}`, {
				withCredentials: true
			});

			await loadSearches();
		} catch (err) {
			error = 'Failed to delete saved search';
		}
	}

	function executeSearch(query: string) {
		goto(`/search?q=${encodeURIComponent(query)}`);
	}
</script>

<div class="min-h-screen bg-gray-50 dark:bg-gray-900 pt-20 px-4">
	<div class="container mx-auto max-w-5xl">
		<!-- Header -->
		<div class="flex justify-between items-center mb-6">
			<div>
				<h1 class="text-3xl font-bold text-gray-900 dark:text-white mb-2 flex items-center gap-2">
					<Bookmark size={32} />
					Saved Searches
				</h1>
				<p class="text-gray-600 dark:text-gray-400">Manage your saved search queries</p>
			</div>
			<Button variant="primary" onclick={() => (showCreateModal = true)}>
				<Plus size={18} class="mr-2" />
				New Search
			</Button>
		</div>

		{#if error}
			<div class="mb-4 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md">
				<p class="text-sm text-red-600 dark:text-red-400">{error}</p>
			</div>
		{/if}

		<!-- Create Modal -->
		{#if showCreateModal}
			<div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 px-4">
				<Card class="w-full max-w-md">
					<h2 class="text-xl font-bold text-gray-900 dark:text-white mb-4">New Saved Search</h2>

					<div class="space-y-4">
						<Input
							label="Name"
							bind:value={newSearchName}
							placeholder="My Search"
							required
						/>
						<Input
							label="Search Query"
							bind:value={newSearchQuery}
							placeholder="What to search for..."
							required
						/>
					</div>

					<div class="flex justify-end gap-2 mt-6">
						<Button variant="secondary" onclick={() => (showCreateModal = false)}>
							Cancel
						</Button>
						<Button variant="primary" onclick={createSearch}>
							Save
						</Button>
					</div>
				</Card>
			</div>
		{/if}

		<!-- Data Table -->
		{#if isLoading}
			<Card>
				<p class="text-center py-8 text-gray-600 dark:text-gray-400">Loading...</p>
			</Card>
		{:else if searches.length === 0}
			<Card>
				<div class="text-center py-12">
					<Bookmark size={64} class="mx-auto text-gray-400 mb-4" />
					<p class="text-gray-600 dark:text-gray-400 mb-4">No saved searches yet</p>
					<Button variant="primary" onclick={() => (showCreateModal = true)}>
						Create Your First Search
					</Button>
				</div>
			</Card>
		{:else}
			<Card padding="none">
				<div class="overflow-x-auto">
					<table class="w-full">
						<thead class="bg-gray-50 dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700">
							<tr>
								<th class="px-4 py-3 text-left text-xs font-medium text-gray-700 dark:text-gray-300 uppercase tracking-wider">
									Name
								</th>
								<th class="px-4 py-3 text-left text-xs font-medium text-gray-700 dark:text-gray-300 uppercase tracking-wider">
									Query
								</th>
								<th class="px-4 py-3 text-left text-xs font-medium text-gray-700 dark:text-gray-300 uppercase tracking-wider">
									Created
								</th>
								<th class="px-4 py-3 text-right text-xs font-medium text-gray-700 dark:text-gray-300 uppercase tracking-wider">
									Actions
								</th>
							</tr>
						</thead>
						<tbody class="divide-y divide-gray-200 dark:divide-gray-700">
							{#each searches as search (search.id)}
								<tr class="hover:bg-gray-50 dark:hover:bg-gray-800/50 transition-colors">
									<td class="px-4 py-3 text-sm font-medium text-gray-900 dark:text-white">
										{search.name}
									</td>
									<td class="px-4 py-3 text-sm text-gray-600 dark:text-gray-400 max-w-md truncate">
										{search.query}
									</td>
									<td class="px-4 py-3 text-sm text-gray-500 dark:text-gray-500">
										{formatDistanceToNow(new Date(search.created_at), { addSuffix: true })}
									</td>
									<td class="px-4 py-3 text-sm text-right space-x-2">
										<button
											onclick={() => executeSearch(search.query)}
											class="inline-flex items-center text-blue-600 hover:text-blue-700 dark:text-blue-400 font-medium"
										>
											<SearchIcon size={16} class="mr-1" />
											Search
										</button>
										<button
											onclick={() => deleteSearch(search.id)}
											class="inline-flex items-center text-red-600 hover:text-red-700 dark:text-red-400"
										>
											<Trash2 size={16} />
										</button>
									</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			</Card>

			<div class="mt-4 text-sm text-gray-600 dark:text-gray-400">
				Showing {searches.length} saved {searches.length === 1 ? 'search' : 'searches'}
			</div>
		{/if}
	</div>
</div>
