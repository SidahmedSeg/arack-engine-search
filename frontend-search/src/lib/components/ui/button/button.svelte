<script lang="ts">
	import { type Snippet } from 'svelte';
	import { cn } from '$lib/utils';

	interface Props {
		variant?: 'default' | 'destructive' | 'outline' | 'secondary' | 'ghost' | 'link';
		size?: 'default' | 'sm' | 'lg' | 'icon';
		class?: string;
		type?: 'button' | 'submit' | 'reset';
		disabled?: boolean;
		onclick?: (event: MouseEvent) => void;
		children?: Snippet;
	}

	let {
		variant = 'default',
		size = 'default',
		class: className,
		type = 'button',
		disabled = false,
		onclick,
		children,
		...restProps
	}: Props = $props();

	const variants = {
		default: 'bg-primary text-white hover:bg-primary/90',
		destructive: 'bg-red-600 text-white hover:bg-red-700',
		outline: 'border border-gray-300 bg-white hover:bg-gray-50 text-gray-900',
		secondary: 'bg-gray-100 text-gray-900 hover:bg-gray-200',
		ghost: 'hover:bg-gray-100 hover:text-gray-900',
		link: 'text-primary underline-offset-4 hover:underline'
	};

	const sizes = {
		default: 'h-9 px-4 py-2',
		sm: 'h-8 px-3 text-sm',
		lg: 'h-10 px-6',
		icon: 'h-9 w-9'
	};
</script>

<button
	{type}
	{disabled}
	{onclick}
	class={cn(
		'inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium transition-colors',
		'focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-gray-950',
		'disabled:pointer-events-none disabled:opacity-50',
		variants[variant],
		sizes[size],
		className
	)}
	{...restProps}
>
	{#if children}
		{@render children()}
	{/if}
</button>
