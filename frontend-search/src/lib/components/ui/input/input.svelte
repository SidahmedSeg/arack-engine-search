<script lang="ts">
	import { cn } from '$lib/utils';

	interface Props {
		type?: 'text' | 'email' | 'password' | 'number' | 'tel' | 'url' | 'search';
		value?: string;
		name?: string;
		id?: string;
		placeholder?: string;
		required?: boolean;
		disabled?: boolean;
		label?: string;
		error?: string;
		class?: string;
		oninput?: (e: Event) => void;
	}

	let {
		type = 'text',
		value = $bindable(''),
		name,
		id,
		placeholder,
		required = false,
		disabled = false,
		label,
		error,
		class: className,
		oninput,
		...restProps
	}: Props = $props();
</script>

{#if label}
	<label for={id} class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1.5">
		{label}
		{#if required}<span class="text-red-500 ml-0.5">*</span>{/if}
	</label>
{/if}

<input
	{type}
	{name}
	{id}
	{placeholder}
	{required}
	{disabled}
	{oninput}
	bind:value
	class={cn(
		'flex h-9 w-full rounded-md border bg-white px-3 py-1 text-sm transition-colors',
		'file:border-0 file:bg-transparent file:text-sm file:font-medium',
		'placeholder:text-gray-500',
		'focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-gray-950',
		'disabled:cursor-not-allowed disabled:opacity-50',
		error ? 'border-red-500 focus-visible:ring-red-500' : 'border-gray-300',
		className
	)}
	{...restProps}
/>

{#if error}
	<p class="mt-1.5 text-xs text-red-600 dark:text-red-400">{error}</p>
{/if}
