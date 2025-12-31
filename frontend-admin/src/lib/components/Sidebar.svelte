<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { Home, Search, FileText, Settings, Activity, BarChart3, LogOut, User } from 'lucide-svelte';
	import { user, auth } from '$lib/stores/auth';

	const navItems = [
		{ href: '/', icon: Home, label: 'Dashboard' },
		{ href: '/crawl', icon: Activity, label: 'Crawl Management' },
		{ href: '/index', icon: FileText, label: 'Index Management' },
		{ href: '/search-test', icon: Search, label: 'Search Testing' },
		{ href: '/analytics', icon: BarChart3, label: 'Search Analytics' }
	];

	function isActive(href: string): boolean {
		if (href === '/') {
			return $page.url.pathname === '/';
		}
		return $page.url.pathname.startsWith(href);
	}

	async function handleLogout() {
		await auth.logout();
		goto('/login');
	}

	// Get user initials for avatar
	function getUserInitials(): string {
		if (!$user) return '';
		const first = $user.first_name?.[0] || '';
		const last = $user.last_name?.[0] || '';
		return (first + last).toUpperCase();
	}

	function getUserName(): string {
		if (!$user) return '';
		return `${$user.first_name || ''} ${$user.last_name || ''}`.trim() || $user.email;
	}
</script>

<aside class="w-64 bg-gray-900 text-white h-screen fixed left-0 top-0 overflow-y-auto">
	<!-- Logo/Header -->
	<div class="p-6 border-b border-gray-800">
		<h1 class="text-xl font-bold flex items-center gap-2">
			<Search class="w-6 h-6 text-primary" />
			Search Engine
		</h1>
		<p class="text-sm text-gray-400 mt-1">Admin Dashboard</p>
	</div>

	<!-- Navigation -->
	<nav class="p-4">
		<ul class="space-y-2">
			{#each navItems as item}
				<li>
					<a
						href={item.href}
						class="flex items-center gap-3 px-4 py-3 rounded-lg transition-colors {isActive(item.href)
							? 'bg-primary text-white'
							: 'text-gray-300 hover:bg-gray-800 hover:text-white'}"
					>
						<svelte:component this={item.icon} class="w-5 h-5" />
						<span>{item.label}</span>
					</a>
				</li>
			{/each}
		</ul>
	</nav>

	<!-- User Section -->
	{#if $user}
		<div class="absolute bottom-0 left-0 right-0 p-4 border-t border-gray-800">
			<!-- User Info -->
			<div class="flex items-center gap-3 mb-3">
				<div class="w-10 h-10 rounded-full bg-blue-600 flex items-center justify-center text-sm font-semibold">
					{getUserInitials()}
				</div>
				<div class="flex-1 min-w-0">
					<p class="text-sm font-medium text-white truncate">{getUserName()}</p>
					<p class="text-xs text-gray-400 truncate">{$user.email}</p>
					{#if $user.role === 'admin'}
						<span class="inline-block text-xs bg-blue-600 text-white px-2 py-0.5 rounded mt-1">
							Admin
						</span>
					{/if}
				</div>
			</div>

			<!-- Logout Button -->
			<button
				onclick={handleLogout}
				class="w-full flex items-center gap-2 px-3 py-2 text-sm text-gray-300 hover:text-white hover:bg-gray-800 rounded-lg transition-colors"
			>
				<LogOut class="w-4 h-4" />
				<span>Sign out</span>
			</button>

			<div class="mt-3 pt-3 border-t border-gray-800">
				<p class="text-xs text-gray-500">Version 1.0.0</p>
				<p class="text-xs text-gray-600">Powered by Rust + Meilisearch</p>
			</div>
		</div>
	{/if}
</aside>
