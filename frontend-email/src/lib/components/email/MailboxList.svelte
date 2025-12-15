<script lang="ts">
	import { Inbox, Send, FileText, Trash2, AlertCircle, Folder, Plus, Edit } from 'lucide-svelte';
	import { cn } from '$lib/utils';
	import Button from '$lib/components/ui/Button.svelte';
	import type { Mailbox } from '$lib/api/client';

	interface Props {
		mailboxes: Mailbox[];
		currentMailbox: string;
		onMailboxSelect: (mailboxId: string) => void;
		onCompose: () => void;
	}

	let { mailboxes, currentMailbox, onMailboxSelect, onCompose }: Props = $props();

	// System folder icons
	const folderIcons: Record<string, any> = {
		inbox: Inbox,
		sent: Send,
		drafts: FileText,
		trash: Trash2,
		junk: AlertCircle
	};

	function getIcon(role?: string) {
		if (role && folderIcons[role]) {
			return folderIcons[role];
		}
		return Folder;
	}
</script>

<div class="h-full w-64" style="background-color: #F8FAFD;">
	<!-- Mailbox List -->
	<div class="overflow-y-auto custom-scrollbar h-full py-4">
		<!-- Compose Button -->
		<div class="px-3 mb-4">
			<Button variant="primary" size="lg" onclick={onCompose} class="w-full shadow-md">
				<Edit class="h-5 w-5" />
				<span>Compose</span>
			</Button>
		</div>

		<div class="px-3 space-y-1">
			{#each mailboxes as mailbox}
				{@const Icon = getIcon(mailbox.role)}
				<button
					onclick={() => onMailboxSelect(mailbox.id)}
					class={cn(
						'w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium transition-colors',
						currentMailbox === mailbox.id
							? 'bg-primary-100 dark:bg-primary-900/30 text-primary-700 dark:text-primary-300'
							: 'text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800'
					)}
				>
					<Icon class="h-5 w-5 flex-shrink-0" />
					<span class="flex-1 text-left">{mailbox.name}</span>
					{#if mailbox.unread_emails > 0}
						<span
							class="flex-shrink-0 px-2 py-0.5 text-xs font-semibold rounded-full bg-primary-600 text-white"
						>
							{mailbox.unread_emails}
						</span>
					{/if}
				</button>
			{/each}
		</div>

		<!-- Labels Section (placeholder for Phase 6) -->
		<div class="mt-6 px-2">
			<div class="px-3 py-2 text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase">
				Labels
			</div>
			<button
				class="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm font-medium text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
			>
				<Plus class="h-4 w-4" />
				<span>Create label</span>
			</button>
		</div>
	</div>
</div>
