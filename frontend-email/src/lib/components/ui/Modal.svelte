<script lang="ts">
	import { X } from 'lucide-svelte';
	import Button from './Button.svelte';
	import { cn } from '$lib/utils';

	interface Props {
		open?: boolean;
		title?: string;
		size?: 'sm' | 'md' | 'lg' | 'xl' | 'full';
		onClose?: () => void;
		children?: any;
	}

	let { open = $bindable(false), title = '', size = 'md', onClose, children }: Props = $props();

	const sizeClasses = {
		sm: 'max-w-sm',
		md: 'max-w-md',
		lg: 'max-w-lg',
		xl: 'max-w-4xl',
		full: 'max-w-6xl'
	};

	function handleClose() {
		open = false;
		onClose?.();
	}

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) {
			handleClose();
		}
	}

	function handleEscapeKey(e: KeyboardEvent) {
		if (e.key === 'Escape' && open) {
			handleClose();
		}
	}
</script>

<svelte:window onkeydown={handleEscapeKey} />

{#if open}
	<div
		class="fixed inset-0 z-50 bg-black/50 backdrop-blur-sm flex items-center justify-center p-4"
		onclick={handleBackdropClick}
		role="dialog"
		aria-modal="true"
	>
		<div
			class={cn(
				'relative w-full bg-white dark:bg-gray-800 rounded-lg shadow-xl overflow-hidden',
				sizeClasses[size]
			)}
		>
			<!-- Header -->
			{#if title}
				<div
					class="flex items-center justify-between px-6 py-4 border-b border-gray-200 dark:border-gray-700"
				>
					<h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">{title}</h2>
					<Button variant="ghost" size="icon" onclick={handleClose}>
						<X class="h-5 w-5" />
					</Button>
				</div>
			{/if}

			<!-- Content -->
			<div class="px-6 py-4">
				{@render children?.()}
			</div>
		</div>
	</div>
{/if}
