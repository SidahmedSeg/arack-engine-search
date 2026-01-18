<script lang="ts">
	import {
		Reply,
		ReplyAll,
		Forward,
		Archive,
		Trash2,
		MoreVertical,
		Star,
		Download,
		ArrowLeft,
		Sparkles,
		ChevronDown,
		ChevronUp
	} from 'lucide-svelte';
	import Avatar from '$lib/components/ui/Avatar.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import * as Card from '$lib/components/ui/card';
	import { formatTimestamp } from '$lib/utils';
	import { emailAPI, type Email, type SummarizeResponse } from '$lib/api/client';
	import { emailStore } from '$lib/stores/email.svelte';
	import DOMPurify from 'dompurify';

	interface Props {
		message: Email | null;
		onBack?: () => void;
		onReply?: () => void;
	}

	let { message, onBack, onReply }: Props = $props();

	// Summarization state
	let summary = $state<SummarizeResponse | null>(null);
	let isSummarizing = $state(false);
	let summarizeError = $state<string | null>(null);
	let showSummary = $state(false);

	function sanitizeHTML(html: string): string {
		return DOMPurify.sanitize(html, {
			ALLOWED_TAGS: [
				'p',
				'br',
				'strong',
				'em',
				'u',
				'h1',
				'h2',
				'h3',
				'h4',
				'h5',
				'h6',
				'ul',
				'ol',
				'li',
				'a',
				'blockquote',
				'code',
				'pre',
				'img',
				'div',
				'span'
			],
			ALLOWED_ATTR: ['href', 'src', 'alt', 'title', 'class', 'style']
		});
	}

	function formatFileSize(bytes: number): string {
		if (bytes === 0) return '0 B';
		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return Math.round(bytes / Math.pow(k, i)) + ' ' + sizes[i];
	}

	async function handleSummarize() {
		if (!message) return;

		// Toggle if already showing summary
		if (showSummary && summary) {
			showSummary = !showSummary;
			return;
		}

		isSummarizing = true;
		summarizeError = null;

		try {
			summary = await emailAPI.summarizeThread(emailStore.accountId, {
				email_ids: [message.id] // Single email for now, can extend to threads
			});
			showSummary = true;
		} catch (err: any) {
			summarizeError = err.message || 'Failed to generate summary';
			console.error('Summarize error:', err);
		} finally {
			isSummarizing = false;
		}
	}

	function handleReply() {
		if (!message) return;
		emailStore.startReply(message);
		onReply?.();
	}
</script>

{#if message}
	<div class="h-full flex flex-col bg-white dark:bg-gray-800">
		<!-- Header with actions -->
		<div class="flex-shrink-0 px-4 pt-3 pb-3 border-b border-gray-200 dark:border-gray-700 flex items-center gap-2">
			<!-- Back Button -->
			<Button variant="ghost" size="icon" onclick={onBack}>
				<ArrowLeft class="h-5 w-5" />
			</Button>

			<!-- Left Actions: Archive, Delete, Star, Summarize, More -->
			<div class="flex items-center gap-1">
				<Button variant="ghost" size="sm">
					<Archive class="h-4 w-4" />
				</Button>
				<Button variant="ghost" size="sm">
					<Trash2 class="h-4 w-4" />
				</Button>
				<Button variant="ghost" size="sm">
					<Star
						class={message.is_starred ? 'h-4 w-4 fill-yellow-400 text-yellow-400' : 'h-4 w-4'}
					/>
				</Button>

				<!-- Summarize button -->
				<Button
					variant="ghost"
					size="sm"
					onclick={handleSummarize}
					disabled={isSummarizing}
					class="gap-1"
				>
					{#if isSummarizing}
						<svg class="animate-spin h-4 w-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
							<circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
							<path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
						</svg>
					{:else}
						<Sparkles class="h-4 w-4" />
					{/if}
					<span>{showSummary ? 'Hide' : 'Summarize'}</span>
				</Button>

				<Button variant="ghost" size="sm">
					<MoreVertical class="h-4 w-4" />
				</Button>
			</div>

			<div class="flex-1"></div>

			<!-- Right Actions: Reply, Reply All, Forward -->
			<div class="flex items-center gap-1">
				<Button variant="ghost" size="sm" onclick={handleReply}>
					<Reply class="h-4 w-4" />
					<span>Reply</span>
				</Button>
				<Button variant="ghost" size="sm">
					<ReplyAll class="h-4 w-4" />
					<span>Reply all</span>
				</Button>
				<Button variant="ghost" size="sm">
					<Forward class="h-4 w-4" />
					<span>Forward</span>
				</Button>
			</div>
		</div>

		<!-- Message Content -->
		<div class="flex-1 overflow-y-auto custom-scrollbar">
			<div class="px-8 py-6">
				<!-- Subject -->
				<h1 class="text-2xl font-semibold text-gray-900 dark:text-gray-100 mb-6">
					{message.subject || '(no subject)'}
				</h1>

				<!-- From/To Info -->
				<div class="flex items-start gap-4 mb-8">
					<Avatar email={message.from[0]?.email || ''} name={message.from[0]?.name} size="lg" />

					<div class="flex-1">
						<div class="flex items-center justify-between">
							<div>
								<div class="font-semibold text-gray-900 dark:text-gray-100">
									{message.from[0]?.name || message.from[0]?.email || 'Unknown'}
								</div>
								<div class="text-sm text-gray-600 dark:text-gray-400">
									{message.from[0]?.email || 'Unknown'}
								</div>
							</div>
							<div class="text-sm text-gray-600 dark:text-gray-400">
								{formatTimestamp(message.received_at)}
							</div>
						</div>

						<!-- To/CC (collapsed by default) -->
						<div class="mt-3 text-sm text-gray-600 dark:text-gray-400 space-y-1">
							{#if message.to && message.to.length > 0}
								<div class="flex gap-2">
									<span class="font-medium min-w-8">To:</span>
									<span>{message.to.map((t) => t.email).join(', ')}</span>
								</div>
							{/if}
							{#if message.cc && message.cc.length > 0}
								<div class="flex gap-2">
									<span class="font-medium min-w-8">Cc:</span>
									<span>{message.cc.map((c) => c.email).join(', ')}</span>
								</div>
							{/if}
						</div>
					</div>
				</div>

				<!-- AI Summary Card -->
				{#if showSummary && summary}
					<Card.Root class="mb-6 border-blue-200 dark:border-blue-800 bg-blue-50 dark:bg-blue-950/30">
						<Card.Header class="pb-3">
							<div class="flex items-center justify-between">
								<div class="flex items-center gap-2">
									<Sparkles class="h-4 w-4 text-blue-600 dark:text-blue-400" />
									<Card.Title class="text-base text-blue-900 dark:text-blue-100">
										AI Summary
									</Card.Title>
								</div>
								<button
									onclick={() => (showSummary = false)}
									class="text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-300 transition-colors"
								>
									<ChevronUp class="h-4 w-4" />
								</button>
							</div>
						</Card.Header>
						<Card.Content class="space-y-4">
							<!-- Summary -->
							<div>
								<p class="text-gray-700 dark:text-gray-300">
									{summary.summary}
								</p>
							</div>

							<!-- Key Points -->
							{#if summary.key_points && summary.key_points.length > 0}
								<div>
									<h4 class="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-2">
										Key Points:
									</h4>
									<ul class="space-y-1 text-sm text-gray-700 dark:text-gray-300">
										{#each summary.key_points as point}
											<li class="flex gap-2">
												<span class="text-blue-600 dark:text-blue-400">•</span>
												<span>{point}</span>
											</li>
										{/each}
									</ul>
								</div>
							{/if}

							<!-- Action Items -->
							{#if summary.action_items && summary.action_items.length > 0}
								<div>
									<h4 class="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-2">
										Action Items:
									</h4>
									<ul class="space-y-1 text-sm text-gray-700 dark:text-gray-300">
										{#each summary.action_items as item}
											<li class="flex gap-2">
												<span class="text-blue-600 dark:text-blue-400">→</span>
												<span>{item}</span>
											</li>
										{/each}
									</ul>
								</div>
							{/if}

							<!-- Token count -->
							<div class="text-xs text-gray-500 dark:text-gray-400 text-right">
								{summary.token_count} tokens used
							</div>
						</Card.Content>
					</Card.Root>
				{/if}

				{#if summarizeError}
					<div class="mb-6 p-4 bg-red-50 dark:bg-red-950/30 border border-red-200 dark:border-red-800 rounded-lg">
						<p class="text-sm text-red-700 dark:text-red-300">
							{summarizeError}
						</p>
					</div>
				{/if}

				<!-- Email Body -->
				<div
					class="prose dark:prose-invert max-w-none prose-sm prose-blue prose-img:rounded-lg prose-a:text-primary-600 dark:prose-a:text-primary-400"
				>
					{#if message.body_html}
						{@html sanitizeHTML(message.body_html)}
					{:else if message.body_text}
						<pre class="whitespace-pre-wrap font-sans">{message.body_text}</pre>
					{:else}
						<p class="text-gray-500 dark:text-gray-400 italic">No message body</p>
					{/if}
				</div>

				<!-- Attachments -->
				{#if message.attachments && message.attachments.length > 0}
					<div class="mt-8">
						<!-- Divider -->
						<div class="border-t border-gray-200 dark:border-gray-700 mb-6"></div>

						<!-- Attachments Header -->
						<div class="flex items-center gap-2 mb-4 text-gray-700 dark:text-gray-300">
							<Download class="h-4 w-4" />
							<span class="font-medium">Attachments ({message.attachments.length})</span>
						</div>

						<!-- Attachments List - Compact Design -->
						<div class="flex flex-wrap gap-2">
							{#each message.attachments as attachment}
								<div
									class="relative flex items-center gap-2 bg-white dark:bg-gray-700 border border-gray-200 dark:border-gray-600 rounded-md px-2 py-1 text-xs max-w-[200px]"
								>
									<!-- File info -->
									<div class="flex-1 min-w-0">
										<div class="flex items-center gap-1">
											<span class="truncate font-medium text-gray-900 dark:text-gray-100">
												{attachment.filename}
											</span>
											<span class="text-gray-500 dark:text-gray-400 flex-shrink-0">
												{formatFileSize(attachment.size)}
											</span>
										</div>
									</div>
									<!-- Download button -->
									<button
										onclick={() => {
											// Download attachment via blob endpoint
											const downloadUrl = `https://api-mail.arack.io/api/mail/blobs/${attachment.blob_id}/${encodeURIComponent(attachment.filename)}`;
											window.open(downloadUrl, '_blank');
										}}
										class="flex-shrink-0 p-0.5 hover:bg-gray-200 dark:hover:bg-gray-600 rounded transition-colors"
										title="Download"
										type="button"
									>
										<Download class="h-3 w-3 text-gray-600 dark:text-gray-300" />
									</button>
								</div>
							{/each}
						</div>
					</div>
				{/if}
			</div>
		</div>
	</div>
{/if}
