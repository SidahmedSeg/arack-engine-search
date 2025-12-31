<script lang="ts">
	import { onMount } from 'svelte';
	import { FileText, Trash2, RefreshCw, Search, Loader2, Image } from 'lucide-svelte';
	import { api } from '$lib/stores/api';
	import type { IndexStats, SearchResult } from '$shared/types';
	import { formatDate, formatNumber } from '$shared/utils';

	let stats: IndexStats | null = $state(null);
	let imageStats: any | null = $state(null);
	let documents: SearchResult[] = $state([]);
	let loading = $state(true);
	let searchLoading = $state(false);
	let clearLoading = $state(false);
	let error = $state<string | null>(null);

	let searchQuery = $state('');
	let currentPage = $state(0);
	let pageSize = $state(20);
	let totalDocs = $state(0);

	onMount(async () => {
		await loadStats();
		await loadDocuments();
	});

	async function loadStats() {
		try {
			const [statsData, imageStatsData] = await Promise.all([
				api.getStats(),
				api.getImageStats()
			]);
			stats = statsData;
			imageStats = imageStatsData;
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load stats';
		}
	}

	async function loadDocuments() {
		searchLoading = true;
		error = null;

		try {
			const response = await api.search({
				q: searchQuery || '',
				limit: pageSize,
				offset: currentPage * pageSize,
				sort_by: 'crawled_at',
				sort_order: 'desc'
			});

			documents = response.hits;
			totalDocs = response.total_hits;
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load documents';
		} finally {
			searchLoading = false;
			loading = false;
		}
	}

	async function handleSearch() {
		currentPage = 0;
		await loadDocuments();
	}

	async function handleClearIndex() {
		if (!confirm('Are you sure you want to clear the entire index? This cannot be undone.')) {
			return;
		}

		clearLoading = true;
		error = null;

		try {
			await api.clearIndex();
			await loadStats();
			await loadDocuments();
			alert('Index cleared successfully');
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to clear index';
		} finally {
			clearLoading = false;
		}
	}

	function nextPage() {
		if ((currentPage + 1) * pageSize < totalDocs) {
			currentPage++;
			loadDocuments();
		}
	}

	function prevPage() {
		if (currentPage > 0) {
			currentPage--;
			loadDocuments();
		}
	}

	// Watch for search query changes only (not pagination changes)
	let previousQuery = $state(searchQuery);
	$effect(() => {
		if (searchQuery !== previousQuery && !loading) {
			previousQuery = searchQuery;
			currentPage = 0; // Reset to first page on new search
			loadDocuments();
		}
	});
</script>

<div class="space-y-8">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold text-gray-900 flex items-center gap-3">
				<FileText class="w-8 h-8 text-primary" />
				Index Management
			</h1>
			<p class="text-gray-600 mt-2">Manage your search index and browse documents</p>
		</div>

		<button
			onclick={handleClearIndex}
			disabled={clearLoading || loading}
			class="flex items-center gap-2 px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
		>
			{#if clearLoading}
				<Loader2 class="w-5 h-5 animate-spin" />
			{:else}
				<Trash2 class="w-5 h-5" />
			{/if}
			<span>Clear Index</span>
		</button>
	</div>

	<!-- Error Message -->
	{#if error}
		<div class="bg-red-50 border border-red-200 text-red-800 px-4 py-3 rounded-lg">
			<p class="font-semibold">Error</p>
			<p class="text-sm">{error}</p>
		</div>
	{/if}

	<!-- Stats Overview -->
	{#if stats}
		<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
			<div class="bg-white rounded-lg shadow p-6">
				<div class="flex items-center gap-3 mb-2">
					<FileText class="w-5 h-5 text-blue-600" />
					<p class="text-sm text-gray-600">Total Documents</p>
				</div>
				<p class="text-3xl font-bold text-gray-900">{formatNumber(stats.numberOfDocuments)}</p>
			</div>

			<div class="bg-white rounded-lg shadow p-6">
				<div class="flex items-center gap-3 mb-2">
					<Image class="w-5 h-5 text-purple-600" />
					<p class="text-sm text-gray-600">Total Images</p>
				</div>
				<p class="text-3xl font-bold text-gray-900">{formatNumber(imageStats?.numberOfImages ?? 0)}</p>
			</div>

			<div class="bg-white rounded-lg shadow p-6">
				<p class="text-sm text-gray-600 mb-1">Indexing Status</p>
				<p class="text-3xl font-bold {stats.isIndexing || imageStats?.isIndexing ? 'text-orange-600' : 'text-green-600'}">
					{stats.isIndexing || imageStats?.isIndexing ? 'Active' : 'Idle'}
				</p>
			</div>

			<div class="bg-white rounded-lg shadow p-6">
				<p class="text-sm text-gray-600 mb-1">Indexed Fields</p>
				<p class="text-3xl font-bold text-gray-900">
					{Object.keys(stats.fieldDistribution).length}
				</p>
			</div>
		</div>
	{/if}

	<!-- Document Browser -->
	<div class="bg-white rounded-lg shadow">
		<div class="px-6 py-4 border-b border-gray-200">
			<h2 class="text-xl font-semibold text-gray-900 flex items-center gap-2">
				<Search class="w-5 h-5" />
				Browse Documents
			</h2>
		</div>

		<div class="p-6 space-y-6">
			<!-- Search Bar -->
			<div class="flex gap-4">
				<div class="flex-1 relative">
					<Search class="w-5 h-5 absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400" />
					<input
						type="text"
						bind:value={searchQuery}
						placeholder="Search documents..."
						class="w-full pl-10 pr-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary focus:border-transparent"
					/>
				</div>
				<button
					onclick={handleSearch}
					disabled={searchLoading}
					class="px-6 py-3 bg-primary text-white rounded-lg hover:bg-blue-600 disabled:opacity-50 transition-colors"
				>
					{#if searchLoading}
						<Loader2 class="w-5 h-5 animate-spin" />
					{:else}
						Search
					{/if}
				</button>
			</div>

			<!-- Results Info -->
			<div class="flex items-center justify-between text-sm text-gray-600">
				<p>
					Showing {currentPage * pageSize + 1} - {Math.min(
						(currentPage + 1) * pageSize,
						totalDocs
					)}
					of {formatNumber(totalDocs)} documents
				</p>

				<!-- Pagination -->
				<div class="flex gap-2">
					<button
						onclick={prevPage}
						disabled={currentPage === 0 || searchLoading}
						class="px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
					>
						Previous
					</button>
					<button
						onclick={nextPage}
						disabled={(currentPage + 1) * pageSize >= totalDocs || searchLoading}
						class="px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
					>
						Next
					</button>
				</div>
			</div>

			<!-- Documents Table -->
			{#if searchLoading}
				<div class="text-center py-12">
					<Loader2 class="w-8 h-8 animate-spin mx-auto text-primary" />
					<p class="text-gray-600 mt-4">Loading documents...</p>
				</div>
			{:else if documents.length === 0}
				<div class="text-center py-12">
					<FileText class="w-12 h-12 mx-auto text-gray-400" />
					<p class="text-gray-600 mt-4">No documents found</p>
					{#if searchQuery}
						<button
							onclick={() => {
								searchQuery = '';
								handleSearch();
							}}
							class="mt-4 text-primary hover:underline"
						>
							Clear search
						</button>
					{/if}
				</div>
			{:else}
				<div class="overflow-x-auto">
					<table class="w-full">
						<thead class="bg-gray-50 border-b border-gray-200">
							<tr>
								<th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
									Title
								</th>
								<th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
									URL
								</th>
								<th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
									Words
								</th>
								<th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
									Images
								</th>
								<th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
									Crawled At
								</th>
							</tr>
						</thead>
						<tbody class="bg-white divide-y divide-gray-200">
							{#each documents as doc}
								<tr class="hover:bg-gray-50">
									<td class="px-4 py-4">
										<p class="font-medium text-gray-900">{doc.title}</p>
										<p class="text-sm text-gray-600 truncate max-w-md">
											{doc.content.substring(0, 100)}...
										</p>
									</td>
									<td class="px-4 py-4">
										<a
											href={doc.url}
											target="_blank"
											rel="noopener noreferrer"
											class="text-primary hover:underline text-sm break-all"
										>
											{doc.url}
										</a>
									</td>
									<td class="px-4 py-4 text-sm text-gray-900">
										{formatNumber(doc.word_count)}
									</td>
									<td class="px-4 py-4 text-sm text-gray-900">
										<div class="flex items-center gap-1.5">
											<Image class="w-4 h-4 text-purple-600" />
											<span>{doc.image_count ?? 0}</span>
										</div>
									</td>
									<td class="px-4 py-4 text-sm text-gray-600">
										{formatDate(doc.crawled_at)}
									</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			{/if}
		</div>
	</div>
</div>
