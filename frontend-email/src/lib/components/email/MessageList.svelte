<script lang="ts">
	import { Star, Paperclip, ChevronLeft, ChevronRight } from 'lucide-svelte';
	import { cn, formatTimestamp, truncate } from '$lib/utils';
	import type { Email } from '$lib/api/client';

	interface Props {
		messages: Email[];
		selectedMessage: Email | null;
		onMessageSelect: (message: Email) => void;
		loading?: boolean;
	}

	let { messages, selectedMessage, onMessageSelect, loading = false }: Props = $props();

	// Selection state
	let selectedIds = $state<Set<string>>(new Set());
	let selectAll = $state(false);

	// Pagination state
	let currentPage = $state(1);
	let itemsPerPage = 50;

	function handleSelectAll() {
		if (selectAll) {
			selectedIds = new Set(messages.map((m) => m.id));
		} else {
			selectedIds = new Set();
		}
	}

	function toggleSelect(id: string) {
		const newSet = new Set(selectedIds);
		if (newSet.has(id)) {
			newSet.delete(id);
		} else {
			newSet.add(id);
		}
		selectedIds = newSet;
		selectAll = newSet.size === messages.length;
	}

	function handlePreviousPage() {
		if (currentPage > 1) {
			currentPage--;
		}
	}

	function handleNextPage() {
		const totalPages = Math.ceil(messages.length / itemsPerPage);
		if (currentPage < totalPages) {
			currentPage++;
		}
	}

	$effect(() => {
		// Reset selection when messages change
		selectedIds = new Set();
		selectAll = false;
		currentPage = 1;
	});
</script>

<div class="h-full flex flex-col">
	<!-- Header with Select All, Tabs, and Pagination -->
	<div class="flex-shrink-0 border-b border-gray-200 dark:border-gray-700">
		<div class="flex items-center px-4 pt-3">
			<!-- Select All Checkbox -->
			<label class="flex items-center cursor-pointer">
				<input
					type="checkbox"
					bind:checked={selectAll}
					onchange={handleSelectAll}
					class="w-4 h-4 text-primary-600 bg-gray-100 border-gray-300 rounded focus:ring-primary-500 focus:ring-2"
				/>
			</label>

			<!-- Tabs -->
			<div class="flex-1 flex items-center ml-6">
				<button
					class="px-4 pb-3 text-sm font-medium text-primary-600 border-b-2 border-primary-600"
				>
					Primary
				</button>
			</div>

			<!-- Pagination -->
			<div class="flex items-center gap-2">
				<span class="text-xs text-gray-500 dark:text-gray-400">
					1-{Math.min(messages.length, itemsPerPage)} of {messages.length}
				</span>
				<button
					onclick={handlePreviousPage}
					disabled={currentPage === 1}
					class="p-1 hover:bg-gray-100 dark:hover:bg-gray-700 rounded disabled:opacity-50 disabled:cursor-not-allowed"
				>
					<ChevronLeft class="h-4 w-4" />
				</button>
				<button
					onclick={handleNextPage}
					disabled={currentPage >= Math.ceil(messages.length / itemsPerPage)}
					class="p-1 hover:bg-gray-100 dark:hover:bg-gray-700 rounded disabled:opacity-50 disabled:cursor-not-allowed"
				>
					<ChevronRight class="h-4 w-4" />
				</button>
			</div>
		</div>
	</div>

	<!-- Message List -->
	<div class="flex-1 overflow-y-auto custom-scrollbar">
		{#if loading}
			<div class="flex items-center justify-center h-32">
				<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"></div>
			</div>
		{:else if messages.length === 0}
			<div class="flex flex-col items-center justify-center h-32 text-gray-500 dark:text-gray-400">
				<p class="text-sm">No messages</p>
			</div>
		{:else}
			<div>
				{#each messages as message}
					<button
						onclick={() => onMessageSelect(message)}
						class={cn(
							'w-full px-4 py-2.5 flex items-center gap-3 border-b border-gray-100 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors text-left group',
							selectedMessage?.id === message.id ? 'bg-blue-50 dark:bg-blue-900/20' : '',
							!message.is_read ? 'bg-blue-50/30 dark:bg-blue-900/10' : ''
						)}
					>
						<!-- Checkbox -->
						<div class="flex-shrink-0" onclick={(e) => e.stopPropagation()} role="presentation">
							<input
								type="checkbox"
								checked={selectedIds.has(message.id)}
								onchange={() => toggleSelect(message.id)}
								class="w-4 h-4 text-primary-600 bg-gray-100 border-gray-300 rounded focus:ring-primary-500 focus:ring-2"
							/>
						</div>

						<!-- Star (always visible) -->
						<div class="flex-shrink-0">
							<div
							onclick={(e) => e.stopPropagation()}
							onkeydown={(e) => {
								if (e.key === 'Enter' || e.key === ' ') {
									e.preventDefault();
									e.stopPropagation();
								}
							}}
							role="button"
							tabindex="0"
							class="p-0.5 hover:bg-gray-200 dark:hover:bg-gray-600 rounded cursor-pointer"
						>
								<Star
									class={message.is_starred
										? 'h-4 w-4 fill-yellow-400 text-yellow-400'
										: 'h-4 w-4 text-gray-300 hover:text-yellow-400'}
								/>
							</div>
						</div>

						<!-- Sender (fixed width) -->
						<div class="w-40 flex-shrink-0">
							<div
								class={cn(
									'text-sm truncate',
									!message.is_read
										? 'font-semibold text-gray-900 dark:text-gray-100'
										: 'font-normal text-gray-700 dark:text-gray-300'
								)}
							>
								{message.from.name || message.from.email}
							</div>
						</div>

						<!-- Subject & Preview -->
						<div class="flex-1 min-w-0">
							<div class="flex items-center gap-2">
								<span
									class={cn(
										'text-sm truncate',
										!message.is_read
											? 'font-semibold text-gray-900 dark:text-gray-100'
											: 'font-normal text-gray-700 dark:text-gray-300'
									)}
								>
									{message.subject || '(no subject)'}
								</span>
								<span class="text-sm text-gray-500 dark:text-gray-400 truncate">
									— {truncate(message.preview, 60)}
								</span>
							</div>
						</div>

						<!-- Attachment Icon & Date -->
						<div class="flex-shrink-0 flex items-center gap-3">
							{#if message.has_attachments}
								<Paperclip class="h-4 w-4 text-gray-400" />
							{/if}
							<span class="text-xs text-gray-500 dark:text-gray-400 w-16 text-right">
								{formatTimestamp(message.received_at)}
							</span>
						</div>
					</button>
				{/each}
			</div>
		{/if}
	</div>
</div>
