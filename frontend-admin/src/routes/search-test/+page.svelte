<script lang="ts">
	import { Search, Loader2, Clock, FileText } from 'lucide-svelte';
	import { api } from '$lib/stores/api';
	import type { SearchResponse, SearchParams } from '$shared/types';
	import { formatNumber } from '$shared/utils';

	let query = $state('');
	let limit = $state(20);
	let offset = $state(0);
	let minWordCount = $state<number | undefined>(undefined);
	let maxWordCount = $state<number | undefined>(undefined);
	let sortBy = $state<'crawled_at' | 'word_count' | undefined>(undefined);
	let sortOrder = $state<'asc' | 'desc'>('desc');

	let results: SearchResponse | null = $state(null);
	let loading = $state(false);
	let error = $state<string | null>(null);
	let showJson = $state(false);

	async function handleSearch(e?: Event) {
		if (e) e.preventDefault();

		if (!query.trim()) {
			error = 'Please enter a search query';
			return;
		}

		loading = true;
		error = null;

		const params: SearchParams = {
			q: query,
			limit,
			offset
		};

		if (minWordCount !== undefined && minWordCount > 0) {
			params.min_word_count = minWordCount;
		}
		if (maxWordCount !== undefined && maxWordCount > 0) {
			params.max_word_count = maxWordCount;
		}
		if (sortBy) {
			params.sort_by = sortBy;
			params.sort_order = sortOrder;
		}

		try {
			results = await api.search(params);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Search failed';
		} finally {
			loading = false;
		}
	}

	function resetFilters() {
		limit = 20;
		offset = 0;
		minWordCount = undefined;
		maxWordCount = undefined;
		sortBy = undefined;
		sortOrder = 'desc';
	}
</script>

<div class="space-y-8">
	<!-- Header -->
	<div>
		<h1 class="text-3xl font-bold text-gray-900 flex items-center gap-3">
			<Search class="w-8 h-8 text-primary" />
			Search Testing
		</h1>
		<p class="text-gray-600 mt-2">Test search queries with advanced filters and parameters</p>
	</div>

	<!-- Search Form -->
	<div class="bg-white rounded-lg shadow p-6">
		<form onsubmit={handleSearch} class="space-y-6">
			<!-- Search Query -->
			<div>
				<label for="query" class="block text-sm font-medium text-gray-700 mb-2">
					Search Query
					<span class="text-red-500">*</span>
				</label>
				<div class="flex gap-4">
					<input
						id="query"
						type="text"
						bind:value={query}
						placeholder="Enter search query..."
						class="flex-1 px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary focus:border-transparent"
						required
					/>
					<button
						type="submit"
						disabled={loading}
						class="px-6 py-3 bg-primary text-white rounded-lg hover:bg-blue-600 disabled:opacity-50 transition-colors flex items-center gap-2"
					>
						{#if loading}
							<Loader2 class="w-5 h-5 animate-spin" />
							<span>Searching...</span>
						{:else}
							<Search class="w-5 h-5" />
							<span>Search</span>
						{/if}
					</button>
				</div>
			</div>

			<!-- Filters Grid -->
			<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
				<!-- Limit -->
				<div>
					<label for="limit" class="block text-sm font-medium text-gray-700 mb-2">
						Limit
					</label>
					<input
						id="limit"
						type="number"
						bind:value={limit}
						min="1"
						max="100"
						class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary focus:border-transparent"
					/>
				</div>

				<!-- Offset -->
				<div>
					<label for="offset" class="block text-sm font-medium text-gray-700 mb-2">
						Offset
					</label>
					<input
						id="offset"
						type="number"
						bind:value={offset}
						min="0"
						class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary focus:border-transparent"
					/>
				</div>

				<!-- Min Word Count -->
				<div>
					<label for="minWordCount" class="block text-sm font-medium text-gray-700 mb-2">
						Min Word Count
					</label>
					<input
						id="minWordCount"
						type="number"
						bind:value={minWordCount}
						min="0"
						placeholder="No minimum"
						class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary focus:border-transparent"
					/>
				</div>

				<!-- Max Word Count -->
				<div>
					<label for="maxWordCount" class="block text-sm font-medium text-gray-700 mb-2">
						Max Word Count
					</label>
					<input
						id="maxWordCount"
						type="number"
						bind:value={maxWordCount}
						min="0"
						placeholder="No maximum"
						class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary focus:border-transparent"
					/>
				</div>

				<!-- Sort By -->
				<div>
					<label for="sortBy" class="block text-sm font-medium text-gray-700 mb-2">
						Sort By
					</label>
					<select
						id="sortBy"
						bind:value={sortBy}
						class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary focus:border-transparent"
					>
						<option value={undefined}>Relevance</option>
						<option value="crawled_at">Crawled Date</option>
						<option value="word_count">Word Count</option>
					</select>
				</div>

				<!-- Sort Order -->
				<div>
					<label for="sortOrder" class="block text-sm font-medium text-gray-700 mb-2">
						Sort Order
					</label>
					<select
						id="sortOrder"
						bind:value={sortOrder}
						class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary focus:border-transparent"
					>
						<option value="desc">Descending</option>
						<option value="asc">Ascending</option>
					</select>
				</div>
			</div>

			<!-- Reset Button -->
			<button
				type="button"
				onclick={resetFilters}
				class="text-sm text-primary hover:underline"
			>
				Reset Filters
			</button>
		</form>
	</div>

	<!-- Error Message -->
	{#if error}
		<div class="bg-red-50 border border-red-200 text-red-800 px-4 py-3 rounded-lg">
			<p class="font-semibold">Error</p>
			<p class="text-sm">{error}</p>
		</div>
	{/if}

	<!-- Results -->
	{#if results}
		<div class="space-y-6">
			<!-- Results Header -->
			<div class="bg-white rounded-lg shadow p-4">
				<div class="flex items-center justify-between">
					<div class="flex items-center gap-4 text-sm text-gray-600">
						<span class="flex items-center gap-2">
							<FileText class="w-4 h-4" />
							<strong>{formatNumber(results.total_hits)}</strong> results
						</span>
						<span class="flex items-center gap-2">
							<Clock class="w-4 h-4" />
							<strong>{results.processing_time_ms}ms</strong>
						</span>
					</div>

					<button
						onclick={() => (showJson = !showJson)}
						class="text-sm text-primary hover:underline"
					>
						{showJson ? 'Hide' : 'Show'} JSON
					</button>
				</div>
			</div>

			<!-- JSON Viewer -->
			{#if showJson}
				<div class="bg-gray-900 text-gray-100 rounded-lg p-6 overflow-x-auto">
					<pre class="text-sm font-mono">{JSON.stringify(results, null, 2)}</pre>
				</div>
			{/if}

			<!-- Results List -->
			<div class="space-y-4">
				{#each results.hits as result}
					<div class="bg-white rounded-lg shadow p-6">
						<h3 class="text-lg font-semibold text-gray-900 mb-2">
							{result.title}
						</h3>

						<a
							href={result.url}
							target="_blank"
							rel="noopener noreferrer"
							class="text-sm text-primary hover:underline mb-3 block"
						>
							{result.url}
						</a>

						<p class="text-gray-700 mb-4">
							{result.content.substring(0, 200)}...
						</p>

						<div class="flex gap-4 text-sm text-gray-600">
							<span>
								<strong>Word Count:</strong>
								{formatNumber(result.word_count)}
							</span>
							<span>
								<strong>Crawled:</strong>
								{new Date(result.crawled_at).toLocaleString()}
							</span>
						</div>
					</div>
				{/each}
			</div>
		</div>
	{/if}
</div>
