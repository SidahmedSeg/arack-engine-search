<script lang="ts">
	import { authStore } from '$lib/stores/auth.svelte';
	import Avatar from './ui/avatar/avatar.svelte';
	import Button from './ui/button/button.svelte';
	import { LayoutGrid } from 'lucide-svelte';

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
		</div>
	</div>
</header>
