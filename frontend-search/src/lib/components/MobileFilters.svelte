<script lang="ts">
	import { Filter, X } from 'lucide-svelte';
	import FiltersPanel from './FiltersPanel.svelte';
	import type { SearchFilters } from '$lib/types';

	interface Props {
		filters: SearchFilters;
		onFiltersChange: (filters: SearchFilters) => void;
		onClearFilters: () => void;
	}

	let { filters, onFiltersChange, onClearFilters }: Props = $props();
	let isOpen = $state(false);

	function toggleDrawer() {
		isOpen = !isOpen;
	}

	function handleFiltersChange(newFilters: SearchFilters) {
		onFiltersChange(newFilters);
		isOpen = false;
	}
</script>

<!-- Mobile Filters Button -->
<button
	onclick={toggleDrawer}
	class="lg:hidden fixed bottom-6 right-6 z-40 p-4 bg-primary text-white rounded-full shadow-lg hover:bg-blue-600 transition-all"
	aria-label="Open filters"
>
	<Filter class="w-6 h-6" />
</button>

<!-- Mobile Drawer Overlay -->
{#if isOpen}
	<div
		class="fixed inset-0 bg-black bg-opacity-50 z-50 lg:hidden transition-opacity"
		onclick={toggleDrawer}
		role="presentation"
	></div>
{/if}

<!-- Mobile Drawer -->
<div
	class="fixed top-0 right-0 bottom-0 w-80 bg-white shadow-2xl z-50 lg:hidden transform transition-transform duration-300 ease-in-out {isOpen
		? 'translate-x-0'
		: 'translate-x-full'}"
>
	<div class="h-full overflow-y-auto">
		<div class="sticky top-0 bg-white border-b px-6 py-4 flex items-center justify-between z-10">
			<div class="flex items-center gap-2">
				<Filter class="w-5 h-5 text-primary" />
				<h3 class="font-semibold text-gray-900">Filters</h3>
			</div>
			<button onclick={toggleDrawer} class="p-2 hover:bg-gray-100 rounded-full transition-colors">
				<X class="w-5 h-5 text-gray-600" />
			</button>
		</div>

		<div class="p-6">
			<FiltersPanel
				{filters}
				onFiltersChange={handleFiltersChange}
				{onClearFilters}
			/>
		</div>
	</div>
</div>
