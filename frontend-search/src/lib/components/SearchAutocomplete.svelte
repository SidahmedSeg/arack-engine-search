<script lang="ts">
	import { Search } from 'lucide-svelte';
	import { autocomplete } from '$lib/api';
	import type { AutocompleteSuggestion } from '$lib/types';

	let {
		value = $bindable(''),
		onSearch,
		placeholder = 'Search for anything...',
		class: className = ''
	}: {
		value?: string;
		onSearch: () => void;
		placeholder?: string;
		class?: string;
	} = $props();

	let suggestions = $state<AutocompleteSuggestion[]>([]);
	let showSuggestions = $state(false);
	let selectedIndex = $state(-1);
	let isLoading = $state(false);
	let debounceTimer: number | null = null;

	async function fetchSuggestions(query: string) {
		if (!query.trim() || query.length < 2) {
			suggestions = [];
			showSuggestions = false;
			return;
		}

		isLoading = true;
		try {
			const response = await autocomplete(query, 5);
			suggestions = response.suggestions;
			showSuggestions = suggestions.length > 0;
			selectedIndex = -1;
		} catch (error) {
			console.error('Autocomplete error:', error);
			suggestions = [];
			showSuggestions = false;
		} finally {
			isLoading = false;
		}
	}

	function handleInput(event: Event) {
		const target = event.target as HTMLInputElement;
		value = target.value;

		// Debounce autocomplete requests
		if (debounceTimer) {
			clearTimeout(debounceTimer);
		}

		debounceTimer = window.setTimeout(() => {
			fetchSuggestions(value);
		}, 300);
	}

	function handleKeyDown(event: KeyboardEvent) {
		if (!showSuggestions) {
			if (event.key === 'Enter') {
				onSearch();
			}
			return;
		}

		switch (event.key) {
			case 'ArrowDown':
				event.preventDefault();
				selectedIndex = Math.min(selectedIndex + 1, suggestions.length - 1);
				break;
			case 'ArrowUp':
				event.preventDefault();
				selectedIndex = Math.max(selectedIndex - 1, -1);
				break;
			case 'Enter':
				event.preventDefault();
				if (selectedIndex >= 0) {
					selectSuggestion(suggestions[selectedIndex].query);
				} else {
					onSearch();
				}
				break;
			case 'Escape':
				showSuggestions = false;
				selectedIndex = -1;
				break;
		}
	}

	function selectSuggestion(query: string) {
		value = query;
		suggestions = [];
		showSuggestions = false;
		selectedIndex = -1;
		onSearch();
	}

	function handleBlur() {
		// Delay hiding suggestions to allow click events to register
		setTimeout(() => {
			showSuggestions = false;
			selectedIndex = -1;
		}, 200);
	}

	function handleFocus() {
		if (suggestions.length > 0) {
			showSuggestions = true;
		}
	}
</script>

<div class="relative {className}">
	<input
		type="text"
		bind:value
		oninput={handleInput}
		onkeydown={handleKeyDown}
		onblur={handleBlur}
		onfocus={handleFocus}
		{placeholder}
		class="w-full px-6 py-4 pr-14 text-lg border-2 border-gray-200 rounded-full shadow-lg focus:outline-none focus:border-primary focus:ring-4 focus:ring-blue-100 transition-all"
	/>
	<button
		onclick={onSearch}
		class="absolute right-2 top-1/2 -translate-y-1/2 p-3 bg-primary text-white rounded-full hover:bg-blue-600 transition-colors"
		aria-label="Search"
	>
		<Search class="w-5 h-5" />
	</button>

	{#if showSuggestions && suggestions.length > 0}
		<div
			class="absolute top-full left-0 right-0 mt-2 bg-white border border-gray-200 rounded-lg shadow-xl z-50 overflow-hidden"
		>
			{#each suggestions as suggestion, index}
				<button
					onclick={() => selectSuggestion(suggestion.query)}
					class="w-full px-6 py-3 text-left hover:bg-gray-50 transition-colors border-b border-gray-100 last:border-b-0 {index ===
					selectedIndex
						? 'bg-blue-50'
						: ''}"
				>
					<div class="flex items-center justify-between">
						<span class="text-gray-900 font-medium">{suggestion.query}</span>
						{#if suggestion.count > 0}
							<span class="text-xs text-gray-500">{suggestion.count} results</span>
						{/if}
					</div>
				</button>
			{/each}
		</div>
	{/if}

	{#if isLoading}
		<div class="absolute right-16 top-1/2 -translate-y-1/2">
			<div class="w-5 h-5 border-2 border-primary border-t-transparent rounded-full animate-spin"></div>
		</div>
	{/if}
</div>
