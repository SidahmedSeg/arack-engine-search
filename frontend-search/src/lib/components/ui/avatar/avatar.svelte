<script lang="ts">
	import { cn } from '$lib/utils';

	interface Props {
		user: { firstName: string; lastName: string; email: string };
		size?: 'sm' | 'md' | 'lg';
		class?: string;
		onclick?: () => void;
	}

	let { user, size = 'md', class: className, onclick }: Props = $props();

	// Generate initials from first and last name
	const initials = $derived(() => {
		const first = user.firstName?.charAt(0).toUpperCase() || '';
		const last = user.lastName?.charAt(0).toUpperCase() || '';
		return first + last || '?';
	});

	// Generate consistent color from email hash
	const backgroundColor = $derived(() => {
		let hash = 0;
		const email = user.email || '';
		for (let i = 0; i < email.length; i++) {
			hash = email.charCodeAt(i) + ((hash << 5) - hash);
		}

		// Generate pleasing pastel colors
		const hue = Math.abs(hash % 360);
		return `hsl(${hue}, 65%, 55%)`;
	});

	const sizes = {
		sm: 'w-8 h-8 text-xs',
		md: 'w-10 h-10 text-sm',
		lg: 'w-12 h-12 text-base'
	};
</script>

<button
	{onclick}
	class={cn(
		'rounded-full flex items-center justify-center font-semibold text-white transition-opacity',
		'hover:opacity-90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-blue-500',
		sizes[size],
		className
	)}
	style="background-color: {backgroundColor()};"
	aria-label="User menu"
	type="button"
>
	{initials()}
</button>
