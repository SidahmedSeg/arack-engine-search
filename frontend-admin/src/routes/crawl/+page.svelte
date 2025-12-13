<script lang="ts">
	import { onMount } from 'svelte';
	import { Activity, Plus, Loader2, CheckCircle, AlertCircle, Clock, CheckCircle2, XCircle } from 'lucide-svelte';
	import { api } from '$lib/stores/api';
	import type { CrawlResponse } from '$shared/types';
	import { formatDate, formatNumber } from '$shared/utils';

	let urls = $state('');
	let maxDepth = $state(1);
	let loading = $state(false);
	let success = $state<CrawlResponse | null>(null);
	let error = $state<string | null>(null);

	// Crawl history state
	let historyLoading = $state(true);
	let historyError = $state<string | null>(null);
	let history: any[] = $state([]);
	let totalHistory = $state(0);
	let historyPage = $state(0);
	let historyPageSize = $state(10);

	onMount(async () => {
		await loadHistory();
	});

	async function loadHistory() {
		historyLoading = true;
		historyError = null;

		try {
			const response = await api.getCrawlHistory(historyPageSize, historyPage * historyPageSize);
			history = response.history;
			totalHistory = response.total;
		} catch (err) {
			historyError = err instanceof Error ? err.message : 'Failed to load history';
			console.error('History error:', err);
		} finally {
			historyLoading = false;
		}
	}

	function nextHistoryPage() {
		if ((historyPage + 1) * historyPageSize < totalHistory) {
			historyPage++;
			loadHistory();
		}
	}

	function prevHistoryPage() {
		if (historyPage > 0) {
			historyPage--;
			loadHistory();
		}
	}

	async function handleSubmit(e: Event) {
		e.preventDefault();

		loading = true;
		success = null;
		error = null;

		// Parse URLs
		const urlList = urls
			.split('\n')
			.map((u) => u.trim())
			.filter((u) => u.length > 0);

		if (urlList.length === 0) {
			error = 'Please enter at least one URL';
			loading = false;
			return;
		}

		try {
			const response = await api.startCrawl({
				urls: urlList,
				max_depth: maxDepth
			});

			success = response;

			// Clear form
			urls = '';
			maxDepth = 1;

			// Reload history to show the new crawl job
			await loadHistory();
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to start crawl';
			console.error('Crawl error:', err);
		} finally {
			loading = false;
		}
	}
</script>

<div class="space-y-8">
	<!-- Header -->
	<div>
		<h1 class="text-3xl font-bold text-gray-900 flex items-center gap-3">
			<Activity class="w-8 h-8 text-primary" />
			Crawl Management
		</h1>
		<p class="text-gray-600 mt-2">Start new crawl jobs to index websites</p>
	</div>

	<!-- Success Message -->
	{#if success}
		<div class="bg-green-50 border border-green-200 text-green-800 px-4 py-4 rounded-lg">
			<div class="flex items-start gap-3">
				<CheckCircle class="w-5 h-5 mt-0.5 flex-shrink-0" />
				<div class="flex-1">
					<p class="font-semibold">Crawl Completed Successfully!</p>
					<p class="text-sm mt-1">{success.message}</p>
					<p class="text-sm mt-1">
						<span class="font-medium">Documents Indexed:</span>
						{success.documents_indexed}
					</p>
					<div class="mt-2">
						<p class="text-sm font-medium">URLs Crawled:</p>
						<ul class="list-disc list-inside text-sm mt-1">
							{#each success.urls as url}
								<li>{url}</li>
							{/each}
						</ul>
					</div>
				</div>
			</div>
		</div>
	{/if}

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

	<!-- Start Crawl Form -->
	<div class="bg-white rounded-lg shadow">
		<div class="px-6 py-4 border-b border-gray-200">
			<h2 class="text-xl font-semibold text-gray-900 flex items-center gap-2">
				<Plus class="w-5 h-5" />
				Start New Crawl
			</h2>
		</div>

		<form onsubmit={handleSubmit} class="p-6 space-y-6">
			<!-- URLs Input -->
			<div>
				<label for="urls" class="block text-sm font-medium text-gray-700 mb-2">
					URLs to Crawl
					<span class="text-red-500">*</span>
				</label>
				<textarea
					id="urls"
					bind:value={urls}
					rows="6"
					placeholder="https://example.com&#10;https://example.org&#10;https://example.net"
					class="w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary focus:border-transparent font-mono text-sm"
					required
					disabled={loading}
				></textarea>
				<p class="mt-2 text-sm text-gray-600">Enter one URL per line</p>
			</div>

			<!-- Max Depth -->
			<div>
				<label for="maxDepth" class="block text-sm font-medium text-gray-700 mb-2">
					Maximum Crawl Depth
				</label>
				<div class="flex items-center gap-4">
					<input
						id="maxDepth"
						type="range"
						bind:value={maxDepth}
						min="1"
						max="5"
						class="flex-1"
						disabled={loading}
					/>
					<span
						class="text-lg font-semibold text-gray-900 bg-gray-100 px-4 py-2 rounded-lg min-w-[60px] text-center"
					>
						{maxDepth}
					</span>
				</div>
				<p class="mt-2 text-sm text-gray-600">
					Depth {maxDepth}: Crawl up to {maxDepth} level{maxDepth > 1 ? 's' : ''} deep
				</p>
			</div>

			<!-- Submit Button -->
			<div class="flex gap-4">
				<button
					type="submit"
					disabled={loading}
					class="flex items-center gap-2 px-6 py-3 bg-primary text-white rounded-lg hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors font-medium"
				>
					{#if loading}
						<Loader2 class="w-5 h-5 animate-spin" />
						<span>Crawling...</span>
					{:else}
						<Activity class="w-5 h-5" />
						<span>Start Crawl</span>
					{/if}
				</button>

				{#if loading}
					<button
						type="button"
						disabled
						class="px-6 py-3 border border-gray-300 text-gray-700 rounded-lg opacity-50 cursor-not-allowed"
					>
						Cancel
					</button>
				{/if}
			</div>
		</form>
	</div>

	<!-- Information Card -->
	<div class="bg-blue-50 border border-blue-200 text-blue-900 px-4 py-4 rounded-lg">
		<p class="font-semibold flex items-center gap-2">
			<Activity class="w-5 h-5" />
			How Crawling Works
		</p>
		<ul class="list-disc list-inside text-sm mt-2 space-y-1 ml-7">
			<li>The crawler respects robots.txt files</li>
			<li>Each URL is fetched and its content is extracted</li>
			<li>Links within pages are followed up to the specified depth</li>
			<li>Extracted content is automatically indexed in Meilisearch</li>
			<li>Crawl time depends on the number of pages and website speed</li>
		</ul>
	</div>

	<!-- Crawl History -->
	<div class="bg-white rounded-lg shadow">
		<div class="px-6 py-4 border-b border-gray-200">
			<h2 class="text-xl font-semibold text-gray-900 flex items-center gap-2">
				<Clock class="w-5 h-5" />
				Crawl History
			</h2>
		</div>

		<div class="p-6">
			{#if historyError}
				<div class="bg-red-50 border border-red-200 text-red-800 px-4 py-3 rounded-lg mb-4">
					<p class="font-semibold">Error</p>
					<p class="text-sm">{historyError}</p>
				</div>
			{/if}

			{#if historyLoading}
				<div class="text-center py-12">
					<Loader2 class="w-8 h-8 animate-spin mx-auto text-primary" />
					<p class="text-gray-600 mt-4">Loading crawl history...</p>
				</div>
			{:else if history.length === 0}
				<div class="text-center py-12">
					<Clock class="w-12 h-12 mx-auto text-gray-400" />
					<p class="text-gray-600 mt-4">No crawl history yet</p>
					<p class="text-sm text-gray-500 mt-2">Start a crawl to see history here</p>
				</div>
			{:else}
				<!-- Results Info -->
				<div class="flex items-center justify-between text-sm text-gray-600 mb-4">
					<p>
						Showing {historyPage * historyPageSize + 1} - {Math.min(
							(historyPage + 1) * historyPageSize,
							totalHistory
						)}
						of {formatNumber(totalHistory)} crawl jobs
					</p>

					<!-- Pagination -->
					<div class="flex gap-2">
						<button
							onclick={prevHistoryPage}
							disabled={historyPage === 0 || historyLoading}
							class="px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
						>
							Previous
						</button>
						<button
							onclick={nextHistoryPage}
							disabled={(historyPage + 1) * historyPageSize >= totalHistory || historyLoading}
							class="px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
						>
							Next
						</button>
					</div>
				</div>

				<!-- History Table -->
				<div class="overflow-x-auto">
					<table class="w-full">
						<thead class="bg-gray-50 border-b border-gray-200">
							<tr>
								<th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
									Status
								</th>
								<th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
									URLs
								</th>
								<th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
									Pages
								</th>
								<th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
									Started
								</th>
								<th class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
									Duration
								</th>
							</tr>
						</thead>
						<tbody class="bg-white divide-y divide-gray-200">
							{#each history as job}
								<tr class="hover:bg-gray-50">
									<td class="px-4 py-4">
										{#if job.status === 'completed'}
											<span class="inline-flex items-center gap-1.5 px-2.5 py-1 bg-green-100 text-green-800 text-xs font-medium rounded-full">
												<CheckCircle2 class="w-3.5 h-3.5" />
												Completed
											</span>
										{:else if job.status === 'failed'}
											<span class="inline-flex items-center gap-1.5 px-2.5 py-1 bg-red-100 text-red-800 text-xs font-medium rounded-full">
												<XCircle class="w-3.5 h-3.5" />
												Failed
											</span>
										{:else}
											<span class="inline-flex items-center gap-1.5 px-2.5 py-1 bg-blue-100 text-blue-800 text-xs font-medium rounded-full">
												<Loader2 class="w-3.5 h-3.5 animate-spin" />
												{job.status}
											</span>
										{/if}
									</td>
									<td class="px-4 py-4">
										<div class="max-w-xs">
											{#each job.urls as url, i}
												{#if i < 2}
													<p class="text-sm text-gray-900 truncate">{url}</p>
												{/if}
											{/each}
											{#if job.urls.length > 2}
												<p class="text-xs text-gray-500 mt-1">
													+{job.urls.length - 2} more
												</p>
											{/if}
										</div>
									</td>
									<td class="px-4 py-4 text-sm text-gray-900">
										<div>
											<p>{formatNumber(job.pages_crawled)} crawled</p>
											<p class="text-xs text-gray-500">{formatNumber(job.pages_indexed)} indexed</p>
										</div>
									</td>
									<td class="px-4 py-4 text-sm text-gray-600">
										{formatDate(job.started_at)}
									</td>
									<td class="px-4 py-4 text-sm text-gray-600">
										{#if job.completed_at}
											{Math.round(
												(new Date(job.completed_at).getTime() -
													new Date(job.started_at).getTime()) /
													1000
											)}s
										{:else}
											-
										{/if}
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
