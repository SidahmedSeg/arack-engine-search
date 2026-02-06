<script lang="ts">
	import { authStore } from '$lib/stores/auth.svelte';
	import Avatar from './ui/avatar/avatar.svelte';
	import Button from './ui/button/button.svelte';
	import { LayoutGrid, Clock, Bookmark, Settings, LogOut } from 'lucide-svelte';

	let showUserMenu = $state(false);

	function toggleUserMenu() {
		showUserMenu = !showUserMenu;
	}
</script>

<header class="w-full bg-white relative z-[60]">
	<div class="px-5 py-3">
		<div class="flex justify-end items-center">
			<!-- Right Side: Auth UI -->
			<div class="flex items-center gap-2">
				<!-- Always show Email and Explore buttons -->
				<button
					class="text-sm text-gray-700 hover:underline h-9 px-2"
					onclick={() => (window.location.href = 'https://mail.arack.io')}
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
					<!-- Authenticated: Grid Icon + Avatar with dropdown -->
					<button
						class="p-2 hover:bg-gray-100 rounded-full transition-colors"
						aria-label="Apps menu"
						title="Apps"
					>
						<LayoutGrid size={20} class="text-gray-700" />
					</button>

					<div class="relative">
						<Avatar user={authStore.user} size="md" onclick={toggleUserMenu} />
						{#if showUserMenu}
							<div
								class="absolute right-0 top-full mt-2 z-50"
								onmouseleave={() => (showUserMenu = false)}
							>
								<div class="bg-white rounded-xl shadow-xl border border-gray-200 overflow-hidden min-w-[280px]">
									<!-- Profile Header -->
									<div class="bg-gradient-to-br from-blue-50 to-gray-50 px-4 py-5 text-center">
										<!-- Avatar -->
										<div class="flex justify-center mb-2.5">
											<div class="w-16 h-16 rounded-full bg-gray-300 flex items-center justify-center text-2xl font-semibold text-gray-600">
												{authStore.user.firstName?.[0]?.toUpperCase() || authStore.user.email[0].toUpperCase()}
											</div>
										</div>

										<!-- Email Badge -->
										<div class="flex justify-center mb-2">
											<span class="inline-block bg-[#0059ff] text-white text-xs font-medium px-3 py-1 rounded-full">
												{authStore.user.email}
											</span>
										</div>

										<!-- Greeting -->
										<h3 class="text-base font-semibold text-gray-900">
											Hi, {authStore.user.firstName || 'there'}!
										</h3>
									</div>

									<!-- Menu Items -->
									<div class="py-1">
										<!-- Your search history -->
										<a
											href="/search-history"
											class="flex items-center gap-3 px-4 py-2.5 text-gray-900 hover:bg-gray-50 transition-colors"
										>
											<Clock size={20} class="text-gray-700" strokeWidth={1.5} />
											<span class="text-sm">Your search history</span>
										</a>

										<!-- Saved search -->
										<a
											href="/saved-searches"
											class="flex items-center gap-3 px-4 py-2.5 text-gray-900 hover:bg-gray-50 transition-colors"
										>
											<Bookmark size={20} class="text-gray-700" strokeWidth={1.5} />
											<span class="text-sm">Saved search</span>
										</a>

										<!-- Settings -->
										<a
											href="/settings"
											class="flex items-center gap-3 px-4 py-2.5 text-gray-900 hover:bg-gray-50 transition-colors"
										>
											<Settings size={20} class="text-gray-700" strokeWidth={1.5} />
											<span class="text-sm">Settings</span>
										</a>
									</div>

									<!-- Logout -->
									<div class="border-t border-gray-200 py-1">
										<button
											onclick={() => authStore.logout()}
											class="flex items-center gap-3 w-full px-4 py-2.5 text-red-600 hover:bg-red-50 transition-colors"
										>
											<LogOut size={20} strokeWidth={1.5} />
											<span class="text-sm font-medium">Logout</span>
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
		</div>
	</div>
</header>
