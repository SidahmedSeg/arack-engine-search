<script lang="ts">
	import { cn } from '$lib/utils';
	import { Search } from 'lucide-svelte';
	import { autocomplete } from '$lib/api';
	import type { AutocompleteSuggestion } from '$lib/types';

	interface Props {
		value?: string;
		placeholder?: string;
		onSearch?: (query: string) => void;
		onInput?: (query: string) => void;
		class?: string;
		autofocus?: boolean;
	}

	let {
		value = $bindable(''),
		placeholder = 'Search...',
		onSearch,
		onInput,
		class: className,
		autofocus = false
	}: Props = $props();

	let inputElement: HTMLInputElement;
	let isFocused = $state(false);
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

	function handleInput(e: Event) {
		const target = e.target as HTMLInputElement;
		value = target.value;

		// Call external onInput if provided
		if (onInput) {
			onInput(value);
		}

		// Debounce autocomplete requests
		if (debounceTimer) {
			clearTimeout(debounceTimer);
		}

		debounceTimer = window.setTimeout(() => {
			fetchSuggestions(value);
		}, 300);
	}

	function handleKeydown(event: KeyboardEvent) {
		if (!showSuggestions) {
			if (event.key === 'Enter' && onSearch) {
				onSearch(value);
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
				} else if (onSearch) {
					onSearch(value);
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
		if (onSearch) {
			onSearch(value);
		}
	}

	function handleBlur() {
		isFocused = false;
		// Delay hiding suggestions to allow click events to register
		setTimeout(() => {
			showSuggestions = false;
			selectedIndex = -1;
		}, 200);
	}

	function handleFocus() {
		isFocused = true;
		if (suggestions.length > 0) {
			showSuggestions = true;
		}
	}
</script>

<div class={cn("relative w-full max-w-xl", className?.includes('search-header') ? '' : 'mx-auto')}>
	<div
		class={cn(
			'relative w-full transition-shadow',
			'rounded-3xl',
			isFocused
				? 'shadow-[0_1px_4px_0px_rgba(0,0,0,.08)]'
				: 'shadow-[0_1px_2px_0px_rgba(0,0,0,.05)] hover:shadow-[0_1px_4px_0px_rgba(0,0,0,.08)]',
			className
		)}
	>
		<!-- Search Icon -->
		<div class="absolute left-4 top-1/2 -translate-y-1/2 pointer-events-none">
			<Search class="w-5 h-5 text-[#9aa0a6]" />
		</div>

		<!-- Input Field -->
		<input
			bind:this={inputElement}
			type="text"
			{placeholder}
			{value}
			{autofocus}
			onfocus={handleFocus}
			onblur={handleBlur}
			oninput={handleInput}
			onkeydown={handleKeydown}
			class={cn(
				'w-full h-11 pl-14 pr-4 py-1',
				'rounded-3xl border border-[#dadce0]',
				'text-base text-gray-900',
				'placeholder:text-gray-500',
				'focus:outline-none focus:border-[#dadce0]',
				'transition-colors'
			)}
			aria-label="Search"
		/>

		<!-- Loading Indicator -->
		{#if isLoading}
			<div class="absolute right-4 top-1/2 -translate-y-1/2">
				<div
					class="w-4 h-4 border-2 border-[#0059ff] border-t-transparent rounded-full animate-spin"
				></div>
			</div>
		{/if}
	</div>

	<!-- Autocomplete Suggestions Dropdown -->
	{#if showSuggestions && suggestions.length > 0}
		<div
			class="absolute top-full left-0 right-0 mt-2 bg-white border border-gray-200 rounded-lg shadow-xl z-50 overflow-hidden"
		>
			{#each suggestions as suggestion, index}
				<button
					onclick={() => selectSuggestion(suggestion.query)}
					class={cn(
						'w-full px-4 py-3 text-left hover:bg-gray-50 transition-colors border-b border-gray-100 last:border-b-0',
						index === selectedIndex ? 'bg-blue-50' : ''
					)}
				>
					<div class="flex items-center justify-between">
						<span class="text-gray-900">{suggestion.query}</span>
						{#if suggestion.count > 0}
							<span class="text-xs text-gray-500">{suggestion.count} results</span>
						{/if}
					</div>
				</button>
			{/each}
		</div>
	{/if}
</div>

<style>
	/* Remove autofill background color */
	input:-webkit-autofill,
	input:-webkit-autofill:hover,
	input:-webkit-autofill:focus {
		-webkit-box-shadow: 0 0 0px 1000px white inset;
		box-shadow: 0 0 0px 1000px white inset;
	}
</style>
