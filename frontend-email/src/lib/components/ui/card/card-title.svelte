<script lang="ts">
	import { type Snippet } from 'svelte';
	import { cn } from '$lib/utils';

	interface Props {
		class?: string;
		children?: Snippet;
	}

	let { class: className, children, ...restProps }: Props = $props();
</script>

<h3
	class={cn('text-2xl font-semibold leading-none tracking-tight', className)}
	{...restProps}
>
	{#if children}
		{@render children()}
	{/if}
</h3>
