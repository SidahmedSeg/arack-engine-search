<script lang="ts">
	interface Props {
		type?: 'text' | 'email' | 'password' | 'number' | 'search';
		placeholder?: string;
		value?: string;
		disabled?: boolean;
		required?: boolean;
		error?: string;
		label?: string;
		class?: string;
		name?: string;
		oninput?: (e: Event) => void;
	}

	let {
		type = 'text',
		placeholder = '',
		value = $bindable(''),
		disabled = false,
		required = false,
		error = '',
		label = '',
		class: className = '',
		name,
		oninput
	}: Props = $props();

	const baseClasses = 'w-full px-3 py-2 text-sm border rounded-md transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent disabled:opacity-50 disabled:cursor-not-allowed';

	const stateClasses = error
		? 'border-red-500 focus:ring-red-500'
		: 'border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 hover:border-gray-400 dark:hover:border-gray-500';

	const computedClasses = `${baseClasses} ${stateClasses} ${className}`;
</script>

{#if label}
	<label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1.5">
		{label}
		{#if required}<span class="text-red-500">*</span>{/if}
	</label>
{/if}

<input
	{type}
	{placeholder}
	{disabled}
	{required}
	{name}
	bind:value
	class={computedClasses}
	oninput={oninput}
/>

{#if error}
	<p class="mt-1.5 text-xs text-red-600 dark:text-red-400">{error}</p>
{/if}
