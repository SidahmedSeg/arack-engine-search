<script lang="ts">
	import { cn } from '$lib/utils';

	interface Props {
		variant?: 'default' | 'primary' | 'secondary' | 'ghost' | 'danger';
		size?: 'sm' | 'md' | 'lg' | 'icon';
		class?: string;
		disabled?: boolean;
		type?: 'button' | 'submit' | 'reset';
		onclick?: () => void;
		children?: any;
	}

	let {
		variant = 'default',
		size = 'md',
		class: className,
		disabled = false,
		type = 'button',
		onclick,
		children
	}: Props = $props();

	const variantStyles = {
		default:
			'bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-200 border border-gray-300 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-gray-700',
		primary:
			'bg-primary-600 text-white border border-primary-600 hover:bg-primary-700 dark:bg-primary-500 dark:hover:bg-primary-600',
		secondary:
			'bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-200 border border-transparent hover:bg-gray-200 dark:hover:bg-gray-600',
		ghost:
			'bg-transparent text-gray-700 dark:text-gray-200 border border-transparent hover:bg-gray-100 dark:hover:bg-gray-800',
		danger:
			'bg-red-600 text-white border border-red-600 hover:bg-red-700 dark:bg-red-500 dark:hover:bg-red-600'
	};

	const sizeStyles = {
		sm: 'h-8 px-3 text-sm',
		md: 'h-9 px-4 text-sm',
		lg: 'h-10 px-6 text-base',
		icon: 'h-9 w-9 p-0'
	};
</script>

<button
	{type}
	{disabled}
	{onclick}
	class={cn(
		'inline-flex items-center justify-center gap-2 rounded-md font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary-500 focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50',
		variantStyles[variant],
		sizeStyles[size],
		className
	)}
>
	{@render children?.()}
</button>
