<script lang="ts">
	import { cn } from '$lib/utils';

	interface Props {
		type?: string;
		placeholder?: string;
		value?: string;
		class?: string;
		disabled?: boolean;
		required?: boolean;
		oninput?: (e: Event) => void;
	}

	let {
		type = 'text',
		placeholder = '',
		value = $bindable(''),
		class: className,
		disabled = false,
		required = false,
		oninput
	}: Props = $props();
</script>

<input
	{type}
	{placeholder}
	{disabled}
	{required}
	bind:value
	{oninput}
	class={cn(
		'flex h-9 w-full rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 px-3 py-1 text-sm text-gray-900 dark:text-gray-100 placeholder:text-gray-500 dark:placeholder:text-gray-400 focus:outline-none focus:ring-2 focus:ring-primary-500 focus:ring-offset-0 disabled:cursor-not-allowed disabled:opacity-50',
		className
	)}
/>
