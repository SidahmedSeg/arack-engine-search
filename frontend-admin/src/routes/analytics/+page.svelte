<script lang="ts">
	import { BarChart3, TrendingUp, MousePointerClick, Search } from 'lucide-svelte';
	import { onMount } from 'svelte';
	import { api } from '$lib/stores/api';

	interface QueryStats {
		query: string;
		search_count: number;
		avg_result_count: number;
		avg_processing_time_ms: number;
		click_count: number;
		click_through_rate: number;
		last_searched: string;
	}

	interface AnalyticsSummary {
		total_searches: number;
		total_clicks: number;
		avg_click_through_rate: number;
		top_queries: QueryStats[];
		zero_result_queries: string[];
		popular_results: any[];
	}

	let days = $state(7);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let analytics = $state<AnalyticsSummary | null>(null);

	async function fetchAnalytics() {
		loading = true;
		error = null;

		try {
			analytics = await api.getAnalyticsSummary(days);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to fetch analytics';
			console.error('Analytics error:', err);
		} finally {
			loading = false;
		}
	}

	onMount(() => {
		fetchAnalytics();
	});

	function formatDate(dateStr: string) {
		return new Date(dateStr).toLocaleString();
	}
</script>

<div class="space-y-8">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold text-gray-900 flex items-center gap-3">
				<BarChart3 class="w-8 h-8 text-primary" />
				Search Analytics
			</h1>
			<p class="text-gray-600 mt-2">Track search queries, clicks, and user engagement</p>
		</div>

		<!-- Time Range Selector -->
		<div class="flex items-center gap-2">
			<label for="days" class="text-sm font-medium text-gray-700">Time Range:</label>
			<select
				id="days"
				bind:value={days}
				onchange={() => fetchAnalytics()}
				class="px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary focus:border-primary"
			>
				<option value={1}>Last 24 hours</option>
				<option value={7}>Last 7 days</option>
				<option value={30}>Last 30 days</option>
				<option value={90}>Last 90 days</option>
			</select>
		</div>
	</div>

	{#if loading}
		<div class="flex items-center justify-center py-12">
			<div class="w-8 h-8 border-4 border-primary border-t-transparent rounded-full animate-spin"></div>
		</div>
	{:else if error}
		<div class="bg-red-50 border border-red-200 text-red-800 px-4 py-4 rounded-lg">
			<p class="font-semibold">Error loading analytics</p>
			<p class="text-sm mt-1">{error}</p>
		</div>
	{:else if analytics}
		<!-- Summary Cards -->
		<div class="grid grid-cols-1 md:grid-cols-3 gap-6">
			<!-- Total Searches -->
			<div class="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
				<div class="flex items-center justify-between">
					<div>
						<p class="text-sm font-medium text-gray-600">Total Searches</p>
						<p class="text-3xl font-bold text-gray-900 mt-2">
							{analytics.total_searches.toLocaleString()}
						</p>
					</div>
					<div class="bg-blue-100 p-3 rounded-full">
						<Search class="w-6 h-6 text-blue-600" />
					</div>
				</div>
			</div>

			<!-- Total Clicks -->
			<div class="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
				<div class="flex items-center justify-between">
					<div>
						<p class="text-sm font-medium text-gray-600">Total Clicks</p>
						<p class="text-3xl font-bold text-gray-900 mt-2">
							{analytics.total_clicks.toLocaleString()}
						</p>
					</div>
					<div class="bg-green-100 p-3 rounded-full">
						<MousePointerClick class="w-6 h-6 text-green-600" />
					</div>
				</div>
			</div>

			<!-- Average CTR -->
			<div class="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
				<div class="flex items-center justify-between">
					<div>
						<p class="text-sm font-medium text-gray-600">Avg Click-Through Rate</p>
						<p class="text-3xl font-bold text-gray-900 mt-2">
							{analytics.avg_click_through_rate.toFixed(1)}%
						</p>
					</div>
					<div class="bg-purple-100 p-3 rounded-full">
						<TrendingUp class="w-6 h-6 text-purple-600" />
					</div>
				</div>
			</div>
		</div>

		<!-- Top Queries Table -->
		<div class="bg-white rounded-lg shadow-sm border border-gray-200">
			<div class="px-6 py-4 border-b border-gray-200">
				<h2 class="text-lg font-semibold text-gray-900">Top Search Queries</h2>
				<p class="text-sm text-gray-600 mt-1">Most popular searches and their performance</p>
			</div>

			{#if analytics.top_queries.length > 0}
				<div class="overflow-x-auto">
					<table class="w-full">
						<thead class="bg-gray-50 border-b border-gray-200">
							<tr>
								<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Query</th>
								<th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">Searches</th>
								<th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">Avg Results</th>
								<th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">Avg Time (ms)</th>
								<th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">Clicks</th>
								<th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">CTR</th>
								<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Last Searched</th>
							</tr>
						</thead>
						<tbody class="bg-white divide-y divide-gray-200">
							{#each analytics.top_queries as query}
								<tr class="hover:bg-gray-50">
									<td class="px-6 py-4 whitespace-nowrap">
										<span class="text-sm font-medium text-gray-900">{query.query}</span>
									</td>
									<td class="px-6 py-4 whitespace-nowrap text-right">
										<span class="text-sm text-gray-900">{query.search_count}</span>
									</td>
									<td class="px-6 py-4 whitespace-nowrap text-right">
										<span class="text-sm text-gray-900">{query.avg_result_count.toFixed(1)}</span>
									</td>
									<td class="px-6 py-4 whitespace-nowrap text-right">
										<span class="text-sm text-gray-900">{query.avg_processing_time_ms.toFixed(0)}</span>
									</td>
									<td class="px-6 py-4 whitespace-nowrap text-right">
										<span class="text-sm text-gray-900">{query.click_count}</span>
									</td>
									<td class="px-6 py-4 whitespace-nowrap text-right">
										<span class="inline-flex px-2 py-1 text-xs font-semibold rounded-full {
											query.click_through_rate >= 50 ? 'bg-green-100 text-green-800' :
											query.click_through_rate >= 20 ? 'bg-yellow-100 text-yellow-800' :
											'bg-red-100 text-red-800'
										}">
											{query.click_through_rate.toFixed(1)}%
										</span>
									</td>
									<td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
										{formatDate(query.last_searched)}
									</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			{:else}
				<div class="px-6 py-12 text-center">
					<p class="text-gray-500">No search queries found for this time period</p>
				</div>
			{/if}
		</div>

		<!-- Zero Result Queries -->
		{#if analytics.zero_result_queries.length > 0}
			<div class="bg-white rounded-lg shadow-sm border border-gray-200">
				<div class="px-6 py-4 border-b border-gray-200">
					<h2 class="text-lg font-semibold text-gray-900">Zero Result Queries</h2>
					<p class="text-sm text-gray-600 mt-1">Recent searches that returned no results</p>
				</div>

				<div class="px-6 py-4">
					<div class="flex flex-wrap gap-2">
						{#each analytics.zero_result_queries as query}
							<span class="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-orange-100 text-orange-800">
								{query}
							</span>
						{/each}
					</div>
				</div>
			</div>
		{/if}
	{/if}
</div>
