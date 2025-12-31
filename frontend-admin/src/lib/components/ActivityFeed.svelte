<script lang="ts">
	import { Activity, Search, FileText, Trash2, Clock } from 'lucide-svelte';

	interface ActivityItem {
		id: string;
		type: 'crawl' | 'search' | 'index' | 'clear';
		message: string;
		timestamp: string;
	}

	interface Props {
		activities: ActivityItem[];
		maxItems?: number;
	}

	let { activities, maxItems = 10 }: Props = $props();

	const displayedActivities = $derived(activities.slice(0, maxItems));

	function formatRelativeTime(timestamp: string): string {
		const now = new Date();
		const past = new Date(timestamp);
		const diffMs = now.getTime() - past.getTime();
		const diffMins = Math.floor(diffMs / 60000);

		if (diffMins < 1) return 'just now';
		if (diffMins < 60) return `${diffMins} minute${diffMins > 1 ? 's' : ''} ago`;

		const diffHours = Math.floor(diffMins / 60);
		if (diffHours < 24) return `${diffHours} hour${diffHours > 1 ? 's' : ''} ago`;

		const diffDays = Math.floor(diffHours / 24);
		return `${diffDays} day${diffDays > 1 ? 's' : ''} ago`;
	}

	function getIcon(type: string) {
		switch (type) {
			case 'crawl':
				return Activity;
			case 'search':
				return Search;
			case 'index':
				return FileText;
			case 'clear':
				return Trash2;
			default:
				return Clock;
		}
	}

	function getColor(type: string) {
		switch (type) {
			case 'crawl':
				return 'text-blue-600 bg-blue-50';
			case 'search':
				return 'text-green-600 bg-green-50';
			case 'index':
				return 'text-orange-600 bg-orange-50';
			case 'clear':
				return 'text-red-600 bg-red-50';
			default:
				return 'text-gray-600 bg-gray-50';
		}
	}
</script>

<div class="space-y-3">
	{#if displayedActivities.length === 0}
		<p class="text-sm text-gray-500 text-center py-4">No recent activity</p>
	{:else}
		{#each displayedActivities as activity (activity.id)}
			{@const Icon = getIcon(activity.type)}
			<div class="flex items-start gap-3 p-3 bg-gray-50 rounded-lg hover:bg-gray-100 transition-colors">
				<div class="p-2 rounded-lg {getColor(activity.type)}">
					<Icon class="w-4 h-4" />
				</div>
				<div class="flex-1 min-w-0">
					<p class="text-sm text-gray-900">{activity.message}</p>
					<p class="text-xs text-gray-500 mt-1">{formatRelativeTime(activity.timestamp)}</p>
				</div>
			</div>
		{/each}
	{/if}
</div>
