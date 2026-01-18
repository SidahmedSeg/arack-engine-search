<script lang="ts">
	import { X, Send, Paperclip, Save, Maximize2, Minimize2 } from 'lucide-svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import Input from '$lib/components/ui/Input.svelte';
	import RichTextEditor from './RichTextEditor.svelte';
	import EmailChipInput from './EmailChipInput.svelte';
	import { cn } from '$lib/utils';
	import { emailStore } from '$lib/stores/email.svelte';
	import { emailAPI, type AttachmentInfo } from '$lib/api/client';

	interface Props {
		open?: boolean;
		onClose?: () => void;
	}

	let { open = $bindable(false), onClose }: Props = $props();

	// Attachment state
	interface Attachment {
		file: File;
		id: string;
		progress: number;
		uploaded: boolean;
		blob_id?: string;
		filename: string;
		content_type: string;
		size: number;
		error?: string;
	}

	// Form state - using arrays for email chips
	let toEmails = $state<string[]>([]);
	let ccEmails = $state<string[]>([]);
	let subject = $state('');
	let showCC = $state(false);
	let isExpanded = $state(false);
	let editorRef: any;
	let sending = $state(false);
	let saveStatus = $state<string | null>(null);
	let attachments = $state<Attachment[]>([]);
	let fileInputRef: HTMLInputElement;

	// Initialize from reply context if present
	let initialContent = $state('');
	let replyContextPopulated = $state(false);

	// Effect to populate form when reply context changes
	$effect(() => {
		if (open && emailStore.replyContext && !replyContextPopulated) {
			console.log('[Composer] $effect triggered - populating from reply context');
			toEmails = [...emailStore.replyContext.to];
			subject = emailStore.replyContext.subject;
			initialContent = emailStore.replyContext.quotedBody;
			replyContextPopulated = true;

			console.log('[Composer] Populated from reply context:', {
				to: toEmails,
				subject,
				contentLength: emailStore.replyContext.quotedBody.length
			});
		} else if (!open) {
			// Reset flag when composer closes
			replyContextPopulated = false;
		}
	});

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
		if (
			toEmails.length > 0 ||
			subject ||
			editorRef?.getContent().text.trim() ||
			attachments.length > 0
		) {
			if (!confirm('Discard this draft?')) {
				return;
			}
		}
		resetForm();
		emailStore.clearReplyContext();
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
		if (toEmails.length === 0) {
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

		// Check if any attachments are still uploading
		const uploadingAttachments = attachments.filter((a) => !a.uploaded && !a.error);
		if (uploadingAttachments.length > 0) {
			alert('Please wait for attachments to finish uploading');
			return;
		}

		// Check if any attachments failed
		const failedAttachments = attachments.filter((a) => a.error);
		if (failedAttachments.length > 0) {
			if (!confirm(`${failedAttachments.length} attachment(s) failed to upload. Send without them?`)) {
				return;
			}
		}

		sending = true;
		try {
			// Use email arrays directly (already validated as emails)
			const recipients = toEmails;
			const ccRecipients = ccEmails;

			// Map uploaded attachments to AttachmentInfo format
			const attachmentInfos: AttachmentInfo[] | undefined = attachments
				.filter((a) => a.uploaded && a.blob_id)
				.map((a) => ({
					filename: a.filename,
					content_type: a.content_type,
					size: a.size,
					blob_id: a.blob_id
				}));

			// Get In-Reply-To header from reply context if present
			const inReplyTo = emailStore.replyContext?.inReplyTo;

			// Send email via API with attachments and reply header
			const messageId = await emailStore.sendEmail(
				recipients,
				subject,
				content.text,
				attachmentInfos.length > 0 ? attachmentInfos : undefined,
				inReplyTo
			);

			// Clear reply context after successful send
			emailStore.clearReplyContext();

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
		if (
			toEmails.length === 0 &&
			!subject &&
			!editorRef?.getContent().text.trim() &&
			attachments.length === 0
		) {
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
		toEmails = [];
		ccEmails = [];
		subject = '';
		showCC = false;
		saveStatus = null;
		lastSaved = null;
		attachments = [];
		initialContent = '';
		replyContextPopulated = false;
	}

	// Attachment handling
	function handleAttachmentClick() {
		fileInputRef?.click();
	}

	function handleFileSelect(e: Event) {
		const input = e.target as HTMLInputElement;
		const files = input.files;
		if (!files) return;

		Array.from(files).forEach((file) => {
			const attachment: Attachment = {
				file,
				id: `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
				progress: 0,
				uploaded: false,
				filename: file.name,
				content_type: file.type || 'application/octet-stream',
				size: file.size
			};

			// Add to attachments array (create new reference for Svelte reactivity)
			attachments = [...attachments, attachment];

			// Start real upload
			uploadAttachment(attachment.id);
		});

		// Clear input so same file can be selected again
		input.value = '';
	}

	async function uploadAttachment(attachmentId: string) {
		const index = attachments.findIndex((a) => a.id === attachmentId);
		if (index === -1) return;

		const attachment = attachments[index];

		try {
			// Simulate progress updates while uploading
			const progressInterval = setInterval(() => {
				const idx = attachments.findIndex((a) => a.id === attachmentId);
				if (idx !== -1 && attachments[idx].progress < 90) {
					attachments[idx] = {
						...attachments[idx],
						progress: Math.min(90, attachments[idx].progress + 10)
					};
					attachments = [...attachments];
				}
			}, 100);

			// Upload file to backend
			const result = await emailAPI.uploadBlob(attachment.file, emailStore.accountId!);

			clearInterval(progressInterval);

			// Update attachment with blob_id and mark as uploaded
			const finalIndex = attachments.findIndex((a) => a.id === attachmentId);
			if (finalIndex !== -1) {
				attachments[finalIndex] = {
					...attachments[finalIndex],
					blob_id: result.blob_id,
					uploaded: true,
					progress: 100
				};
				attachments = [...attachments];
			}
		} catch (error: any) {
			console.error('Upload error:', error);
			const finalIndex = attachments.findIndex((a) => a.id === attachmentId);
			if (finalIndex !== -1) {
				attachments[finalIndex] = {
					...attachments[finalIndex],
					error: error.message || 'Upload failed',
					progress: 0
				};
				attachments = [...attachments];
			}
		}
	}

	function removeAttachment(attachmentId: string) {
		attachments = attachments.filter((a) => a.id !== attachmentId);
	}

	function formatFileSize(bytes: number): string {
		if (bytes === 0) return '0 B';
		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return Math.round(bytes / Math.pow(k, i)) + ' ' + sizes[i];
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
		class={cn(
			'fixed inset-0 z-50',
			isExpanded ? 'bg-black/20 backdrop-blur-sm flex items-center justify-center' : 'pointer-events-none'
		)}
		role="dialog"
		aria-modal="true"
		onkeydown={handleKeyDown}
		onclick={(e) => isExpanded && handleBackdropClick(e)}
	>
		<div
			class={cn(
				'bg-white dark:bg-gray-800 overflow-hidden pointer-events-auto flex flex-col shadow-lg',
				isExpanded
					? 'w-full max-w-4xl rounded-lg max-h-[85vh]'
					: 'absolute bottom-0 right-[30px] w-[540px] rounded-t-lg h-[calc(100vh-100px)] max-h-[700px]'
			)}
		>
			<!-- Header -->
			<div
				class="flex-shrink-0 flex items-center justify-between px-4 py-2"
				style="background-color: #F1F4FA;"
			>
				<h2 class="text-sm font-medium text-gray-900 dark:text-gray-100">New message</h2>
				<div class="flex items-center gap-1">
					{#if saveStatus}
						<span class="text-xs text-gray-500 dark:text-gray-400">{saveStatus}</span>
					{/if}
					<button
						onclick={() => (isExpanded = !isExpanded)}
						class="p-1 hover:bg-gray-200 dark:hover:bg-gray-600 rounded transition-colors"
						title={isExpanded ? 'Minimize' : 'Expand'}
					>
						{#if isExpanded}
							<Minimize2 class="h-4 w-4 text-gray-600 dark:text-gray-300" />
						{:else}
							<Maximize2 class="h-4 w-4 text-gray-600 dark:text-gray-300" />
						{/if}
					</button>
					<button
						onclick={handleClose}
						class="p-1 hover:bg-gray-200 dark:hover:bg-gray-600 rounded transition-colors"
						title="Close"
					>
						<X class="h-4 w-4 text-gray-600 dark:text-gray-300" />
					</button>
				</div>
			</div>

			<!-- Form -->
			<div class="flex-1 overflow-y-auto">
				<div class="space-y-0">
					<!-- To field -->
					<div class="py-1 pl-4">
						<div class="flex items-center pb-1 border-b border-gray-200 dark:border-gray-700">
							<EmailChipInput
								bind:emails={toEmails}
								placeholder="To"
								class="flex-1 bg-transparent"
							/>
							<button
								onclick={() => (showCC = !showCC)}
								class="text-xs text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-100 px-4 flex-shrink-0"
							>
								Cc
							</button>
						</div>
					</div>

					<!-- CC field (conditional) -->
					{#if showCC}
						<div class="py-1 px-4">
							<div class="pb-1 border-b border-gray-200 dark:border-gray-700">
								<EmailChipInput
									bind:emails={ccEmails}
									placeholder="Cc"
									class="w-full bg-transparent"
								/>
							</div>
						</div>
					{/if}

					<!-- Subject field -->
					<div class="py-1 px-4">
						<div class="pb-1 border-b border-gray-200 dark:border-gray-700">
							<Input
								type="text"
								bind:value={subject}
								placeholder="Subject"
								class="w-full border-0 focus:ring-0 px-0 bg-transparent h-auto py-0"
							/>
						</div>
					</div>

					<!-- Rich text editor -->
					<div>
						<RichTextEditor
							bind:this={editorRef}
							content={initialContent}
							enableSmartCompose={true}
							accountId={emailStore.accountId}
							{subject}
							recipient={toEmails.join(', ')}
							isReply={!!emailStore.replyContext}
						/>
					</div>
				</div>
			</div>

			<!-- Attachment section (just above footer) -->
			{#if attachments.length > 0}
				<div class="flex-shrink-0 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-800 px-4 py-3">
					<div class="flex items-center gap-2 mb-2 text-sm text-gray-700 dark:text-gray-300">
						<Paperclip class="h-4 w-4" />
						<span class="font-medium">Attachments ({attachments.length})</span>
					</div>
					<div class="flex flex-wrap gap-2">
						{#each attachments as attachment (attachment.id)}
							<div
								class="relative flex items-center gap-2 bg-white dark:bg-gray-700 border border-gray-200 dark:border-gray-600 rounded-md px-2 py-1 text-xs max-w-[200px]"
							>
								<!-- File info -->
								<div class="flex-1 min-w-0">
									<div class="flex items-center gap-1">
										<span class="truncate font-medium text-gray-900 dark:text-gray-100">
											{attachment.file.name}
										</span>
										<span class="text-gray-500 dark:text-gray-400 flex-shrink-0">
											{formatFileSize(attachment.file.size)}
										</span>
									</div>
									<!-- Progress bar (only show if not uploaded) -->
									{#if !attachment.uploaded}
										<div class="mt-1 w-full bg-gray-200 dark:bg-gray-600 rounded-full h-1">
											<div
												class="bg-blue-600 dark:bg-blue-500 h-1 rounded-full transition-all"
												style="width: {attachment.progress}%"
											></div>
										</div>
									{/if}
								</div>
								<!-- Remove button -->
								<button
									onclick={() => removeAttachment(attachment.id)}
									class="flex-shrink-0 p-0.5 hover:bg-gray-200 dark:hover:bg-gray-600 rounded transition-colors"
									title="Remove"
									type="button"
								>
									<X class="h-3 w-3 text-gray-600 dark:text-gray-300" />
								</button>
							</div>
						{/each}
					</div>
				</div>
			{/if}

			<!-- Footer -->
			<div
				class="flex-shrink-0 px-4 py-3 flex items-center justify-between"
				style="background-color: #F1F4FA;"
			>
				<div class="flex items-center gap-2">
					<Button variant="primary" onclick={handleSend} disabled={sending} class="text-sm">
						{sending ? 'Sending...' : 'Send'}
					</Button>
					<button
						onclick={handleAttachmentClick}
						class="p-2 hover:bg-gray-200 dark:hover:bg-gray-600 rounded transition-colors"
						title="Attach files"
						type="button"
					>
						<Paperclip class="h-4 w-4 text-gray-600 dark:text-gray-300" />
					</button>
					<!-- Hidden file input -->
					<input
						bind:this={fileInputRef}
						type="file"
						multiple
						onchange={handleFileSelect}
						class="hidden"
						accept="*/*"
					/>
				</div>

				<div class="text-xs text-gray-500 dark:text-gray-400">
					{#if lastSaved}
						<span>Saved {lastSaved.toLocaleTimeString()}</span>
					{:else if saveStatus}
						<span>{saveStatus}</span>
					{/if}
				</div>
			</div>
		</div>
	</div>
{/if}
