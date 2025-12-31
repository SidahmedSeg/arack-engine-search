<script lang="ts">
	import { cn, getGravatarUrl, getInitials } from '$lib/utils';

	interface Props {
		email: string;
		name?: string;
		size?: 'sm' | 'md' | 'lg';
		class?: string;
	}

	let { email, name = '', size = 'md', class: className }: Props = $props();

	const sizeClasses = {
		sm: 'h-8 w-8 text-xs',
		md: 'h-10 w-10 text-sm',
		lg: 'h-12 w-12 text-base'
	};

	const sizePixels = {
		sm: 32,
		md: 40,
		lg: 48
	};

	let imageError = $state(false);
	const gravatarUrl = getGravatarUrl(email, sizePixels[size]);
	const initials = name ? getInitials(name) : email.substring(0, 2).toUpperCase();
</script>

<div
	class={cn(
		'relative inline-flex items-center justify-center rounded-full overflow-hidden',
		sizeClasses[size],
		className
	)}
>
	{#if !imageError}
		<img
			src={gravatarUrl}
			alt={name || email}
			class="h-full w-full object-cover"
			onerror={() => (imageError = true)}
		/>
	{:else}
		<div
			class="flex h-full w-full items-center justify-center bg-primary-600 text-white font-medium"
		>
			{initials}
		</div>
	{/if}
</div>
