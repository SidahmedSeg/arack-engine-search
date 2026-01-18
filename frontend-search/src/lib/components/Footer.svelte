<script lang="ts">
	import { onMount } from 'svelte';
	import { locationStore } from '$lib/stores/location.svelte';
	import { MapPin } from 'lucide-svelte';

	onMount(() => {
		// Initialize location detection on component mount
		locationStore.init();
	});
</script>

<footer class="w-full bg-gray-100">
	<div class="px-3 md:px-5 py-3 md:py-3">
		<div class="flex flex-col md:flex-row md:justify-between md:items-center gap-3 md:gap-0 text-sm text-gray-600">
			<!-- Left Side: Location, About, How to use arak -->
			<div class="flex flex-col sm:flex-row items-start sm:items-center gap-3 sm:gap-6">
				<!-- Dynamic Location Display -->
				<div class="flex items-center gap-1.5">
					<MapPin class="w-3.5 h-3.5" />
					<span class="text-gray-700 font-medium">{locationStore.displayText}</span>
				</div>

				<div class="flex items-center gap-4 sm:gap-6">
					<button class="hover:underline" onclick={() => (window.location.href = '/about')}>
						About
					</button>
					<button class="hover:underline" onclick={() => (window.location.href = '/how-to-use')}>
						How to use arak
					</button>
				</div>
			</div>

			<!-- Right Side: Settings, Privacy, Usage conditions (R to L) -->
			<div class="flex items-center gap-4 sm:gap-6">
				<button class="hover:underline" onclick={() => (window.location.href = '/usage-conditions')}>
					Usage conditions
				</button>
				<button class="hover:underline" onclick={() => (window.location.href = '/privacy')}>
					Privacy
				</button>
				<button class="hover:underline" onclick={() => (window.location.href = '/settings')}>
					Settings
				</button>
			</div>
		</div>
	</div>
</footer>
