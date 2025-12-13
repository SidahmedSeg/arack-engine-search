<script lang="ts">
	import { Filter, X, ChevronDown, ChevronUp } from 'lucide-svelte';
	import type { SearchFilters } from '$lib/types';

	interface Props {
		filters: SearchFilters;
		onFiltersChange: (filters: SearchFilters) => void;
		onClearFilters: () => void;
	}

	let { filters, onFiltersChange, onClearFilters }: Props = $props();

	let minWordCount = $state('');
	let maxWordCount = $state('');
	let sortBy = $state<'relevance' | 'date' | 'word_count'>('relevance');
	let order = $state<'asc' | 'desc'>('desc');
	let showWordCount = $state(false);
	let showSorting = $state(true);

	// Sync local state with filters prop
	$effect(() => {
		minWordCount = filters.min_word_count?.toString() || '';
		maxWordCount = filters.max_word_count?.toString() || '';
		sortBy = filters.sort_by || 'relevance';
		order = filters.order || 'desc';
	});

	function applyFilters() {
		const newFilters: SearchFilters = {
			...filters,
			min_word_count: minWordCount ? parseInt(minWordCount) : undefined,
			max_word_count: maxWordCount ? parseInt(maxWordCount) : undefined,
			sort_by: sortBy as 'relevance' | 'date' | 'word_count',
			order: order as 'asc' | 'desc'
		};
		onFiltersChange(newFilters);
	}

	function handleClearFilters() {
		minWordCount = '';
		maxWordCount = '';
		sortBy = 'relevance';
		order = 'desc';
		onClearFilters();
	}
</script>

<div class="bg-white rounded-lg shadow-lg p-6 sticky top-24">
	<div class="flex items-center justify-between mb-6">
		<div class="flex items-center gap-2">
			<Filter class="w-5 h-5 text-primary" />
			<h3 class="font-semibold text-gray-900">Filters</h3>
		</div>
		<button
			onclick={handleClearFilters}
			class="text-sm text-gray-600 hover:text-primary transition-colors"
		>
			Clear all
		</button>
	</div>

	<!-- Sorting Section -->
	<div class="mb-6">
		<button
			onclick={() => (showSorting = !showSorting)}
			class="flex items-center justify-between w-full mb-3"
		>
			<span class="font-medium text-gray-900">Sort By</span>
			{#if showSorting}
				<ChevronUp class="w-4 h-4 text-gray-500" />
			{:else}
				<ChevronDown class="w-4 h-4 text-gray-500" />
			{/if}
		</button>

		{#if showSorting}
			<div class="space-y-3 pl-1">
				<label class="flex items-center gap-2 cursor-pointer">
					<input
						type="radio"
						name="sort"
						value="relevance"
						bind:group={sortBy}
						onchange={applyFilters}
						class="w-4 h-4 text-primary focus:ring-primary"
					/>
					<span class="text-sm text-gray-700">Relevance</span>
				</label>
				<label class="flex items-center gap-2 cursor-pointer">
					<input
						type="radio"
						name="sort"
						value="date"
						bind:group={sortBy}
						onchange={applyFilters}
						class="w-4 h-4 text-primary focus:ring-primary"
					/>
					<span class="text-sm text-gray-700">Date</span>
				</label>
				<label class="flex items-center gap-2 cursor-pointer">
					<input
						type="radio"
						name="sort"
						value="word_count"
						bind:group={sortBy}
						onchange={applyFilters}
						class="w-4 h-4 text-primary focus:ring-primary"
					/>
					<span class="text-sm text-gray-700">Word Count</span>
				</label>

				<!-- Order Selection -->
				<div class="mt-3 pt-3 border-t border-gray-200">
					<label class="flex items-center gap-2 cursor-pointer mb-2">
						<input
							type="radio"
							name="order"
							value="desc"
							bind:group={order}
							onchange={applyFilters}
							class="w-4 h-4 text-primary focus:ring-primary"
						/>
						<span class="text-sm text-gray-700">Descending</span>
					</label>
					<label class="flex items-center gap-2 cursor-pointer">
						<input
							type="radio"
							name="order"
							value="asc"
							bind:group={order}
							onchange={applyFilters}
							class="w-4 h-4 text-primary focus:ring-primary"
						/>
						<span class="text-sm text-gray-700">Ascending</span>
					</label>
				</div>
			</div>
		{/if}
	</div>

	<!-- Word Count Section -->
	<div class="mb-6">
		<button
			onclick={() => (showWordCount = !showWordCount)}
			class="flex items-center justify-between w-full mb-3"
		>
			<span class="font-medium text-gray-900">Word Count</span>
			{#if showWordCount}
				<ChevronUp class="w-4 h-4 text-gray-500" />
			{:else}
				<ChevronDown class="w-4 h-4 text-gray-500" />
			{/if}
		</button>

		{#if showWordCount}
			<div class="space-y-3 pl-1">
				<div>
					<label for="min-words" class="block text-sm text-gray-700 mb-1">Minimum</label>
					<input
						id="min-words"
						type="number"
						bind:value={minWordCount}
						placeholder="e.g., 100"
						class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent"
					/>
				</div>
				<div>
					<label for="max-words" class="block text-sm text-gray-700 mb-1">Maximum</label>
					<input
						id="max-words"
						type="number"
						bind:value={maxWordCount}
						placeholder="e.g., 5000"
						class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary focus:border-transparent"
					/>
				</div>
				<button
					onclick={applyFilters}
					class="w-full py-2 bg-primary text-white rounded-lg hover:bg-blue-600 transition-colors font-medium"
				>
					Apply
				</button>
			</div>
		{/if}
	</div>

	<!-- Quick Filters -->
	<div class="border-t border-gray-200 pt-4">
		<p class="text-sm font-medium text-gray-900 mb-3">Quick Filters</p>
		<div class="flex flex-wrap gap-2">
			<button
				onclick={() => {
					minWordCount = '';
					maxWordCount = '500';
					applyFilters();
				}}
				class="px-3 py-1 text-sm bg-gray-100 hover:bg-gray-200 rounded-full transition-colors"
			>
				Short articles
			</button>
			<button
				onclick={() => {
					minWordCount = '500';
					maxWordCount = '2000';
					applyFilters();
				}}
				class="px-3 py-1 text-sm bg-gray-100 hover:bg-gray-200 rounded-full transition-colors"
			>
				Medium articles
			</button>
			<button
				onclick={() => {
					minWordCount = '2000';
					maxWordCount = '';
					applyFilters();
				}}
				class="px-3 py-1 text-sm bg-gray-100 hover:bg-gray-200 rounded-full transition-colors"
			>
				Long articles
			</button>
		</div>
	</div>
</div>
