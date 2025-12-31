<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { Search, Settings, Moon, Sun, Wifi, WifiOff, Sparkles } from 'lucide-svelte';
	import MailboxList from '$lib/components/email/MailboxList.svelte';
	import MessageList from '$lib/components/email/MessageList.svelte';
	import MessageDetail from '$lib/components/email/MessageDetail.svelte';
	import Composer from '$lib/components/email/Composer.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import { emailStore } from '$lib/stores/email.svelte';
	import {
		realtimeStore,
		type NewEmailEvent,
		type EmailUpdatedEvent,
		type MailboxUpdatedEvent
	} from '$lib/stores/realtime.svelte';
	import { ShortcutManager, EMAIL_SHORTCUTS } from '$lib/utils/shortcuts';
	import { emailAPI, type PriorityEmail, type Email } from '$lib/api/client';
	import { goto } from '$app/navigation';

	let searchQuery = $state('');
	let darkMode = $state(false);
	let composerOpen = $state(false);
	let shortcutManager: ShortcutManager;

	// Priority inbox state
	let priorityEmails = $state<PriorityEmail[]>([]);
	let rankedMessages = $state<Email[]>([]);
	let loading = $state(false);
	let error = $state<string | null>(null);

	onMount(async () => {
		// Session auth: If user is not logged in, API calls will return 401
		// and main +page.svelte will redirect to Kratos login

		darkMode = localStorage.getItem('darkMode') === 'true';

		// Wait for account to be initialized, then load messages and rankings
		const checkAccount = setInterval(async () => {
			if (emailStore.accountId) {
				clearInterval(checkAccount);
				await emailStore.loadMessages('inbox');
				await loadPriorityRankings();
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
		// Reload priority rankings when new email arrives
		loadPriorityRankings();
	}

	function handleEmailUpdated(event: EmailUpdatedEvent) {
		emailStore.handleEmailUpdated(event.email_id, event.update_type);
	}

	function handleMailboxUpdated(event: MailboxUpdatedEvent) {
		emailStore.handleMailboxUpdated(event.mailbox_id, event.action);
	}

	async function loadPriorityRankings() {
		loading = true;
		error = null;

		try {
			// Get email IDs from current inbox messages
			const emailIds = emailStore.messages.map((m) => m.id);

			if (emailIds.length === 0) {
				rankedMessages = [];
				return;
			}

			// Get priority rankings from AI
			const result = await emailAPI.priorityRank(emailStore.accountId, {
				mailbox_id: 'inbox',
				email_ids: emailIds
			});

			priorityEmails = result.ranked_emails;

			// Sort messages by priority score
			rankedMessages = emailStore.messages
				.map((message) => {
					const priority = priorityEmails.find((p) => p.email_id === message.id);
					return {
						...message,
						priority_score: priority?.priority_score || 5,
						priority_reason: priority?.reason || ''
					};
				})
				.sort((a: any, b: any) => b.priority_score - a.priority_score);
		} catch (err: any) {
			error = err.message || 'Failed to load priority rankings';
			console.error('Priority ranking error:', err);
			// Fallback to regular message order
			rankedMessages = emailStore.messages;
		} finally {
			loading = false;
		}
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
		// Reload priority rankings for new mailbox
		await loadPriorityRankings();
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
			await loadPriorityRankings();
		} else {
			await emailStore.loadMessages(emailStore.currentMailbox);
			await loadPriorityRankings();
		}
	}

	function selectNextMessage() {
		const messages = rankedMessages;
		const currentIndex = messages.findIndex((m: any) => m.id === emailStore.selectedMessage?.id);
		if (currentIndex < messages.length - 1) {
			emailStore.selectMessage(messages[currentIndex + 1]);
		}
	}

	function selectPreviousMessage() {
		const messages = rankedMessages;
		const currentIndex = messages.findIndex((m: any) => m.id === emailStore.selectedMessage?.id);
		if (currentIndex > 0) {
			emailStore.selectMessage(messages[currentIndex - 1]);
		}
	}

	function handleComposerClose() {
		composerOpen = false;
		emailStore.loadMessages(emailStore.currentMailbox);
		loadPriorityRankings();
	}

	function getPriorityColor(score: number): string {
		if (score >= 8) return 'text-red-600 dark:text-red-400';
		if (score >= 6) return 'text-orange-600 dark:text-orange-400';
		return 'text-gray-600 dark:text-gray-400';
	}

	function getPriorityBgColor(score: number): string {
		if (score >= 8) return 'bg-red-100 dark:bg-red-950/30';
		if (score >= 6) return 'bg-orange-100 dark:bg-orange-950/30';
		return 'bg-gray-100 dark:bg-gray-800';
	}
</script>

<svelte:head>
	<title>Priority Inbox - Arack Mail</title>
</svelte:head>

<div class="h-screen flex flex-col overflow-hidden bg-gray-50 dark:bg-gray-900">
	<!-- Header -->
	<header class="flex-shrink-0 h-16" style="background-color: #F8FAFD;">
		<div class="h-full px-6 flex items-center gap-4">
			<div class="flex items-center gap-2">
				<img src="/arackmail.svg" alt="Arack Mail" class="h-6" />
				<Sparkles class="h-5 w-5 text-blue-600 dark:text-blue-400" />
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
				<div
					class="flex items-center gap-1 px-2 py-1 rounded-full text-xs {realtimeStore.connected
						? 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300'
						: realtimeStore.connecting
							? 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900 dark:text-yellow-300'
							: 'bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-300'}"
				>
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
			currentMailbox="priority"
			onMailboxSelect={handleMailboxSelect}
			onCompose={() => (composerOpen = true)}
		/>

		<div class="flex-1 overflow-hidden pt-4 pr-6 pb-6">
			<div class="h-full bg-white dark:bg-gray-800 rounded-2xl shadow-sm overflow-hidden">
				{#if emailStore.selectedMessage}
					<MessageDetail message={emailStore.selectedMessage} onBack={handleBackToList} />
				{:else}
					<!-- Priority Message List -->
					<div class="h-full flex flex-col">
						<!-- Header -->
						<div class="flex-shrink-0 px-6 py-4 border-b border-gray-200 dark:border-gray-700">
							<div class="flex items-center justify-between">
								<div class="flex items-center gap-2">
									<Sparkles class="h-5 w-5 text-blue-600 dark:text-blue-400" />
									<h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">
										Priority Inbox
									</h2>
								</div>
								<button
									onclick={loadPriorityRankings}
									disabled={loading}
									class="text-sm text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-300 disabled:opacity-50"
								>
									{loading ? 'Ranking...' : 'Refresh Rankings'}
								</button>
							</div>
							<p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
								AI-ranked emails by importance and urgency
							</p>
						</div>

						<!-- Message List with Priority Indicators -->
						<div class="flex-1 overflow-y-auto">
							{#if loading}
								<div class="flex items-center justify-center h-full">
									<div class="text-center">
										<svg
											class="animate-spin h-8 w-8 mx-auto mb-4 text-blue-600"
											xmlns="http://www.w3.org/2000/svg"
											fill="none"
											viewBox="0 0 24 24"
										>
											<circle
												class="opacity-25"
												cx="12"
												cy="12"
												r="10"
												stroke="currentColor"
												stroke-width="4"
											></circle>
											<path
												class="opacity-75"
												fill="currentColor"
												d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
											></path>
										</svg>
										<p class="text-gray-600 dark:text-gray-400">AI is ranking your emails...</p>
									</div>
								</div>
							{:else if error}
								<div class="flex items-center justify-center h-full">
									<div class="text-center px-8">
										<p class="text-red-600 dark:text-red-400 mb-4">{error}</p>
										<Button onclick={loadPriorityRankings}>Try Again</Button>
									</div>
								</div>
							{:else if rankedMessages.length === 0}
								<div class="flex items-center justify-center h-full">
									<p class="text-gray-600 dark:text-gray-400">No emails to rank</p>
								</div>
							{:else}
								<div class="divide-y divide-gray-200 dark:divide-gray-700">
									{#each rankedMessages as message (message.id)}
										<button
											onclick={() => handleMessageSelect(message)}
											class="w-full px-6 py-4 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors text-left flex items-start gap-4"
										>
											<!-- Priority Badge -->
											<div class="flex-shrink-0">
												<div
													class="{getPriorityBgColor(
														message.priority_score
													)} px-3 py-1 rounded-full flex items-center gap-1"
												>
													<span
														class="text-lg font-bold {getPriorityColor(message.priority_score)}"
													>
														{message.priority_score}
													</span>
												</div>
											</div>

											<!-- Message Info -->
											<div class="flex-1 min-w-0">
												<div class="flex items-start justify-between gap-4 mb-1">
													<span
														class="font-semibold text-gray-900 dark:text-gray-100 {message.is_read
															? ''
															: 'font-bold'}"
													>
														{message.from.name || message.from.email}
													</span>
													<span class="text-sm text-gray-600 dark:text-gray-400 flex-shrink-0">
														{new Date(message.received_at).toLocaleDateString()}
													</span>
												</div>
												<div
													class="text-sm font-medium text-gray-900 dark:text-gray-100 mb-1 {message.is_read
														? ''
														: 'font-semibold'}"
												>
													{message.subject || '(no subject)'}
												</div>
												<div class="text-sm text-gray-600 dark:text-gray-400 mb-2">
													{message.preview}
												</div>
												<!-- Priority Reason -->
												{#if message.priority_reason}
													<div class="text-xs text-blue-600 dark:text-blue-400 flex items-center gap-1">
														<Sparkles class="h-3 w-3" />
														<span>{message.priority_reason}</span>
													</div>
												{/if}
											</div>
										</button>
									{/each}
								</div>
							{/if}
						</div>
					</div>
				{/if}
			</div>
		</div>
	</div>

	<Composer bind:open={composerOpen} onClose={handleComposerClose} />

	<div class="fixed bottom-4 right-4 text-xs text-gray-500 dark:text-gray-400">
		<details
			class="bg-white dark:bg-gray-800 p-2 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700"
		>
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
