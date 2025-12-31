<script lang="ts">
	import { page } from '$app/stores';
	import { authStore } from '$lib/stores/auth.svelte';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import Avatar from '$lib/components/ui/avatar/avatar.svelte';
	import { User, Bookmark, History, Settings, Menu, X } from 'lucide-svelte';

	let sidebarOpen = $state(false);
	let showUserMenu = $state(false);

	onMount(() => {
		if (!authStore.isAuthenticated) {
			goto('/auth/login');
		}
	});

	const navItems = [
		{ href: '/dashboard/profile', label: 'My Profile', icon: User },
		{ href: '/dashboard/saved', label: 'My saved search', icon: Bookmark },
		{ href: '/dashboard/history', label: 'my search history', icon: History },
		{ href: '/dashboard/settings', label: 'settings', icon: Settings },
	];

	function isActive(href: string) {
		return $page.url.pathname === href;
	}

	function toggleUserMenu() {
		showUserMenu = !showUserMenu;
	}
</script>

<div class="min-h-screen bg-gray-50">
	<!-- Header: Same as search page but with "Account" text -->
	<header class="bg-gray-100 px-5 py-3 sticky top-0 z-20 border-b border-gray-200">
		<div class="flex items-center justify-between gap-4">
			<!-- Left: Logo + "Account" -->
			<div class="flex items-center gap-3">
				<a href="/" class="flex-shrink-0">
					<img src="/logo-2arak.svg" alt="2arak Search" class="h-8 w-auto" />
				</a>
				<span class="text-lg font-medium text-gray-700">Account</span>
			</div>

			<!-- Right: Email, Explore, Avatar -->
			<div class="flex items-center gap-2 flex-shrink-0">
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

				<!-- Mobile Menu Button -->
				<button
					class="lg:hidden p-2 hover:bg-gray-200 rounded-full"
					onclick={() => (sidebarOpen = !sidebarOpen)}
				>
					{#if sidebarOpen}
						<X size={20} />
					{:else}
						<Menu size={20} />
					{/if}
				</button>

				<!-- Avatar with dropdown (desktop only) -->
				<div class="relative hidden lg:block">
					<Avatar user={authStore.user} size="md" onclick={toggleUserMenu} />
					{#if showUserMenu}
						<div
							class="absolute right-0 top-full mt-2 z-50"
							onmouseleave={() => (showUserMenu = false)}
						>
							<div class="bg-white rounded-lg shadow-xl border border-gray-200 py-1 min-w-[200px]">
								<div class="px-4 py-3 border-b border-gray-200">
									<p class="text-sm font-medium text-gray-900">
										Hi {authStore.user.firstName}
									</p>
								</div>
								<div class="py-1 px-2">
									<a
										href="/dashboard"
										class="flex items-center gap-2 px-2 py-2 text-sm text-gray-700 hover:bg-gray-100 rounded transition-colors"
									>
										<span>Manage my account</span>
									</a>
								</div>
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
			</div>
		</div>
	</header>

	<!-- Sidebar + Content Layout -->
	<div class="flex min-h-[calc(100vh-57px)]">
		<!-- Sidebar -->
		<aside
			class="fixed lg:sticky top-[57px] left-0 h-screen lg:h-auto w-64 bg-white border-r border-gray-200 z-10 transform transition-transform lg:translate-x-0 {sidebarOpen ? 'translate-x-0' : '-translate-x-full'}"
		>
			<nav class="p-4 space-y-2">
				{#each navItems as item}
					<a
						href={item.href}
						class="flex items-center gap-3 px-4 py-3 rounded-lg text-sm transition-colors {isActive(item.href)
							? 'bg-blue-50 text-blue-600 font-medium'
							: 'text-gray-700 hover:bg-gray-100'}"
						onclick={() => (sidebarOpen = false)}
					>
						<svelte:component this={item.icon} size={20} />
						<span>{item.label}</span>
					</a>
				{/each}
			</nav>
		</aside>

		<!-- Main Content -->
		<main class="flex-1 lg:ml-0">
			<div class="container mx-auto px-4 py-6 max-w-6xl">
				<slot />
			</div>
		</main>
	</div>
</div>
