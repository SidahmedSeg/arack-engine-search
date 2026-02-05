<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { authStore } from '$lib/stores/auth.svelte';
	import axios from 'axios';
	import Button from '$lib/components/ui/Button.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import Card from '$lib/components/ui/Card.svelte';
	import { History, Search as SearchIcon, ChevronLeft, ChevronRight } from 'lucide-svelte';
	import { formatDistanceToNow } from 'date-fns';

	interface SearchHistory {
		id: string;
		kratos_identity_id: string;
		query: string;
		filters: any;
		result_count: number | null;
		clicked_url: string | null;
		clicked_position: number | null;
		created_at: string;
	}

	let history = $state<SearchHistory[]>([]);
	let filteredHistory = $state<SearchHistory[]>([]);
	let isLoading = $state(true);
	let error = $state('');

	// Pagination
	let currentPage = $state(1);
	let itemsPerPage = $state(20);
	let totalPages = $state(1);

	// Filters
	let searchFilter = $state('');
	let showClickedOnly = $state(false);

	onMount(async () => {
		if (!authStore.isAuthenticated) {
			goto('/auth/login');
			return;
		}

		await loadHistory();
	});

	async function loadHistory() {
		isLoading = true;
		error = '';

		try {
			const response = await axios.get(`https://api.arack.io/api/user/search-history?limit=100`, {
				withCredentials: true
			});

			history = response.data.data.history || [];
			applyFilters();
		} catch (err: any) {
			console.error('Failed to load history:', err);
			error = 'Failed to load search history';
		} finally {
			isLoading = false;
		}
	}

	function applyFilters() {
		let filtered = [...history];

		// Apply search filter
		if (searchFilter.trim()) {
			const search = searchFilter.toLowerCase();
			filtered = filtered.filter((item) => item.query.toLowerCase().includes(search));
		}

		// Apply clicked-only filter
		if (showClickedOnly) {
			filtered = filtered.filter((item) => item.clicked_url !== null);
		}

		filteredHistory = filtered;
		totalPages = Math.ceil(filtered.length / itemsPerPage);
		currentPage = 1; // Reset to first page on filter change
	}

	$effect(() => {
		searchFilter;
		showClickedOnly;
		applyFilters();
	});

	function getPaginatedItems() {
		const start = (currentPage - 1) * itemsPerPage;
		const end = start + itemsPerPage;
		return filteredHistory.slice(start, end);
	}

	function nextPage() {
		if (currentPage < totalPages) {
			currentPage++;
		}
	}

	function prevPage() {
		if (currentPage > 1) {
			currentPage--;
		}
	}

	function executeSearch(query: string) {
		goto(`/search?q=${encodeURIComponent(query)}`);
	}
</script>

<div class="space-y-6">
	<!-- Header -->
	<div class="mb-6">
		<h1 class="text-3xl font-bold text-gray-900 mb-2 flex items-center gap-2">
			<History size={32} />
			my search history
		</h1>
		<p class="text-gray-600">View and search through your past queries</p>
	</div>

		{#if error}
			<div class="mb-4 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md">
				<p class="text-sm text-red-600 dark:text-red-400">{error}</p>
			</div>
		{/if}

		<!-- Filters -->
		<Card class="mb-4">
			<div class="flex flex-col sm:flex-row gap-4">
				<div class="flex-1">
					<Input
						type="search"
						placeholder="Filter by query..."
						bind:value={searchFilter}
					/>
				</div>
				<label class="flex items-center gap-2 text-sm cursor-pointer">
					<input
						type="checkbox"
						bind:checked={showClickedOnly}
						class="w-4 h-4 text-blue-600 bg-gray-100 border-gray-300 rounded focus:ring-blue-500"
					/>
					<span class="text-gray-700 dark:text-gray-300">Clicked results only</span>
				</label>
			</div>
		</Card>

		<!-- Data Table -->
		{#if isLoading}
			<Card>
				<p class="text-center py-8 text-gray-600 dark:text-gray-400">Loading...</p>
			</Card>
		{:else if filteredHistory.length === 0}
			<Card>
				<div class="text-center py-12">
					<History size={64} class="mx-auto text-gray-400 mb-4" />
					<p class="text-gray-600 dark:text-gray-400 mb-2">
						{history.length === 0 ? 'No search history yet' : 'No results match your filters'}
					</p>
					<p class="text-sm text-gray-500 dark:text-gray-500">
						{history.length === 0
							? 'Start searching to build your history'
							: 'Try adjusting your filters'}
					</p>
				</div>
			</Card>
		{:else}
			<Card padding="none">
				<div class="overflow-x-auto">
					<table class="w-full">
						<thead class="bg-gray-50 dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700">
							<tr>
								<th class="px-4 py-3 text-left text-xs font-medium text-gray-700 dark:text-gray-300 uppercase tracking-wider">
									Query
								</th>
								<th class="px-4 py-3 text-left text-xs font-medium text-gray-700 dark:text-gray-300 uppercase tracking-wider">
									Results
								</th>
								<th class="px-4 py-3 text-left text-xs font-medium text-gray-700 dark:text-gray-300 uppercase tracking-wider">
									Clicked
								</th>
								<th class="px-4 py-3 text-left text-xs font-medium text-gray-700 dark:text-gray-300 uppercase tracking-wider">
									When
								</th>
								<th class="px-4 py-3 text-right text-xs font-medium text-gray-700 dark:text-gray-300 uppercase tracking-wider">
									Actions
								</th>
							</tr>
						</thead>
						<tbody class="divide-y divide-gray-200 dark:divide-gray-700">
							{#each getPaginatedItems() as item (item.id)}
								<tr class="hover:bg-gray-50 dark:hover:bg-gray-800/50 transition-colors">
									<td class="px-4 py-3 text-sm text-gray-900 dark:text-white max-w-md truncate">
										{item.query}
									</td>
									<td class="px-4 py-3 text-sm text-gray-600 dark:text-gray-400">
										{item.result_count !== null ? item.result_count : '—'}
									</td>
									<td class="px-4 py-3 text-sm">
										{#if item.clicked_url}
											<span class="text-xs bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400 px-2 py-1 rounded">
												Position {item.clicked_position}
											</span>
										{:else}
											<span class="text-gray-400">—</span>
										{/if}
									</td>
									<td class="px-4 py-3 text-sm text-gray-500 dark:text-gray-500">
										{formatDistanceToNow(new Date(item.created_at), { addSuffix: true })}
									</td>
									<td class="px-4 py-3 text-sm text-right">
										<button
											onclick={() => executeSearch(item.query)}
											class="inline-flex items-center text-blue-600 hover:text-blue-700 dark:text-blue-400 font-medium"
										>
											<SearchIcon size={16} class="mr-1" />
											Search
										</button>
									</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			</Card>

			<!-- Pagination -->
			<div class="mt-4 flex items-center justify-between text-sm">
				<p class="text-gray-600 dark:text-gray-400">
					Showing {(currentPage - 1) * itemsPerPage + 1} to {Math.min(currentPage * itemsPerPage, filteredHistory.length)} of {filteredHistory.length} results
				</p>

				<div class="flex items-center gap-2">
					<Button
						variant="secondary"
						size="sm"
						disabled={currentPage === 1}
						onclick={prevPage}
					>
						<ChevronLeft size={16} />
						Previous
					</Button>

					<span class="text-gray-700 dark:text-gray-300 px-3">
						Page {currentPage} of {totalPages}
					</span>

					<Button
						variant="secondary"
						size="sm"
						disabled={currentPage === totalPages}
						onclick={nextPage}
					>
						Next
						<ChevronRight size={16} />
					</Button>
				</div>
			</div>
		{/if}
</div>
