<script lang="ts">
	import { Activity, RefreshCw, AlertCircle, BarChart3, Timer, Shield, Filter, Globe } from 'lucide-svelte';
	import { api } from '$lib/stores/api';
	import { onMount } from 'svelte';

	let metrics = $state<any>(null);
	let domains = $state<any>(null);
	let scheduler = $state<any>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let lastUpdated = $state<Date | null>(null);
	let autoRefresh = $state(false);
	let refreshInterval: number | null = null;

	async function loadMetrics() {
		try {
			error = null;
			const [metricsData, domainsData, schedulerData] = await Promise.all([
				api.getCrawlerMetrics(),
				api.getCrawlerDomains(),
				api.getCrawlerScheduler()
			]);

			metrics = metricsData;
			domains = domainsData;
			scheduler = schedulerData;
			lastUpdated = new Date();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load crawler metrics';
			console.error('Metrics error:', err);
		} finally {
			loading = false;
		}
	}

	function toggleAutoRefresh() {
		autoRefresh = !autoRefresh;
		if (autoRefresh) {
			refreshInterval = window.setInterval(loadMetrics, 5000); // Refresh every 5 seconds
		} else if (refreshInterval) {
			clearInterval(refreshInterval);
			refreshInterval = null;
		}
	}

	onMount(() => {
		loadMetrics();
		return () => {
			if (refreshInterval) {
				clearInterval(refreshInterval);
			}
		};
	});
</script>

<div class="space-y-8">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold text-gray-900 flex items-center gap-3">
				<BarChart3 class="w-8 h-8 text-primary" />
				Crawler Metrics
			</h1>
			<p class="text-gray-600 mt-2">Monitor crawler performance and health</p>
		</div>
		<div class="flex items-center gap-3">
			{#if lastUpdated}
				<span class="text-sm text-gray-600">
					Updated: {lastUpdated.toLocaleTimeString()}
				</span>
			{/if}
			<button
				onclick={toggleAutoRefresh}
				class="px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50 transition-colors {autoRefresh
					? 'bg-green-50 border-green-300 text-green-700'
					: ''}"
			>
				{autoRefresh ? 'Auto-refresh ON' : 'Auto-refresh OFF'}
			</button>
			<button
				onclick={() => loadMetrics()}
				disabled={loading}
				class="flex items-center gap-2 px-4 py-2 bg-primary text-white rounded-lg hover:bg-blue-600 disabled:opacity-50 transition-colors"
			>
				<RefreshCw class="w-4 h-4 {loading ? 'animate-spin' : ''}" />
				Refresh
			</button>
		</div>
	</div>

	<!-- Error Message -->
	{#if error}
		<div class="bg-red-50 border border-red-200 text-red-800 px-4 py-4 rounded-lg">
			<div class="flex items-start gap-3">
				<AlertCircle class="w-5 h-5 mt-0.5 flex-shrink-0" />
				<div>
					<p class="font-semibold">Error</p>
					<p class="text-sm mt-1">{error}</p>
				</div>
			</div>
		</div>
	{/if}

	{#if loading && !metrics}
		<div class="flex items-center justify-center py-12">
			<RefreshCw class="w-8 h-8 animate-spin text-primary" />
		</div>
	{:else if metrics}
		<!-- Rate Limiter Stats -->
		<div class="bg-white rounded-lg shadow">
			<div class="px-6 py-4 border-b border-gray-200">
				<h2 class="text-xl font-semibold text-gray-900 flex items-center gap-2">
					<Timer class="w-5 h-5 text-blue-600" />
					Rate Limiter
				</h2>
			</div>
			<div class="p-6 grid grid-cols-1 md:grid-cols-3 gap-4">
				<div class="bg-blue-50 p-4 rounded-lg">
					<p class="text-sm text-blue-600 font-medium">Total Requests</p>
					<p class="text-2xl font-bold text-blue-900 mt-1">
						{metrics.rate_limiter?.total_requests || 0}
					</p>
				</div>
				<div class="bg-green-50 p-4 rounded-lg">
					<p class="text-sm text-green-600 font-medium">Allowed</p>
					<p class="text-2xl font-bold text-green-900 mt-1">
						{metrics.rate_limiter?.requests_allowed || 0}
					</p>
				</div>
				<div class="bg-red-50 p-4 rounded-lg">
					<p class="text-sm text-red-600 font-medium">Rejected</p>
					<p class="text-2xl font-bold text-red-900 mt-1">
						{metrics.rate_limiter?.requests_rejected || 0}
					</p>
				</div>
			</div>
		</div>

		<!-- Circuit Breaker Stats -->
		<div class="bg-white rounded-lg shadow">
			<div class="px-6 py-4 border-b border-gray-200">
				<h2 class="text-xl font-semibold text-gray-900 flex items-center gap-2">
					<Shield class="w-5 h-5 text-purple-600" />
					Circuit Breaker
				</h2>
			</div>
			<div class="p-6">
				<div class="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
					<div class="bg-gray-50 p-4 rounded-lg">
						<p class="text-sm text-gray-600 font-medium">Total Circuits</p>
						<p class="text-2xl font-bold text-gray-900 mt-1">
							{metrics.circuit_breaker?.total_circuits || 0}
						</p>
					</div>
					<div class="bg-green-50 p-4 rounded-lg">
						<p class="text-sm text-green-600 font-medium">Closed</p>
						<p class="text-2xl font-bold text-green-900 mt-1">
							{metrics.circuit_breaker?.closed_circuits || 0}
						</p>
					</div>
					<div class="bg-yellow-50 p-4 rounded-lg">
						<p class="text-sm text-yellow-600 font-medium">Half-Open</p>
						<p class="text-2xl font-bold text-yellow-900 mt-1">
							{metrics.circuit_breaker?.half_open_circuits || 0}
						</p>
					</div>
					<div class="bg-red-50 p-4 rounded-lg">
						<p class="text-sm text-red-600 font-medium">Open</p>
						<p class="text-2xl font-bold text-red-900 mt-1">
							{metrics.circuit_breaker?.open_circuits || 0}
						</p>
					</div>
				</div>

				<!-- Domains Table -->
				{#if domains?.domains && domains.domains.length > 0}
					<div class="overflow-x-auto">
						<table class="min-w-full divide-y divide-gray-200">
							<thead class="bg-gray-50">
								<tr>
									<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
										Domain
									</th>
									<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
										State
									</th>
									<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
										Failures
									</th>
									<th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
										Successes
									</th>
								</tr>
							</thead>
							<tbody class="bg-white divide-y divide-gray-200">
								{#each domains.domains as [domain, stats]}
									<tr>
										<td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
											{domain}
										</td>
										<td class="px-6 py-4 whitespace-nowrap text-sm">
											<span
												class="px-2 py-1 rounded-full text-xs font-medium {stats.state ===
												'Closed'
													? 'bg-green-100 text-green-800'
													: stats.state === 'Open'
														? 'bg-red-100 text-red-800'
														: 'bg-yellow-100 text-yellow-800'}"
											>
												{stats.state}
											</span>
										</td>
										<td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
											{stats.failure_count} / {stats.total_failures}
										</td>
										<td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
											{stats.success_count} / {stats.total_successes}
										</td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				{:else}
					<p class="text-gray-600 text-center py-4">No domains tracked yet</p>
				{/if}
			</div>
		</div>

		<!-- Scheduler Stats -->
		<div class="bg-white rounded-lg shadow">
			<div class="px-6 py-4 border-b border-gray-200">
				<h2 class="text-xl font-semibold text-gray-900 flex items-center gap-2">
					<Activity class="w-5 h-5 text-indigo-600" />
					Scheduler
				</h2>
			</div>
			<div class="p-6">
				<div class="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
					<div class="bg-indigo-50 p-4 rounded-lg">
						<p class="text-sm text-indigo-600 font-medium">Total Tasks</p>
						<p class="text-2xl font-bold text-indigo-900 mt-1">
							{scheduler?.stats?.total_tasks || 0}
						</p>
					</div>
					<div class="bg-green-50 p-4 rounded-lg">
						<p class="text-sm text-green-600 font-medium">Due Tasks</p>
						<p class="text-2xl font-bold text-green-900 mt-1">
							{scheduler?.stats?.due_tasks || 0}
						</p>
					</div>
					<div class="bg-orange-50 p-4 rounded-lg">
						<p class="text-sm text-orange-600 font-medium">Overdue</p>
						<p class="text-2xl font-bold text-orange-900 mt-1">
							{scheduler?.stats?.overdue_tasks || 0}
						</p>
					</div>
					<div class="bg-blue-50 p-4 rounded-lg">
						<p class="text-sm text-blue-600 font-medium">Avg Freshness</p>
						<p class="text-2xl font-bold text-blue-900 mt-1">
							{((scheduler?.stats?.average_freshness || 0) * 100).toFixed(1)}%
						</p>
					</div>
				</div>

				<!-- Tasks Preview -->
				{#if scheduler?.tasks && scheduler.tasks.length > 0}
					<div>
						<h3 class="text-sm font-medium text-gray-700 mb-3">Upcoming Tasks (Top 10)</h3>
						<div class="space-y-2">
							{#each scheduler.tasks.slice(0, 10) as task}
								<div class="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
									<div class="flex-1">
										<p class="text-sm font-medium text-gray-900 truncate">{task.url}</p>
										<div class="flex items-center gap-4 mt-1">
											<span class="text-xs text-gray-500">
												Priority: {task.priority}
											</span>
											<span class="text-xs text-gray-500">
												Freshness: {(task.freshness_score * 100).toFixed(1)}%
											</span>
											<span class="text-xs text-gray-500">
												{task.frequency}
											</span>
										</div>
									</div>
								</div>
							{/each}
						</div>
					</div>
				{:else}
					<p class="text-gray-600 text-center py-4">No scheduled tasks</p>
				{/if}
			</div>
		</div>

		<!-- Filter Stats -->
		<div class="bg-white rounded-lg shadow">
			<div class="px-6 py-4 border-b border-gray-200">
				<h2 class="text-xl font-semibold text-gray-900 flex items-center gap-2">
					<Filter class="w-5 h-5 text-orange-600" />
					Content Filters
				</h2>
			</div>
			<div class="p-6 grid grid-cols-1 md:grid-cols-3 gap-4">
				<div class="bg-blue-50 p-4 rounded-lg">
					<p class="text-sm text-blue-600 font-medium">Total Checked</p>
					<p class="text-2xl font-bold text-blue-900 mt-1">
						{metrics.filters?.total_urls_checked || 0}
					</p>
				</div>
				<div class="bg-green-50 p-4 rounded-lg">
					<p class="text-sm text-green-600 font-medium">Allowed</p>
					<p class="text-2xl font-bold text-green-900 mt-1">
						{metrics.filters?.urls_allowed || 0}
					</p>
				</div>
				<div class="bg-red-50 p-4 rounded-lg">
					<p class="text-sm text-red-600 font-medium">Blocked</p>
					<p class="text-2xl font-bold text-red-900 mt-1">
						{metrics.filters?.urls_blocked || 0}
					</p>
				</div>
			</div>
		</div>

		<!-- Robots.txt Stats -->
		<div class="bg-white rounded-lg shadow">
			<div class="px-6 py-4 border-b border-gray-200">
				<h2 class="text-xl font-semibold text-gray-900 flex items-center gap-2">
					<Globe class="w-5 h-5 text-teal-600" />
					Robots.txt
				</h2>
			</div>
			<div class="p-6">
				<div class="bg-teal-50 p-4 rounded-lg">
					<p class="text-sm text-teal-600 font-medium">Cached Domains</p>
					<p class="text-2xl font-bold text-teal-900 mt-1">
						{metrics.robots?.cached_domains || 0}
					</p>
				</div>
			</div>
		</div>

		<!-- Politeness Stats -->
		{#if metrics.politeness}
			<div class="bg-white rounded-lg shadow">
				<div class="px-6 py-4 border-b border-gray-200">
					<h2 class="text-xl font-semibold text-gray-900 flex items-center gap-2">
						<Timer class="w-5 h-5 text-pink-600" />
						Politeness
					</h2>
				</div>
				<div class="p-6 grid grid-cols-1 md:grid-cols-2 gap-4">
					<div class="bg-pink-50 p-4 rounded-lg">
						<p class="text-sm text-pink-600 font-medium">Total Delays</p>
						<p class="text-2xl font-bold text-pink-900 mt-1">
							{metrics.politeness?.total_delays || 0}
						</p>
					</div>
					<div class="bg-purple-50 p-4 rounded-lg">
						<p class="text-sm text-purple-600 font-medium">Avg Delay (ms)</p>
						<p class="text-2xl font-bold text-purple-900 mt-1">
							{(metrics.politeness?.average_delay_ms || 0).toFixed(2)}
						</p>
					</div>
				</div>
			</div>
		{/if}
	{/if}
</div>
