<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import StatCard from '$lib/components/StatCard.svelte';
	import Chart from '$lib/components/Chart.svelte';
	import ActivityFeed from '$lib/components/ActivityFeed.svelte';
	import { FileText, Activity, Search, Clock, RefreshCw, Image } from 'lucide-svelte';
	import { api } from '$lib/stores/api';
	import type { IndexStats, HealthResponse } from '$shared/types';
	import type { ChartConfiguration } from 'chart.js';

	let stats: IndexStats | null = $state(null);
	let imageStats: any | null = $state(null);
	let health: HealthResponse | null = $state(null);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let autoRefresh = $state(true);
	let refreshInterval: number | null = null;

	// Simulated activity feed (in production, this would come from the backend)
	let activities = $state([
		{
			id: '1',
			type: 'crawl' as const,
			message: 'Crawled example.com (10 documents indexed)',
			timestamp: new Date(Date.now() - 5 * 60 * 1000).toISOString()
		},
		{
			id: '2',
			type: 'search' as const,
			message: 'Search query: "rust programming" (42 results)',
			timestamp: new Date(Date.now() - 15 * 60 * 1000).toISOString()
		},
		{
			id: '3',
			type: 'index' as const,
			message: 'Index updated successfully',
			timestamp: new Date(Date.now() - 30 * 60 * 1000).toISOString()
		}
	]);

	// Chart configuration for field distribution
	let fieldDistributionChart = $derived<ChartConfiguration>({
		type: 'pie',
		data: {
			labels: stats?.fieldDistribution ? Object.keys(stats.fieldDistribution) : [],
			datasets: [
				{
					label: 'Documents',
					data: stats?.fieldDistribution ? Object.values(stats.fieldDistribution) : [],
					backgroundColor: [
						'#3B82F6',
						'#10B981',
						'#F59E0B',
						'#EF4444',
						'#8B5CF6',
						'#EC4899',
						'#6366F1'
					]
				}
			]
		},
		options: {
			responsive: true,
			maintainAspectRatio: false,
			plugins: {
				legend: {
					position: 'bottom'
				},
				title: {
					display: true,
					text: 'Field Distribution'
				}
			}
		}
	});

	// Chart configuration for document stats (example data)
	let documentStatsChart = $derived<ChartConfiguration>({
		type: 'bar',
		data: {
			labels: ['Total Documents', 'Indexed Fields', 'Active Crawls'],
			datasets: [
				{
					label: 'Count',
					data: [
						stats?.numberOfDocuments || 0,
						stats?.fieldDistribution ? Object.keys(stats.fieldDistribution).length : 0,
						0
					],
					backgroundColor: ['#3B82F6', '#10B981', '#F59E0B']
				}
			]
		},
		options: {
			responsive: true,
			maintainAspectRatio: false,
			plugins: {
				legend: {
					display: false
				},
				title: {
					display: true,
					text: 'System Overview'
				}
			},
			scales: {
				y: {
					beginAtZero: true
				}
			}
		}
	});

	onMount(async () => {
		await loadData();
		startAutoRefresh();
	});

	onDestroy(() => {
		stopAutoRefresh();
	});

	async function loadData() {
		loading = true;
		error = null;

		try {
			const [statsData, imageStatsData, healthData] = await Promise.all([
				api.getStats(),
				api.getImageStats(),
				api.healthCheck()
			]);

			stats = statsData;
			imageStats = imageStatsData;
			health = healthData;
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load dashboard data';
			console.error('Dashboard error:', err);
		} finally {
			loading = false;
		}
	}

	function startAutoRefresh() {
		if (autoRefresh && !refreshInterval) {
			refreshInterval = window.setInterval(loadData, 30000); // Refresh every 30 seconds
		}
	}

	function stopAutoRefresh() {
		if (refreshInterval) {
			clearInterval(refreshInterval);
			refreshInterval = null;
		}
	}

	$effect(() => {
		if (autoRefresh) {
			startAutoRefresh();
		} else {
			stopAutoRefresh();
		}

		return () => stopAutoRefresh();
	});
</script>

<div class="space-y-8">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold text-gray-900">Dashboard</h1>
			<p class="text-gray-600 mt-2">Monitor your search engine performance and statistics</p>
		</div>

		<div class="flex items-center gap-4">
			<!-- Auto-refresh toggle -->
			<label class="flex items-center gap-2 text-sm">
				<input type="checkbox" bind:checked={autoRefresh} class="rounded" />
				<span class="text-gray-700">Auto-refresh (30s)</span>
			</label>

			<!-- Manual refresh button -->
			<button
				onclick={loadData}
				disabled={loading}
				class="flex items-center gap-2 px-4 py-2 bg-white border border-gray-300 rounded-lg hover:bg-gray-50 disabled:opacity-50 transition-colors"
			>
				<RefreshCw class="w-4 h-4 {loading ? 'animate-spin' : ''}" />
				<span>Refresh</span>
			</button>
		</div>
	</div>

	<!-- Error Message -->
	{#if error}
		<div class="bg-red-50 border border-red-200 text-red-800 px-4 py-3 rounded-lg">
			<p class="font-semibold">Error</p>
			<p class="text-sm">{error}</p>
		</div>
	{/if}

	<!-- Stats Grid -->
	<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
		<StatCard
			title="Total Documents"
			value={stats?.numberOfDocuments ?? 0}
			icon={FileText}
			color="blue"
			{loading}
		/>

		<StatCard
			title="Total Images"
			value={imageStats?.numberOfImages ?? 0}
			icon={Image}
			color="purple"
			{loading}
		/>

		<StatCard
			title="Indexing Status"
			value={stats?.isIndexing || imageStats?.isIndexing ? 'Active' : 'Idle'}
			icon={Activity}
			color={stats?.isIndexing || imageStats?.isIndexing ? 'orange' : 'green'}
			{loading}
		/>

		<StatCard
			title="API Status"
			value={health?.status === 'healthy' ? 'Healthy' : 'Unknown'}
			icon={Activity}
			color={health?.status === 'healthy' ? 'green' : 'red'}
			{loading}
		/>
	</div>

	<!-- Charts Row -->
	{#if !loading && stats}
		<div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
			<!-- System Overview Chart -->
			<div class="bg-white rounded-lg shadow p-6">
				<Chart config={documentStatsChart} height={300} />
			</div>

			<!-- Field Distribution Chart -->
			<div class="bg-white rounded-lg shadow p-6">
				<Chart config={fieldDistributionChart} height={300} />
			</div>
		</div>
	{/if}

	<!-- Activity Feed and Quick Actions Row -->
	<div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
		<!-- Recent Activity -->
		<div class="bg-white rounded-lg shadow p-6">
			<h2 class="text-xl font-semibold text-gray-900 mb-4 flex items-center gap-2">
				<Clock class="w-5 h-5" />
				Recent Activity
			</h2>
			<ActivityFeed {activities} maxItems={5} />
		</div>

		<!-- Quick Actions -->
		<div class="bg-white rounded-lg shadow p-6">
			<h2 class="text-xl font-semibold text-gray-900 mb-4">Quick Actions</h2>

			<div class="space-y-3">
				<a
					href="/crawl"
					class="flex items-center gap-3 p-4 border-2 border-gray-200 rounded-lg hover:border-primary hover:bg-blue-50 transition-colors"
				>
					<Activity class="w-6 h-6 text-primary" />
					<div>
						<p class="font-semibold text-gray-900">Start Crawl</p>
						<p class="text-sm text-gray-600">Index new websites</p>
					</div>
				</a>

				<a
					href="/search-test"
					class="flex items-center gap-3 p-4 border-2 border-gray-200 rounded-lg hover:border-primary hover:bg-blue-50 transition-colors"
				>
					<Search class="w-6 h-6 text-primary" />
					<div>
						<p class="font-semibold text-gray-900">Test Search</p>
						<p class="text-sm text-gray-600">Try advanced queries</p>
					</div>
				</a>

				<a
					href="/index"
					class="flex items-center gap-3 p-4 border-2 border-gray-200 rounded-lg hover:border-primary hover:bg-blue-50 transition-colors"
				>
					<FileText class="w-6 h-6 text-primary" />
					<div>
						<p class="font-semibold text-gray-900">Browse Index</p>
						<p class="text-sm text-gray-600">View all documents</p>
					</div>
				</a>
			</div>
		</div>
	</div>

	<!-- System Info -->
	<div class="bg-white rounded-lg shadow p-6">
		<h2 class="text-xl font-semibold text-gray-900 mb-4">System Information</h2>

		<dl class="grid grid-cols-1 md:grid-cols-2 gap-4">
			<div>
				<dt class="text-sm font-medium text-gray-600">API Endpoint</dt>
				<dd class="mt-1 text-sm text-gray-900 font-mono">http://127.0.0.1:3000</dd>
			</div>

			<div>
				<dt class="text-sm font-medium text-gray-600">Last Checked</dt>
				<dd class="mt-1 text-sm text-gray-900">
					{health?.timestamp ? new Date(health.timestamp).toLocaleString() : 'N/A'}
				</dd>
			</div>

			<div>
				<dt class="text-sm font-medium text-gray-600">Search Engine</dt>
				<dd class="mt-1 text-sm text-gray-900">Meilisearch</dd>
			</div>

			<div>
				<dt class="text-sm font-medium text-gray-600">Backend</dt>
				<dd class="mt-1 text-sm text-gray-900">Rust + Axum</dd>
			</div>

			<div>
				<dt class="text-sm font-medium text-gray-600">Auto-refresh</dt>
				<dd class="mt-1 text-sm text-gray-900">{autoRefresh ? 'Enabled (30s)' : 'Disabled'}</dd>
			</div>

			<div>
				<dt class="text-sm font-medium text-gray-600">Dashboard Version</dt>
				<dd class="mt-1 text-sm text-gray-900">1.0.0</dd>
			</div>
		</dl>
	</div>
</div>
