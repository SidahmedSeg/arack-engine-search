<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { Search, Settings, Moon, Sun, Wifi, WifiOff } from 'lucide-svelte';
	import MailboxList from '$lib/components/email/MailboxList.svelte';
	import MessageList from '$lib/components/email/MessageList.svelte';
	import MessageDetail from '$lib/components/email/MessageDetail.svelte';
	import Composer from '$lib/components/email/Composer.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import { emailStore } from '$lib/stores/email.svelte';
	import { realtimeStore, type NewEmailEvent, type EmailUpdatedEvent, type MailboxUpdatedEvent } from '$lib/stores/realtime.svelte';
	import { ShortcutManager, EMAIL_SHORTCUTS } from '$lib/utils/shortcuts';
	import { goto } from '$app/navigation';
	import { emailAPI } from '$lib/api/client';

	let searchQuery = $state('');
	let darkMode = $state(false);
	let composerOpen = $state(false);
	let shortcutManager: ShortcutManager;

	onMount(async () => {
		// Session auth: If user is not logged in, API calls will return 401
		// and main +page.svelte will redirect to Kratos login

		darkMode = localStorage.getItem('darkMode') === 'true';

		// Wait for account to be initialized, then load messages
		const checkAccount = setInterval(() => {
			if (emailStore.accountId) {
				clearInterval(checkAccount);
				emailStore.loadMessages('drafts');
			}
		}, 100);

		setTimeout(() => clearInterval(checkAccount), 5000);

		await connectRealtime();

		shortcutManager = new ShortcutManager();

		shortcutManager.register(EMAIL_SHORTCUTS.COMPOSE, 'Compose new email', () => {
			composerOpen = true;
		});

		shortcutManager.register(EMAIL_SHORTCUTS.NEXT_MESSAGE, 'Next message', () => {
			selectNextMessage();
		});

		shortcutManager.register(EMAIL_SHORTCUTS.PREV_MESSAGE, 'Previous message', () => {
			selectPreviousMessage();
		});

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
		realtimeStore.disconnect();
	});

	async function connectRealtime() {
		await realtimeStore.connect(emailStore.userId, {
			onNewEmail: handleNewEmail,
			onEmailUpdated: handleEmailUpdated,
			onMailboxUpdated: handleMailboxUpdated,
			onConnectionStateChange: (connected) => {
				console.log('Realtime connection state:', connected ? 'connected' : 'disconnected');
			}
		});
	}

	function handleNewEmail(event: NewEmailEvent) {
		emailStore.handleNewEmail(event.email_id, event.from, event.subject, event.preview);
	}

	function handleEmailUpdated(event: EmailUpdatedEvent) {
		emailStore.handleEmailUpdated(event.email_id, event.update_type);
	}

	function handleMailboxUpdated(event: MailboxUpdatedEvent) {
		emailStore.handleMailboxUpdated(event.mailbox_id, event.action);
	}

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
		emailStore.loadMessages(emailStore.currentMailbox);
	}
</script>

<svelte:head>
	<title>Drafts - Arack Mail</title>
</svelte:head>

<div class="h-screen flex flex-col overflow-hidden bg-gray-50 dark:bg-gray-900">
	<!-- Header -->
	<header class="flex-shrink-0 h-16" style="background-color: #F8FAFD;">
		<div class="h-full px-6 flex items-center gap-4">
			<div class="flex items-center">
				<img src="/arackmail.svg" alt="Arack Mail" class="h-6" />
			</div>

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

			<div class="flex items-center gap-2 ml-auto">
				<div class="flex items-center gap-1 px-2 py-1 rounded-full text-xs {realtimeStore.connected ? 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300' : realtimeStore.connecting ? 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900 dark:text-yellow-300' : 'bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-300'}">
					{#if realtimeStore.connected}
						<Wifi class="h-3 w-3" />
						<span>Live</span>
					{:else if realtimeStore.connecting}
						<Wifi class="h-3 w-3 animate-pulse" />
						<span>Connecting...</span>
					{:else}
						<WifiOff class="h-3 w-3" />
						<span>Offline</span>
					{/if}
				</div>
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
		<MailboxList
			mailboxes={emailStore.mailboxes}
			currentMailbox={emailStore.currentMailbox}
			onMailboxSelect={handleMailboxSelect}
			onCompose={() => (composerOpen = true)}
		/>

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

	<Composer bind:open={composerOpen} onClose={handleComposerClose} />

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
