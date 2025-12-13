<script lang="ts">
	import { ImageOff } from 'lucide-svelte';
	import type { ImageData } from '$shared/types';

	interface Props {
		images: ImageData[];
		onImageClick?: (image: ImageData) => void;
	}

	let { images = $bindable([]), onImageClick }: Props = $props();

	// Track broken images
	let brokenImages = $state(new Set<string>());

	function handleImageError(imageId: string) {
		brokenImages = new Set([...brokenImages, imageId]);
	}

	function handleImageClick(image: ImageData) {
		if (onImageClick) {
			onImageClick(image);
		}
	}

	function formatDimensions(width?: number, height?: number): string {
		if (width && height) {
			return `${width}×${height}`;
		}
		return 'Unknown size';
	}
</script>

{#if images.length === 0}
	<div class="flex flex-col items-center justify-center py-16 text-gray-500">
		<ImageOff class="w-16 h-16 mb-4 text-gray-300" />
		<p class="text-lg font-medium">No images found</p>
		<p class="text-sm">Try a different search query</p>
	</div>
{:else}
	<div
		class="grid grid-cols-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-3"
	>
		{#each images as image (image.id)}
			<button
				type="button"
				class="group relative aspect-square overflow-hidden rounded-lg bg-gray-100 hover:ring-2 hover:ring-blue-500 transition-all cursor-pointer"
				onclick={() => handleImageClick(image)}
			>
				{#if brokenImages.has(image.id)}
					<!-- Broken image placeholder -->
					<div
						class="w-full h-full flex flex-col items-center justify-center bg-gray-50 text-gray-400"
					>
						<ImageOff class="w-8 h-8 mb-2" />
						<span class="text-xs text-center px-2">Image unavailable</span>
					</div>
				{:else}
					<!-- Image -->
					<img
						src={image.image_url}
						alt={image.alt_text || 'Image'}
						class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-200"
						onerror={() => handleImageError(image.id)}
						loading="lazy"
					/>

					<!-- Hover overlay with info -->
					<div
						class="absolute inset-0 bg-gradient-to-t from-black/80 via-black/20 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-200 flex items-end"
					>
						<div class="p-2 text-white w-full">
							<p class="text-xs font-medium truncate mb-0.5">
								{image.figcaption || image.alt_text || image.page_title}
							</p>
							<div class="flex items-center justify-between text-xs text-gray-300">
								<span>{formatDimensions(image.width, image.height)}</span>
								<span class="truncate ml-1">{image.domain}</span>
							</div>
							{#if image.is_og_image}
								<div class="text-xs text-blue-300 mt-0.5">★ High Quality</div>
							{/if}
						</div>
					</div>
				{/if}
			</button>
		{/each}
	</div>
{/if}
