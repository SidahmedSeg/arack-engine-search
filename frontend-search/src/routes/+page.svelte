<script lang="ts">
	import { goto } from '$app/navigation';
	import Header from '$lib/components/Header.svelte';
	import Footer from '$lib/components/Footer.svelte';
	import SearchBar from '$lib/components/ui/search-bar/search-bar.svelte';

	let query = $state('');

	function handleSearch() {
		if (query.trim()) {
			goto(`/search?q=${encodeURIComponent(query)}`);
		}
	}
</script>

<div class="min-h-screen bg-white flex flex-col">
	<!-- Header with right-aligned nav only -->
	<Header />

	<!-- Main Content: Centered Logo + Search (accounting for header and footer) -->
	<main class="flex-1 flex flex-col items-center justify-center px-4 -mt-20">
		<!-- 2arak Logo -->
		<a href="/" class="mb-8">
			<img src="/logo-2arak.svg" alt="2arak Search" class="h-16 w-auto" />
		</a>

		<!-- Google-style Search Bar with lighter shadow -->
		<div class="w-full max-w-xl">
			<SearchBar bind:value={query} onSearch={handleSearch} placeholder="Search..." autofocus />
		</div>
	</main>

	<!-- Footer -->
	<Footer />
</div>
