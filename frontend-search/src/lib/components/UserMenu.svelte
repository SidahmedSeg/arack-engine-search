<script lang="ts">
	import { User, Bookmark, History, Settings, LogOut, Moon, Sun } from 'lucide-svelte';
	import { authStore } from '$lib/stores/auth.svelte';
	import { preferencesStore } from '$lib/stores/preferences.svelte';
	import Dropdown from './ui/Dropdown.svelte';
	import Button from './ui/Button.svelte';

	async function handleLogout() {
		await authStore.logout();
	}

	async function toggleTheme() {
		await preferencesStore.toggleTheme();
	}
</script>

{#if authStore.isAuthenticated && authStore.user}
	<Dropdown>
		{#snippet trigger()}
			<button
				class="flex items-center gap-2 px-3 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-800 hover:bg-gray-200 dark:hover:bg-gray-700 rounded-md transition-colors"
			>
				<User size={18} />
				<span>{authStore.user.firstName}</span>
			</button>
		{/snippet}

		{#snippet children()}
			<div class="py-1">
				<!-- User Info -->
				<div class="px-4 py-3 border-b border-gray-200 dark:border-gray-700">
					<p class="text-sm font-medium text-gray-900 dark:text-gray-100">
						{authStore.user.firstName} {authStore.user.lastName}
					</p>
					<p class="text-xs text-gray-500 dark:text-gray-400 truncate">
						{authStore.user.email}
					</p>
				</div>

				<!-- Menu Items -->
				<div class="py-1">
					<a
						href="/saved-searches"
						class="flex items-center gap-3 px-4 py-2.5 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
					>
						<Bookmark size={16} />
						<span>Saved Searches</span>
					</a>

					<a
						href="/history"
						class="flex items-center gap-3 px-4 py-2.5 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
					>
						<History size={16} />
						<span>Search History</span>
					</a>

					<a
						href="/auth/settings"
						class="flex items-center gap-3 px-4 py-2.5 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
					>
						<Settings size={16} />
						<span>Settings</span>
					</a>
				</div>

				<!-- Theme Toggle -->
				<div class="py-1 border-t border-gray-200 dark:border-gray-700">
					<button
						onclick={toggleTheme}
						class="flex items-center gap-3 w-full px-4 py-2.5 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
					>
						{#if preferencesStore.theme === 'dark'}
							<Sun size={16} />
							<span>Light Mode</span>
						{:else}
							<Moon size={16} />
							<span>Dark Mode</span>
						{/if}
					</button>
				</div>

				<!-- Logout -->
				<div class="py-1 border-t border-gray-200 dark:border-gray-700">
					<button
						onclick={handleLogout}
						class="flex items-center gap-3 w-full px-4 py-2.5 text-sm text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 transition-colors"
					>
						<LogOut size={16} />
						<span>Logout</span>
					</button>
				</div>
			</div>
		{/snippet}
	</Dropdown>
{:else}
	<div class="flex items-center gap-2">
		<Button variant="ghost" size="sm" onclick={() => (window.location.href = '/auth/login')}>
			Login
		</Button>
		<Button variant="primary" size="sm" onclick={() => (window.location.href = '/auth/register')}>
			Sign Up
		</Button>
	</div>
{/if}
