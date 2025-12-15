<script lang="ts">
	import { type Snippet } from 'svelte';
	import { cn } from '$lib/utils';

	interface Props {
		class?: string;
		children?: Snippet;
	}

	let { class: className, children, ...restProps }: Props = $props();
</script>

<div
	class={cn('rounded-lg border border-gray-200 bg-white text-gray-950', className)}
	{...restProps}
>
	{#if children}
		{@render children()}
	{/if}
</div>
