<script lang="ts">
	import { X, Send, Paperclip, Save } from 'lucide-svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import RichTextEditor from './RichTextEditor.svelte';
	import ContactAutocomplete from './ContactAutocomplete.svelte';
	import { cn } from '$lib/utils';
	import { emailStore } from '$lib/stores/email.svelte';

	interface Props {
		open?: boolean;
		onClose?: () => void;
		replyTo?: string;
		replySubject?: string;
		replyBody?: string;
	}

	let {
		open = $bindable(false),
		onClose,
		replyTo = '',
		replySubject = '',
		replyBody = ''
	}: Props = $props();

	// Form state
	let to = $state(replyTo);
	let cc = $state('');
	let subject = $state(replySubject);
	let showCC = $state(false);
	let editorRef: any;
	let sending = $state(false);
	let saveStatus = $state<string | null>(null);

	// Initialize reply body if provided
	let initialContent = $state('');
	if (replyBody) {
		initialContent = `<br><br><blockquote class="border-l-4 border-gray-300 pl-4 text-gray-600">${replyBody}</blockquote>`;
	}

	// Auto-save to drafts every 30 seconds
	let autoSaveInterval: number;
	let lastSaved = $state<Date | null>(null);

	$effect(() => {
		if (open) {
			// Start auto-save interval when composer opens
			autoSaveInterval = setInterval(() => {
				saveDraft();
			}, 30000); // 30 seconds

			return () => {
				if (autoSaveInterval) {
					clearInterval(autoSaveInterval);
				}
			};
		}
	});

	function handleClose() {
		// Ask for confirmation if there's content
		if (to || subject || editorRef?.getContent().text.trim()) {
			if (!confirm('Discard this draft?')) {
				return;
			}
		}
		resetForm();
		open = false;
		onClose?.();
	}

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) {
			handleClose();
		}
	}

	function handleEscapeKey(e: KeyboardEvent) {
		if (e.key === 'Escape' && open) {
			handleClose();
		}
	}

	async function handleSend() {
		// Validation
		if (!to.trim()) {
			alert('Please enter at least one recipient');
			return;
		}

		if (!subject.trim()) {
			if (!confirm('Send this message without a subject?')) {
				return;
			}
		}

		const content = editorRef?.getContent();
		if (!content || !content.text.trim()) {
			alert('Please enter a message');
			return;
		}

		sending = true;
		try {
			// Parse recipients (comma or semicolon separated)
			const recipients = to
				.split(/[,;]/)
				.map((email) => email.trim())
				.filter(Boolean);

			const ccRecipients = cc
				.split(/[,;]/)
				.map((email) => email.trim())
				.filter(Boolean);

			// Send email via API
			const messageId = await emailStore.sendEmail(recipients, subject, content.text);

			// Show success feedback
			saveStatus = 'Sent!';
			setTimeout(() => {
				resetForm();
				open = false;
				onClose?.();
			}, 1000);
		} catch (error) {
			alert('Failed to send email. Please try again.');
			console.error('Send error:', error);
		} finally {
			sending = false;
		}
	}

	async function saveDraft() {
		if (!to && !subject && !editorRef?.getContent().text.trim()) {
			return; // Don't save empty drafts
		}

		saveStatus = 'Saving...';
		// TODO: Implement actual draft saving to API
		// For now, just show feedback
		setTimeout(() => {
			lastSaved = new Date();
			saveStatus = 'Saved';
			setTimeout(() => {
				saveStatus = null;
			}, 2000);
		}, 500);
	}

	function resetForm() {
		to = '';
		cc = '';
		subject = '';
		showCC = false;
		saveStatus = null;
		lastSaved = null;
	}

	// Handle Cmd/Ctrl+Enter to send
	function handleKeyDown(e: KeyboardEvent) {
		if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
			e.preventDefault();
			handleSend();
		}
	}
</script>

<svelte:window onkeydown={handleEscapeKey} />

{#if open}
	<div
		class="fixed inset-0 z-50 bg-black/50 backdrop-blur-sm flex items-end sm:items-center justify-center"
		onclick={handleBackdropClick}
		role="dialog"
		aria-modal="true"
		onkeydown={handleKeyDown}
	>
		<div
			class={cn(
				'relative w-full max-w-4xl bg-white dark:bg-gray-800 rounded-t-lg sm:rounded-lg shadow-2xl overflow-hidden',
				'max-h-[90vh] flex flex-col'
			)}
		>
			<!-- Header -->
			<div
				class="flex-shrink-0 flex items-center justify-between px-6 py-4 border-b border-gray-200 dark:border-gray-700"
			>
				<h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">New Message</h2>
				<div class="flex items-center gap-2">
					{#if saveStatus}
						<span class="text-sm text-gray-500 dark:text-gray-400">{saveStatus}</span>
					{/if}
					<Button variant="ghost" size="icon" onclick={handleClose}>
						<X class="h-5 w-5" />
					</Button>
				</div>
			</div>

			<!-- Form -->
			<div class="flex-1 overflow-y-auto">
				<div class="px-6 py-4 space-y-3">
					<!-- To field -->
					<div class="flex items-center gap-2">
						<label class="text-sm font-medium text-gray-700 dark:text-gray-300 w-12">To:</label>
						<ContactAutocomplete
							bind:value={to}
							placeholder="Recipients (comma-separated)"
							class="flex-1"
						/>
						<Button variant="ghost" size="sm" onclick={() => (showCC = !showCC)}>
							Cc
						</Button>
					</div>

					<!-- CC field (conditional) -->
					{#if showCC}
						<div class="flex items-center gap-2">
							<label class="text-sm font-medium text-gray-700 dark:text-gray-300 w-12">Cc:</label>
							<ContactAutocomplete
								bind:value={cc}
								placeholder="CC recipients (comma-separated)"
								class="flex-1"
							/>
						</div>
					{/if}

					<!-- Subject field -->
					<div class="flex items-center gap-2">
						<label class="text-sm font-medium text-gray-700 dark:text-gray-300 w-12"
							>Subject:</label
						>
						<Input type="text" bind:value={subject} placeholder="Subject" class="flex-1" />
					</div>

					<!-- Rich text editor -->
					<div class="pt-2">
						<RichTextEditor bind:this={editorRef} content={initialContent} />
					</div>

					<!-- Attachments (placeholder) -->
					<div class="pt-2">
						<Button variant="ghost" size="sm" disabled>
							<Paperclip class="h-4 w-4" />
							<span>Attach files (coming soon)</span>
						</Button>
					</div>
				</div>
			</div>

			<!-- Footer -->
			<div
				class="flex-shrink-0 px-6 py-4 border-t border-gray-200 dark:border-gray-700 flex items-center justify-between bg-gray-50 dark:bg-gray-800/50"
			>
				<div class="flex items-center gap-2">
					<Button variant="primary" onclick={handleSend} disabled={sending}>
						<Send class="h-4 w-4" />
						<span>{sending ? 'Sending...' : 'Send'}</span>
					</Button>
					<Button variant="ghost" onclick={saveDraft}>
						<Save class="h-4 w-4" />
						<span>Save draft</span>
					</Button>
				</div>

				<div class="text-xs text-gray-500 dark:text-gray-400">
					{#if lastSaved}
						<span>Last saved: {lastSaved.toLocaleTimeString()}</span>
					{:else}
						<span>Cmd/Ctrl+Enter to send</span>
					{/if}
				</div>
			</div>
		</div>
	</div>
{/if}
