<script lang="ts">
	import { X, ExternalLink, Image as ImageIcon } from 'lucide-svelte';
	import type { ImageData } from '$shared/types';

	interface Props {
		image: ImageData | null;
		onClose: () => void;
	}

	let { image, onClose }: Props = $props();

	let isOpen = $derived(image !== null);

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) {
			onClose();
		}
	}

	function openSource() {
		if (image) {
			window.open(image.source_url, '_blank');
		}
	}

	function openImage() {
		if (image) {
			window.open(image.image_url, '_blank');
		}
	}
</script>

<!-- Drawer overlay -->
{#if isOpen}
	<div
		class="fixed top-14 right-0 bottom-0 left-0 bg-black/50 z-50 flex items-start justify-end"
		onclick={handleBackdropClick}
		role="dialog"
		aria-modal="true"
	>
		<!-- Drawer panel -->
		<div
			class="bg-white h-full w-full max-w-md shadow-2xl overflow-y-auto animate-slide-in-right"
		>
			{#if image}
				<!-- Header -->
				<div class="sticky top-0 bg-white border-b border-gray-200 px-6 py-4 flex items-center justify-between z-10">
					<h2 class="text-lg font-semibold text-gray-900 truncate flex items-center gap-2">
						<ImageIcon class="w-5 h-5" />
						Image Details
					</h2>
					<button
						type="button"
						onclick={onClose}
						class="p-1.5 rounded-lg hover:bg-gray-100 transition-colors"
						aria-label="Close"
					>
						<X class="w-5 h-5" />
					</button>
				</div>

				<!-- Image preview -->
				<div class="relative w-full bg-gray-100">
					<img
						src={image.image_url}
						alt={image.alt_text || 'Image preview'}
						class="w-full h-auto max-h-96 object-contain"
					/>
				</div>

				<!-- Image details -->
				<div class="p-6 space-y-3">
					<!-- Figcaption (Priority 1: Rich semantic description) -->
					{#if image.figcaption}
						<div>
							<h3 class="text-sm font-medium text-gray-500 mb-1.5">Caption</h3>
							<p class="text-base text-gray-900 italic">
								{image.figcaption}
							</p>
						</div>
					{/if}

					<!-- Title / Alt text -->
					{#if image.alt_text || image.title}
						<div>
							<div class="flex items-center gap-2 mb-1.5">
								<h3 class="text-sm font-medium text-gray-500">Title</h3>
								{#if image.is_og_image}
									<span class="inline-flex items-center gap-1 px-2 py-0.5 bg-blue-100 text-blue-800 text-[10px] font-medium rounded-full">
										<ImageIcon class="w-2.5 h-2.5" />
										High Quality
									</span>
								{/if}
							</div>
							<p class="text-base text-gray-900 truncate">
								{image.alt_text || image.title}
							</p>
						</div>
					{/if}

					<!-- Dimensions & Domain Chips -->
					<div class="flex flex-wrap items-center gap-1.5">
						<!-- Dimensions Chip -->
						<div class="inline-flex items-center px-2 py-1 bg-gray-100 text-gray-700 text-xs font-medium rounded-full">
							{#if image.width && image.height}
								{image.width} × {image.height}
							{:else}
								NA
							{/if}
						</div>

						<!-- Domain Chip -->
						<div class="inline-flex items-center px-2 py-1 bg-blue-100 text-blue-700 text-xs font-medium rounded-full">
							{image.domain}
						</div>
					</div>

					<!-- Action button -->
					<button
						type="button"
						onclick={openSource}
						class="w-full flex items-center justify-center gap-1.5 px-3 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-lg transition-colors"
					>
						<span>View Source</span>
						<ExternalLink class="w-3.5 h-3.5" />
					</button>

					<!-- Open Image Button -->
					<button
						type="button"
						onclick={openImage}
						class="w-full text-center text-xs text-blue-600 hover:text-blue-700 hover:underline transition-colors"
					>
						Open image
					</button>
				</div>
			{/if}
		</div>
	</div>
{/if}

<style>
	@keyframes slide-in-right {
		from {
			transform: translateX(100%);
		}
		to {
			transform: translateX(0);
		}
	}

	.animate-slide-in-right {
		animation: slide-in-right 0.3s ease-out;
	}
</style>
