<script lang="ts">
	import type { ComponentType } from 'svelte';

	interface Props {
		title: string;
		value: string | number;
		icon: ComponentType;
		color?: 'blue' | 'green' | 'orange' | 'red';
		loading?: boolean;
	}

	let { title, value, icon: Icon, color = 'blue', loading = false }: Props = $props();

	const colorClasses = {
		blue: 'bg-blue-50 text-blue-600',
		green: 'bg-green-50 text-green-600',
		orange: 'bg-orange-50 text-orange-600',
		red: 'bg-red-50 text-red-600'
	};
</script>

<div class="bg-white rounded-lg shadow p-6">
	<div class="flex items-center justify-between">
		<div class="flex-1">
			<p class="text-sm text-gray-600 mb-1">{title}</p>
			{#if loading}
				<div class="h-8 w-24 bg-gray-200 animate-pulse rounded"></div>
			{:else}
				<p class="text-3xl font-bold text-gray-900">{value}</p>
			{/if}
		</div>
		<div class="p-3 rounded-full {colorClasses[color]}">
			<Icon class="w-6 h-6" />
		</div>
	</div>
</div>
