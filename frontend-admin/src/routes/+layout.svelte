<script lang="ts">
	import '../app.css';
	import favicon from '$lib/assets/favicon.svg';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import { page } from '$app/stores';
	import { user } from '$lib/stores/auth';

	let { children } = $props();

	// Check if we're on a public route (login, invite)
	const publicRoutes = ['/login', '/invite'];
	let isPublicRoute = $derived(
		publicRoutes.some((route) => $page.url.pathname.startsWith(route))
	);
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>Search Engine - Admin Dashboard</title>
</svelte:head>

{#if isPublicRoute || !$user}
	<!-- Public layout (no sidebar) -->
	<div class="min-h-screen bg-gray-50">
		{@render children()}
	</div>
{:else}
	<!-- Authenticated layout (with sidebar) -->
	<div class="flex min-h-screen bg-gray-50">
		<Sidebar />

		<main class="flex-1 ml-64">
			<div class="p-8">
				{@render children()}
			</div>
		</main>
	</div>
{/if}
