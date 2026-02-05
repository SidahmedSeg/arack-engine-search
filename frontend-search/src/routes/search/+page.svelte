<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { search } from '$lib/api';
	import type { SearchResponse, SearchFilters } from '$lib/types';
	import { ExternalLink, Calendar, FileText, ChevronLeft, ChevronRight, Lightbulb, Image as ImageIcon, Star, LayoutGrid, Sparkles, Menu, X, SlidersHorizontal } from 'lucide-svelte';
	import SearchBar from '$lib/components/ui/search-bar/search-bar.svelte';
	import ImageGrid from '$lib/components/ImageGrid.svelte';
	import ImagePreview from '$lib/components/ImagePreview.svelte';
	import Avatar from '$lib/components/ui/avatar/avatar.svelte';
	import Button from '$lib/components/ui/button/button.svelte';
	import { authStore } from '$lib/stores/auth.svelte';
	import { api as apiClient } from '$shared/api-client';
	import type { ImageData, ImageSearchResponse, HybridSearchResponse } from '$shared/types';
	import axios from 'axios';

	// Search tab state
	let activeTab = $state<'all' | 'images'>('all');

	// Text search state
	let query = $state(''); // The query from URL (used for searching)
	let searchInput = $state(''); // The input field value (what user types)
	let currentPage = $state(1);
	let limit = 20;
	let searchResults: SearchResponse | null = $state(null);
	let loading = $state(false);
	let error = $state('');

	// Image search state
	let imageResults: ImageSearchResponse | null = $state(null);
	let imageLoading = $state(false);
	let imageError = $state('');
	let selectedImage: ImageData | null = $state(null);
	let imagePage = $state(1);
	let imageLimit = 20;

	// Image filters
	type SizeFilter = 'all' | 'large' | 'medium' | 'small';
	let sizeFilter = $state<SizeFilter>('all');
	let ogFilter = $state(false); // High Quality Only toggle

	// User menu state
	let showUserMenu = $state(false);

	// Mobile menu state
	let showMobileMenu = $state(false);

	// Mobile filters state
	let showMobileFilters = $state(false);

	let filters: SearchFilters = $state({
		query: '',
		limit: limit,
		offset: 0
	});

	function toggleUserMenu() {
		showUserMenu = !showUserMenu;
	}

	async function performSearch() {
		if (!query.trim()) return;

		loading = true;
		error = '';

		try {
			const offset = (currentPage - 1) * limit;

			// Phase 10: Always use hybrid search (semantic + keyword)
			const hybridResults = await apiClient.hybridSearch({ q: query, limit, offset });
			// Convert HybridSearchResponse to SearchResponse format for compatibility
			searchResults = {
				results: hybridResults.hits.map(hit => ({
					id: hit.id,
					url: hit.url,
					title: hit.title,
					content: hit.content || '',
					description: hit.description,
					keywords: [],
					word_count: 0,
					crawled_at: '',
					_formatted: {
						title: hit.title,
						content: hit.content,
						description: hit.description
					}
				})),
				query: hybridResults.query,
				processing_time_ms: hybridResults.processing_time_ms,
				total_hits: hybridResults.hits.length,
				total: hybridResults.hits.length
			};

			// Track search history if user is authenticated
			if (authStore.isAuthenticated && searchResults) {
				trackSearch(query, searchResults.total_hits, filters);
			}
		} catch (err: any) {
			error = err.response?.data?.error || 'Failed to perform search';
			console.error('Search error:', err);
		} finally {
			loading = false;
		}
	}

	async function trackSearch(searchQuery: string, resultCount: number, searchFilters: SearchFilters) {
		try {
			await axios.post('https://api.arack.io/api/ory/search-history', {
				query: searchQuery,
				result_count: resultCount,
				filters: searchFilters
			}, {
				withCredentials: true
			});
		} catch (err) {
			// Silently fail - tracking shouldn't break the search experience
			console.debug('Failed to track search:', err);
		}
	}

	// Initialize from URL params and trigger search
	// Use $page.url as dependency to only run when URL changes
	$effect(() => {
		const url = $page.url;
		const urlQuery = url.searchParams.get('q') || '';
		const urlPage = parseInt(url.searchParams.get('page') || '1');

		// Only update if different from current state to avoid loops
		if (urlQuery !== query || urlPage !== currentPage) {
			query = urlQuery;
			searchInput = urlQuery; // Also update the input field
			currentPage = urlPage;

			// Trigger search when URL params are loaded
			if (query) {
				performSearch();
				// Also trigger image search if on images tab
				if (activeTab === 'images') {
					performImageSearch();
				}
			}
		}
	});

	function handleSearch() {
		query = searchInput; // Set query from input field
		currentPage = 1;
		updateUrl();
		// performSearch() will be called by the effect when URL updates
	}

	function updateUrl() {
		const params = new URLSearchParams();
		if (query) params.set('q', query);
		if (currentPage > 1) params.set('page', currentPage.toString());
		goto(`/search?${params.toString()}`, { replaceState: true, noScroll: true });
	}

	function nextPage() {
		if (!searchResults) return;
		const totalPages = Math.ceil(searchResults.total / limit);
		if (currentPage < totalPages) {
			currentPage++;
			updateUrl();
			// performSearch() will be called by the effect when URL updates
			window.scrollTo({ top: 0, behavior: 'smooth' });
		}
	}

	function prevPage() {
		if (currentPage > 1) {
			currentPage--;
			updateUrl();
			// performSearch() will be called by the effect when URL updates
			window.scrollTo({ top: 0, behavior: 'smooth' });
		}
	}

	function highlightText(text: string, query: string): string {
		if (!query) return text;
		const regex = new RegExp(`(${query})`, 'gi');
		return text.replace(regex, '<mark class="bg-yellow-200">$1</mark>');
	}

	function formatDate(dateString: string): string {
		const date = new Date(dateString);
		return date.toLocaleDateString('en-US', { year: 'numeric', month: 'short', day: 'numeric' });
	}

	function getFaviconUrl(faviconUrl: string | undefined, pageUrl: string): string {
		// Priority 1: Use favicon extracted during crawl (production approach)
		if (faviconUrl) {
			return faviconUrl;
		}

		// Fallback: Construct /favicon.ico from domain
		try {
			const urlObj = new URL(pageUrl);
			return `${urlObj.origin}/favicon.ico`;
		} catch {
			return '';
		}
	}

	// Phase 7.2: Handle suggestion click
	function handleSuggestionClick(suggestion: string) {
		searchInput = suggestion;
		query = suggestion;
		currentPage = 1;
		updateUrl();
	}

	// Image search functions
	async function performImageSearch() {
		if (!query.trim()) return;

		imageLoading = true;
		imageError = '';

		try {
			const offset = (imagePage - 1) * imageLimit;

			// Build filter parameters
			let min_width: number | undefined;
			let min_height: number | undefined;

			if (sizeFilter === 'large') {
				min_width = 1920;
				min_height = 1080;
			} else if (sizeFilter === 'medium') {
				min_width = 1280;
				min_height = 720;
			}
			// 'small' and 'all' don't set min dimensions

			const params: any = {
				q: query,
				limit: imageLimit,
				offset,
			};

			if (min_width) params.min_width = min_width;
			if (min_height) params.min_height = min_height;

			// Phase 10.5: Use hybrid image search (semantic + keyword)
			const results = await apiClient.hybridImageSearch(params);

			// Filter by OG images if toggle is on
			if (ogFilter) {
				results.hits = results.hits.filter(img => img.is_og_image);
				results.total_hits = results.hits.length;
			}

			imageResults = results;
		} catch (err: any) {
			imageError = err.message || 'Failed to search images';
			console.error('Image search error:', err);
		} finally {
			imageLoading = false;
		}
	}

	function switchTab(tab: 'all' | 'images') {
		activeTab = tab;
		// Always perform image search when switching to images tab with a query
		// This ensures images are refreshed if query changed while on All tab
		if (tab === 'images' && query) {
			performImageSearch();
		}
	}

	function handleImageClick(image: ImageData) {
		selectedImage = image;
	}

	function setSizeFilter(filter: SizeFilter) {
		sizeFilter = filter;
		imagePage = 1;
		if (activeTab === 'images' && query) {
			performImageSearch();
		}
	}

	function toggleOgFilter() {
		ogFilter = !ogFilter;
		imagePage = 1;
		if (activeTab === 'images' && query) {
			performImageSearch();
		}
	}

	function nextImagePage() {
		if (!imageResults) return;
		const totalPages = Math.ceil(imageResults.total_hits / imageLimit);
		if (imagePage < totalPages) {
			imagePage++;
			performImageSearch();
			window.scrollTo({ top: 0, behavior: 'smooth' });
		}
	}

	function prevImagePage() {
		if (imagePage > 1) {
			imagePage--;
			performImageSearch();
			window.scrollTo({ top: 0, behavior: 'smooth' });
		}
	}
</script>

<div class="min-h-screen bg-gray-50">
	<!-- Responsive Header -->
	<header class="bg-gray-100 px-3 md:px-5 py-2 sticky top-0 z-20">
		<div class="flex items-center gap-2 md:gap-3">
			<!-- Mobile: Hamburger Menu -->
			<button
				class="md:hidden p-2 hover:bg-gray-200 rounded-lg transition-colors"
				onclick={() => (showMobileMenu = !showMobileMenu)}
				aria-label="Menu"
			>
				{#if showMobileMenu}
					<X size={24} class="text-gray-700" />
				{:else}
					<Menu size={24} class="text-gray-700" />
				{/if}
			</button>

			<!-- Logo (Hidden on mobile, shown on tablet+) -->
			<a href="/" class="hidden md:block flex-shrink-0">
				<img src="/logo-2arak.svg" alt="2arak Search" class="h-8 w-auto" />
			</a>

			<!-- Search Bar (Full width on mobile, no max-width on desktop to stay close to logo) -->
			<div class="flex-1 md:flex-initial md:w-auto">
				<SearchBar
					bind:value={searchInput}
					onSearch={handleSearch}
					class="search-header md:w-[600px] lg:w-[700px]"
				/>
			</div>

			<!-- Desktop: Right Side Buttons (pushed to far right with ml-auto) -->
			<div class="hidden md:flex items-center gap-2 flex-shrink-0 ml-auto">
				<button
					class="text-sm text-gray-700 hover:underline h-9 px-2"
					onclick={() => (window.location.href = '/contact')}
				>
					Email
				</button>

				<button
					class="text-sm text-gray-700 hover:underline h-9 px-2"
					onclick={() => (window.location.href = '/explore')}
				>
					Explore
				</button>

				{#if authStore.isAuthenticated && authStore.user}
					<!-- Authenticated: Avatar with dropdown -->
					<div class="relative">
						<Avatar user={authStore.user} size="md" onclick={toggleUserMenu} />
						{#if showUserMenu}
							<div
								class="absolute right-0 top-full mt-2 z-50"
								onmouseleave={() => (showUserMenu = false)}
							>
								<div class="bg-white rounded-lg shadow-xl border border-gray-200 py-1 min-w-[200px]">
									<!-- Greeting -->
									<div class="px-4 py-3 border-b border-gray-200">
										<p class="text-sm font-medium text-gray-900">
											Hi {authStore.user.firstName}
										</p>
									</div>

									<!-- Manage my account link -->
									<div class="py-1 px-2">
										<a
											href="/dashboard"
											class="flex items-center gap-2 px-2 py-2 text-sm text-gray-700 hover:bg-gray-100 rounded transition-colors"
										>
											<span>Manage my account</span>
										</a>
									</div>

									<!-- Logout -->
									<div class="py-1 px-2 border-t border-gray-200">
										<button
											onclick={() => authStore.logout()}
											class="flex items-center gap-2 w-full px-2 py-1.5 text-sm text-red-600 hover:bg-red-50 rounded transition-colors"
										>
											<span>Logout</span>
										</button>
									</div>
								</div>
							</div>
						{/if}
					</div>
				{:else}
					<!-- Not Authenticated: Login and Grid Icon -->
					<Button
						variant="default"
						size="sm"
						class="bg-[#0059ff] hover:bg-[#0059ff]/90 text-white h-9 px-4"
						onclick={() => (window.location.href = '/auth/login')}
					>
						Login
					</Button>

					<button
						class="p-2 hover:bg-gray-100 rounded-full transition-colors"
						aria-label="Apps menu"
						title="Apps"
					>
						<LayoutGrid size={20} class="text-gray-700" />
					</button>
				{/if}
			</div>

			<!-- Mobile: User Avatar Only -->
			{#if authStore.isAuthenticated && authStore.user}
				<div class="md:hidden relative">
					<Avatar user={authStore.user} size="sm" onclick={toggleUserMenu} />
					{#if showUserMenu}
						<div
							class="absolute right-0 top-full mt-2 z-50"
							onmouseleave={() => (showUserMenu = false)}
						>
							<div class="bg-white rounded-lg shadow-xl border border-gray-200 py-1 min-w-[200px]">
								<!-- Greeting -->
								<div class="px-4 py-3 border-b border-gray-200">
									<p class="text-sm font-medium text-gray-900">
										Hi {authStore.user.firstName}
									</p>
								</div>

								<!-- Manage my account link -->
								<div class="py-1 px-2">
									<a
										href="/dashboard"
										class="flex items-center gap-2 px-2 py-2 text-sm text-gray-700 hover:bg-gray-100 rounded transition-colors"
									>
										<span>Manage my account</span>
									</a>
								</div>

								<!-- Logout -->
								<div class="py-1 px-2 border-t border-gray-200">
									<button
										onclick={() => authStore.logout()}
										class="flex items-center gap-2 w-full px-2 py-1.5 text-sm text-red-600 hover:bg-red-50 rounded transition-colors"
									>
										<span>Logout</span>
									</button>
								</div>
							</div>
						</div>
					{/if}
				</div>
			{:else}
				<Button
					variant="default"
					size="sm"
					class="md:hidden bg-[#0059ff] hover:bg-[#0059ff]/90 text-white h-9 px-3 text-xs"
					onclick={() => (window.location.href = '/auth/login')}
				>
					Login
				</Button>
			{/if}
		</div>

		<!-- Mobile Menu Dropdown -->
		{#if showMobileMenu}
			<div class="md:hidden mt-3 pb-2 border-t border-gray-200 pt-3">
				<div class="flex flex-col gap-2">
					<button
						class="text-left px-3 py-2 text-sm text-gray-700 hover:bg-gray-200 rounded-lg transition-colors"
						onclick={() => {
							window.location.href = '/contact';
							showMobileMenu = false;
						}}
					>
						Email
					</button>
					<button
						class="text-left px-3 py-2 text-sm text-gray-700 hover:bg-gray-200 rounded-lg transition-colors"
						onclick={() => {
							window.location.href = '/explore';
							showMobileMenu = false;
						}}
					>
						Explore
					</button>
					{#if !authStore.isAuthenticated}
						<button
							class="text-left px-3 py-2 text-sm text-gray-700 hover:bg-gray-200 rounded-lg transition-colors"
							onclick={() => {
								window.location.href = '/auth/login';
								showMobileMenu = false;
							}}
						>
							Login
						</button>
					{/if}
				</div>
			</div>
		{/if}
	</header>

	<!-- Tabs Section (Responsive) -->
	{#if query && (searchResults || imageResults || loading || imageLoading)}
		<div class="bg-gray-100 border-b border-gray-200">
			<div class="px-3 md:px-5">
				<!-- Align with search bar: logo (h-8 ≈ 40-50px wide) + gap (12px) ≈ ~52-62px offset -->
				<div class="md:ml-[52px] flex gap-4 md:gap-6 overflow-x-auto">
					<button
						type="button"
						onclick={() => switchTab('all')}
						class="pb-3 pt-3 px-1 border-b-2 font-medium text-sm whitespace-nowrap transition-colors {activeTab === 'all'
							? 'border-blue-600 text-blue-600'
							: 'border-transparent text-gray-600 hover:text-gray-900'}"
					>
						All
					</button>
					<button
						type="button"
						onclick={() => switchTab('images')}
						class="pb-3 pt-3 px-1 border-b-2 font-medium text-sm whitespace-nowrap transition-colors flex items-center gap-1.5 {activeTab === 'images'
							? 'border-blue-600 text-blue-600'
							: 'border-transparent text-gray-600 hover:text-gray-900'}"
					>
						<ImageIcon class="w-4 h-4" />
						Images
						{#if imageResults && activeTab === 'images'}
							<span class="text-xs text-gray-500">({imageResults.total_hits})</span>
						{/if}
					</button>
				</div>
			</div>
		</div>
	{/if}

	<!-- Results Section (White Background, No Filters Panel) -->
	<div class="bg-white min-h-screen">
		<!-- All Tab Content -->
		{#if activeTab === 'all'}
			<div class="px-3 md:px-5 py-4 md:py-8">
				<!-- Align with search bar and tabs -->
				<div class="md:ml-[52px] max-w-full md:max-w-3xl lg:max-w-4xl">
				{#if loading}
					<!-- Loading Skeletons -->
					<div class="space-y-4">
						{#each Array(5) as _}
							<div class="bg-gray-50 rounded-lg p-6 shadow animate-pulse">
								<div class="h-6 bg-gray-200 rounded w-3/4 mb-4"></div>
								<div class="h-4 bg-gray-200 rounded w-full mb-2"></div>
								<div class="h-4 bg-gray-200 rounded w-5/6"></div>
							</div>
						{/each}
					</div>
				{:else if error}
					<!-- Error State -->
					<div class="bg-red-50 border border-red-200 rounded-lg p-6 text-center">
						<p class="text-red-800 font-semibold mb-2">Search Error</p>
						<p class="text-red-600">{error}</p>
					</div>
				{:else if searchResults}
					<div>
						<!-- Results Count and Search Mode Toggle -->
						<div class="mb-6 flex items-center justify-between">
							<p class="text-gray-600">
								About <span class="font-semibold">{searchResults.total.toLocaleString()}</span> results
								for "<span class="font-semibold">{searchResults.query}</span>"
								<span class="text-purple-600 font-medium">• Powered by AI</span>
							</p>
						</div>

						<!-- Phase 7.2: Search Suggestions (Did You Mean?) -->
						{#if searchResults.suggestions && searchResults.suggestions.length > 0}
							<div class="bg-blue-50 border border-blue-200 rounded-lg p-4 mb-6">
								<div class="flex items-start gap-3">
									<Lightbulb class="w-5 h-5 text-blue-600 mt-0.5 flex-shrink-0" />
									<div class="flex-1">
										<p class="text-sm text-gray-700 mb-2">Did you mean:</p>
										<div class="flex flex-wrap gap-2">
											{#each searchResults.suggestions as suggestion}
												<button
													onclick={() => handleSuggestionClick(suggestion)}
													class="px-3 py-1.5 bg-white border border-blue-300 rounded-full text-sm text-blue-700 hover:bg-blue-100 hover:border-blue-400 transition-colors font-medium"
												>
													{suggestion}
												</button>
											{/each}
										</div>
									</div>
								</div>
							</div>
						{/if}

						{#if searchResults.results.length === 0}
							<!-- Empty State -->
							<div class="bg-gray-50 rounded-lg p-12 text-center shadow">
								<div class="w-24 h-24 bg-gray-100 rounded-full flex items-center justify-center mx-auto mb-4">
									<svg class="w-12 h-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
									</svg>
								</div>
								<h3 class="text-xl font-semibold text-gray-900 mb-2">No results found</h3>
								<p class="text-gray-600 mb-4">Try different keywords or check your spelling</p>
							</div>
						{:else}
							<!-- Search Results -->
							<div class="divide-y divide-gray-200">
								{#each searchResults.results as result}
									<div class="py-5 first:pt-0">
										<a
											href={result.url}
											target="_blank"
											rel="noopener noreferrer"
											class="group block"
										>
											<h2 class="text-xl font-semibold text-primary group-hover:underline mb-2 flex items-center gap-2">
												<img
													src={getFaviconUrl(result.favicon_url, result.url)}
													alt=""
													class="w-4 h-4 flex-shrink-0"
													onerror={(e) => e.currentTarget.style.display='none'}
												/>
												{@html result._formatted?.title || highlightText(result.title, query)}
												<ExternalLink class="w-4 h-4 opacity-0 group-hover:opacity-100 transition-opacity" />
											</h2>
											<p class="text-sm text-green-700 mb-2 break-all">{result.url}</p>
											<p class="text-gray-700 mb-3 line-clamp-3">
												{@html result._formatted?.content || highlightText(result.content.substring(0, 300) + '...', query)}
											</p>
											<div class="flex items-center gap-4 text-sm text-gray-500">
												{#if result.crawled_at}
													<div class="flex items-center gap-1">
														<Calendar class="w-4 h-4" />
														<span>{formatDate(result.crawled_at)}</span>
													</div>
												{/if}
												{#if result.word_count > 0}
													<div class="flex items-center gap-1">
														<FileText class="w-4 h-4" />
														<span>{result.word_count.toLocaleString()} words</span>
													</div>
												{/if}
											</div>
										</a>
									</div>
								{/each}
							</div>

							<!-- Pagination -->
							{#if searchResults.total > limit}
								<div class="mt-8 flex items-center justify-center gap-2 md:gap-4">
									<button
										onclick={prevPage}
										disabled={currentPage === 1}
										class="flex items-center gap-1 md:gap-2 px-3 md:px-4 py-2 bg-gray-50 border border-gray-300 rounded-lg hover:bg-gray-100 disabled:opacity-50 disabled:cursor-not-allowed transition-colors text-sm md:text-base"
									>
										<ChevronLeft class="w-4 h-4" />
										<span class="hidden sm:inline">Previous</span>
									</button>

									<div class="flex items-center gap-1 md:gap-2">
										{#each Array(Math.min(5, Math.ceil(searchResults.total / limit))) as _, i}
											{@const pageNum = i + 1}
											<button
												onclick={() => {
													currentPage = pageNum;
													updateUrl();
													performSearch();
													window.scrollTo({ top: 0, behavior: 'smooth' });
												}}
												class="px-3 md:px-4 py-2 rounded-lg transition-colors text-sm md:text-base min-w-[40px] md:min-w-[44px] {currentPage === pageNum
													? 'bg-primary text-white'
													: 'bg-gray-50 border border-gray-300 hover:bg-gray-100'}"
											>
												{pageNum}
											</button>
										{/each}
									</div>

									<button
										onclick={nextPage}
										disabled={currentPage >= Math.ceil(searchResults.total / limit)}
										class="flex items-center gap-1 md:gap-2 px-3 md:px-4 py-2 bg-gray-50 border border-gray-300 rounded-lg hover:bg-gray-100 disabled:opacity-50 disabled:cursor-not-allowed transition-colors text-sm md:text-base"
									>
										<span class="hidden sm:inline">Next</span>
										<ChevronRight class="w-4 h-4" />
									</button>
								</div>
							{/if}
						{/if}
					</div>
				{:else}
					<!-- Initial State -->
					<div class="bg-gray-50 rounded-lg p-12 text-center shadow">
						<div class="w-24 h-24 bg-primary bg-opacity-10 rounded-full flex items-center justify-center mx-auto mb-4">
							<svg class="w-12 h-12 text-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
							</svg>
						</div>
						<h3 class="text-xl font-semibold text-gray-900 mb-2">Start searching</h3>
						<p class="text-gray-600">Enter a search query to find relevant content</p>
					</div>
				{/if}
				</div>
			</div>
		{/if}

		<!-- Images Tab Content -->
		{#if activeTab === 'images'}
			<div class="px-3 md:px-5 pt-4 pb-8">
				<!-- Align with search bar and tabs -->
				<div class="md:ml-[52px]">
				{#if imageLoading}
					<!-- Loading Skeletons -->
					<div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-2 md:gap-3">
						{#each Array(20) as _}
							<div class="aspect-square bg-gray-200 rounded-lg animate-pulse"></div>
						{/each}
					</div>
				{:else if imageError}
					<!-- Error State -->
					<div class="bg-red-50 border border-red-200 rounded-lg p-6 text-center max-w-2xl mx-auto">
						<p class="text-red-800 font-semibold mb-2">Image Search Error</p>
						<p class="text-red-600">{imageError}</p>
					</div>
				{:else if imageResults}
					<div>
						<!-- Results Count and Filters -->
						<div class="mb-4 flex flex-col md:flex-row items-start md:items-center justify-between gap-3">
							<p class="text-sm md:text-base text-gray-600">
								About <span class="font-semibold">{imageResults.total_hits.toLocaleString()}</span> images
								for "<span class="font-semibold">{imageResults.query}</span>"
								{#if ogFilter}
									<span class="text-blue-600 font-medium">(High Quality)</span>
								{/if}
							</p>

							<!-- Mobile: Filters Button -->
							<button
								type="button"
								onclick={() => (showMobileFilters = !showMobileFilters)}
								class="md:hidden flex items-center gap-2 px-3 py-2 bg-gray-100 rounded-lg text-sm font-medium"
							>
								<SlidersHorizontal class="w-4 h-4" />
								Filters
								{#if ogFilter || sizeFilter !== 'all'}
									<span class="flex h-2 w-2">
										<span class="absolute inline-flex h-2 w-2 rounded-full bg-blue-500 opacity-75"></span>
									</span>
								{/if}
							</button>

							<!-- Desktop: Inline Filters -->
							<div class="hidden md:flex items-center gap-2">
								<!-- Size Filters -->
								<div class="flex items-center gap-1 bg-gray-50 border border-gray-200 rounded-lg p-1">
									<button
										type="button"
										onclick={() => setSizeFilter('all')}
										class="px-3 py-1.5 text-xs font-medium rounded transition-colors {sizeFilter === 'all'
											? 'bg-blue-100 text-blue-700'
											: 'text-gray-600 hover:bg-gray-100'}"
									>
										All sizes
									</button>
									<button
										type="button"
										onclick={() => setSizeFilter('large')}
										class="px-3 py-1.5 text-xs font-medium rounded transition-colors {sizeFilter === 'large'
											? 'bg-blue-100 text-blue-700'
											: 'text-gray-600 hover:bg-gray-100'}"
									>
										Large
									</button>
									<button
										type="button"
										onclick={() => setSizeFilter('medium')}
										class="px-3 py-1.5 text-xs font-medium rounded transition-colors {sizeFilter === 'medium'
											? 'bg-blue-100 text-blue-700'
											: 'text-gray-600 hover:bg-gray-100'}"
									>
										Medium
									</button>
									<button
										type="button"
										onclick={() => setSizeFilter('small')}
										class="px-3 py-1.5 text-xs font-medium rounded transition-colors {sizeFilter === 'small'
											? 'bg-blue-100 text-blue-700'
											: 'text-gray-600 hover:bg-gray-100'}"
									>
										Small
									</button>
								</div>

								<!-- OG Filter Toggle -->
								<button
									type="button"
									onclick={toggleOgFilter}
									class="px-3 py-1.5 text-xs font-medium rounded-lg border transition-colors flex items-center gap-1.5 {ogFilter
										? 'bg-blue-100 text-blue-700 border-blue-300'
										: 'bg-gray-50 text-gray-600 border-gray-200 hover:bg-gray-100'}"
								>
									<Star class="w-3.5 h-3.5" />
									High Quality
								</button>
							</div>
						</div>

						<!-- Mobile Filters Dropdown -->
						{#if showMobileFilters}
							<div class="md:hidden mb-4 p-4 bg-gray-50 rounded-lg space-y-4">
								<!-- Size Filter Section -->
								<div>
									<p class="text-sm font-medium text-gray-700 mb-2">Image Size</p>
									<div class="grid grid-cols-2 gap-2">
										<button
											type="button"
											onclick={() => {
												setSizeFilter('all');
												showMobileFilters = false;
											}}
											class="px-3 py-2 text-sm font-medium rounded-lg transition-colors {sizeFilter === 'all'
												? 'bg-blue-100 text-blue-700 border-2 border-blue-300'
												: 'bg-white text-gray-600 border border-gray-200'}"
										>
											All sizes
										</button>
										<button
											type="button"
											onclick={() => {
												setSizeFilter('large');
												showMobileFilters = false;
											}}
											class="px-3 py-2 text-sm font-medium rounded-lg transition-colors {sizeFilter === 'large'
												? 'bg-blue-100 text-blue-700 border-2 border-blue-300'
												: 'bg-white text-gray-600 border border-gray-200'}"
										>
											Large
										</button>
										<button
											type="button"
											onclick={() => {
												setSizeFilter('medium');
												showMobileFilters = false;
											}}
											class="px-3 py-2 text-sm font-medium rounded-lg transition-colors {sizeFilter === 'medium'
												? 'bg-blue-100 text-blue-700 border-2 border-blue-300'
												: 'bg-white text-gray-600 border border-gray-200'}"
										>
											Medium
										</button>
										<button
											type="button"
											onclick={() => {
												setSizeFilter('small');
												showMobileFilters = false;
											}}
											class="px-3 py-2 text-sm font-medium rounded-lg transition-colors {sizeFilter === 'small'
												? 'bg-blue-100 text-blue-700 border-2 border-blue-300'
												: 'bg-white text-gray-600 border border-gray-200'}"
										>
											Small
										</button>
									</div>
								</div>

								<!-- Quality Toggle -->
								<div>
									<label class="flex items-center justify-between cursor-pointer">
										<span class="text-sm font-medium text-gray-700">High Quality Only</span>
										<button
											type="button"
											onclick={toggleOgFilter}
											class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors {ogFilter
												? 'bg-blue-600'
												: 'bg-gray-200'}"
										>
											<span
												class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform {ogFilter
													? 'translate-x-6'
													: 'translate-x-1'}"
											></span>
										</button>
									</label>
								</div>
							</div>
						{/if}

						{#if imageResults.hits.length === 0}
							<!-- Empty State -->
							<div class="bg-gray-50 rounded-lg p-12 text-center shadow max-w-2xl mx-auto">
								<div class="w-24 h-24 bg-gray-100 rounded-full flex items-center justify-center mx-auto mb-4">
									<ImageIcon class="w-12 h-12 text-gray-400" />
								</div>
								<h3 class="text-xl font-semibold text-gray-900 mb-2">No images found</h3>
								<p class="text-gray-600 mb-4">Try different keywords or adjust filters</p>
								{#if ogFilter || sizeFilter !== 'all'}
									<button
										type="button"
										onclick={() => {
											ogFilter = false;
											sizeFilter = 'all';
											performImageSearch();
										}}
										class="text-blue-600 hover:text-blue-700 font-medium"
									>
										Clear filters
									</button>
								{/if}
							</div>
						{:else}
							<!-- Image Grid -->
							<ImageGrid images={imageResults.hits} onImageClick={handleImageClick} />

							<!-- Pagination -->
							{#if imageResults.total_hits > imageLimit}
								<div class="mt-8 flex items-center justify-center gap-2 md:gap-4">
									<button
										type="button"
										onclick={prevImagePage}
										disabled={imagePage === 1}
										class="flex items-center gap-1 md:gap-2 px-3 md:px-4 py-2 bg-gray-50 border border-gray-300 rounded-lg text-gray-700 hover:bg-gray-100 disabled:opacity-50 disabled:cursor-not-allowed transition-colors text-sm md:text-base"
									>
										<ChevronLeft class="w-4 h-4" />
										<span class="hidden sm:inline">Previous</span>
									</button>
									<span class="text-sm md:text-base text-gray-600 px-2">
										Page {imagePage} of {Math.ceil(imageResults.total_hits / imageLimit)}
									</span>
									<button
										type="button"
										onclick={nextImagePage}
										disabled={imagePage >= Math.ceil(imageResults.total_hits / imageLimit)}
										class="flex items-center gap-1 md:gap-2 px-3 md:px-4 py-2 bg-gray-50 border border-gray-300 rounded-lg text-gray-700 hover:bg-gray-100 disabled:opacity-50 disabled:cursor-not-allowed transition-colors text-sm md:text-base"
									>
										<span class="hidden sm:inline">Next</span>
										<ChevronRight class="w-4 h-4" />
									</button>
								</div>
							{/if}
						{/if}
					</div>
				{:else}
					<!-- Initial State (no search yet) -->
					<div class="bg-gray-50 rounded-lg p-12 text-center shadow max-w-2xl mx-auto">
						<div class="w-24 h-24 bg-primary bg-opacity-10 rounded-full flex items-center justify-center mx-auto mb-4">
							<ImageIcon class="w-12 h-12 text-primary" />
						</div>
						<h3 class="text-xl font-semibold text-gray-900 mb-2">Search for images</h3>
						<p class="text-gray-600">Enter a search query to find images</p>
					</div>
				{/if}
				</div><!-- Close alignment wrapper -->
			</div>
		{/if}
	</div>
</div>

<!-- Image Preview Drawer -->
<ImagePreview image={selectedImage} onClose={() => (selectedImage = null)} />
