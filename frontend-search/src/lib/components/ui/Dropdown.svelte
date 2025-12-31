<script lang="ts">
	import type { Snippet } from 'svelte';
	import { onMount } from 'svelte';

	interface Props {
		trigger: Snippet;
		children: Snippet;
		align?: 'left' | 'right';
		class?: string;
	}

	let {
		trigger,
		children,
		align = 'right',
		class: className = ''
	}: Props = $props();

	let isOpen = $state(false);
	let dropdownRef: HTMLDivElement;

	function toggle() {
		isOpen = !isOpen;
	}

	function close() {
		isOpen = false;
	}

	onMount(() => {
		function handleClickOutside(event: MouseEvent) {
			if (dropdownRef && !dropdownRef.contains(event.target as Node)) {
				close();
			}
		}

		document.addEventListener('mousedown', handleClickOutside);
		return () => document.removeEventListener('mousedown', handleClickOutside);
	});

	const alignClasses = align === 'left' ? 'left-0' : 'right-0';
</script>

<div class="relative inline-block {className}" bind:this={dropdownRef}>
	<div onclick={toggle} class="cursor-pointer">
		{@render trigger()}
	</div>

	{#if isOpen}
		<div
			class="absolute {alignClasses} mt-2 w-56 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg overflow-hidden z-50"
		>
			<div onclick={close}>
				{@render children()}
			</div>
		</div>
	{/if}
</div>
