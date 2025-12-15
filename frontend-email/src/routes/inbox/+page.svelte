<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { Mail, Search, Settings, Moon, Sun } from 'lucide-svelte';
	import MailboxList from '$lib/components/email/MailboxList.svelte';
	import MessageList from '$lib/components/email/MessageList.svelte';
	import MessageDetail from '$lib/components/email/MessageDetail.svelte';
	import Composer from '$lib/components/email/Composer.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import { emailStore } from '$lib/stores/email.svelte';
	import { ShortcutManager, EMAIL_SHORTCUTS } from '$lib/utils/shortcuts';

	let searchQuery = $state('');
	let darkMode = $state(false);
	let composerOpen = $state(false);
	let shortcutManager: ShortcutManager;

	onMount(async () => {
		// Load dark mode preference
		darkMode = localStorage.getItem('darkMode') === 'true';

		// Load mailboxes and messages
		await emailStore.loadMailboxes();
		await emailStore.loadMessages('inbox');

		// Setup keyboard shortcuts
		shortcutManager = new ShortcutManager();

		// Compose (c)
		shortcutManager.register(EMAIL_SHORTCUTS.COMPOSE, 'Compose new email', () => {
			composerOpen = true;
		});

		// Next message (j)
		shortcutManager.register(EMAIL_SHORTCUTS.NEXT_MESSAGE, 'Next message', () => {
			selectNextMessage();
		});

		// Previous message (k)
		shortcutManager.register(EMAIL_SHORTCUTS.PREV_MESSAGE, 'Previous message', () => {
			selectPreviousMessage();
		});

		// Search (/)
		shortcutManager.register(EMAIL_SHORTCUTS.SEARCH, 'Focus search', () => {
			const searchInput = document.querySelector('input[type="search"]') as HTMLInputElement;
			if (searchInput) {
				searchInput.focus();
			}
		});
	});

	onDestroy(() => {
		if (shortcutManager) {
			shortcutManager.destroy();
		}
	});

	function toggleDarkMode() {
		darkMode = !darkMode;
		if (typeof window !== 'undefined' && (window as any).toggleDarkMode) {
			(window as any).toggleDarkMode();
		}
	}

	async function handleMailboxSelect(mailboxId: string) {
		await emailStore.loadMessages(mailboxId);
		emailStore.clearSelection();
	}

	function handleMessageSelect(message: any) {
		emailStore.selectMessage(message);
	}

	function handleBackToList() {
		emailStore.clearSelection();
	}

	async function handleSearch(e: Event) {
		e.preventDefault();
		if (searchQuery.trim()) {
			await emailStore.searchEmails(searchQuery);
		} else {
			await emailStore.loadMessages(emailStore.currentMailbox);
		}
	}

	function selectNextMessage() {
		const messages = emailStore.messages;
		const currentIndex = messages.findIndex(m => m.id === emailStore.selectedMessage?.id);
		if (currentIndex < messages.length - 1) {
			emailStore.selectMessage(messages[currentIndex + 1]);
		}
	}

	function selectPreviousMessage() {
		const messages = emailStore.messages;
		const currentIndex = messages.findIndex(m => m.id === emailStore.selectedMessage?.id);
		if (currentIndex > 0) {
			emailStore.selectMessage(messages[currentIndex - 1]);
		}
	}

	function handleComposerClose() {
		composerOpen = false;
		// Reload messages to show newly sent email
		emailStore.loadMessages(emailStore.currentMailbox);
	}
</script>

<svelte:head>
	<title>Inbox - Arack Mail</title>
</svelte:head>

<div class="h-screen flex flex-col overflow-hidden bg-gray-50 dark:bg-gray-900">
	<!-- Header -->
	<header class="flex-shrink-0 h-16" style="background-color: #F8FAFD;">
		<div class="h-full px-6 flex items-center gap-4">
			<!-- Logo -->
			<div class="flex items-center">
				<img src="/arackmail.svg" alt="Arack Mail" class="h-6" />
			</div>

			<!-- Search -->
			<form onsubmit={handleSearch} class="flex-1 max-w-2xl mx-4">
				<div class="relative">
					<Search class="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400" />
					<Input
						type="search"
						placeholder="Search mail"
						bind:value={searchQuery}
						class="pl-10 w-full bg-white dark:bg-gray-700 border-transparent focus:ring-2 focus:ring-primary-500 rounded-full"
					/>
				</div>
			</form>

			<!-- Right Actions -->
			<div class="flex items-center gap-2 ml-auto">
				<Button variant="ghost" size="icon" onclick={toggleDarkMode}>
					{#if darkMode}
						<Sun class="h-5 w-5" />
					{:else}
						<Moon class="h-5 w-5" />
					{/if}
				</Button>
				<Button variant="ghost" size="icon">
					<Settings class="h-5 w-5" />
				</Button>
			</div>
		</div>
	</header>

	<!-- Main Layout -->
	<div class="flex-1 flex overflow-hidden">
		<!-- Left Sidebar - Mailboxes -->
		<MailboxList
			mailboxes={emailStore.mailboxes}
			currentMailbox={emailStore.currentMailbox}
			onMailboxSelect={handleMailboxSelect}
			onCompose={() => (composerOpen = true)}
		/>

		<!-- Main Content Area - Email List or Detail -->
		<div class="flex-1 overflow-hidden pt-4 pr-6 pb-6">
			<div class="h-full bg-white dark:bg-gray-800 rounded-2xl shadow-sm overflow-hidden">
				{#if emailStore.selectedMessage}
					<MessageDetail message={emailStore.selectedMessage} onBack={handleBackToList} />
				{:else}
					<MessageList
						messages={emailStore.messages}
						selectedMessage={emailStore.selectedMessage}
						onMessageSelect={handleMessageSelect}
						loading={emailStore.loading}
					/>
				{/if}
			</div>
		</div>
	</div>

	<!-- Composer Modal -->
	<Composer bind:open={composerOpen} onClose={handleComposerClose} />

	<!-- Keyboard Shortcuts Help (bottom right corner) -->
	<div class="fixed bottom-4 right-4 text-xs text-gray-500 dark:text-gray-400">
		<details class="bg-white dark:bg-gray-800 p-2 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700">
			<summary class="cursor-pointer font-medium">Keyboard shortcuts</summary>
			<div class="mt-2 space-y-1">
				<div class="flex justify-between gap-4">
					<span>Compose</span>
					<kbd class="px-2 py-0.5 bg-gray-100 dark:bg-gray-700 rounded">c</kbd>
				</div>
				<div class="flex justify-between gap-4">
					<span>Next message</span>
					<kbd class="px-2 py-0.5 bg-gray-100 dark:bg-gray-700 rounded">j</kbd>
				</div>
				<div class="flex justify-between gap-4">
					<span>Previous message</span>
					<kbd class="px-2 py-0.5 bg-gray-100 dark:bg-gray-700 rounded">k</kbd>
				</div>
				<div class="flex justify-between gap-4">
					<span>Search</span>
					<kbd class="px-2 py-0.5 bg-gray-100 dark:bg-gray-700 rounded">/</kbd>
				</div>
			</div>
		</details>
	</div>
</div>
