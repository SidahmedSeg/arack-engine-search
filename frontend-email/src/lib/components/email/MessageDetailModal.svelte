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
		X
	} from 'lucide-svelte';
	import Avatar from '$lib/components/ui/Avatar.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import { formatTimestamp } from '$lib/utils';
	import type { Email } from '$lib/api/client';
	import DOMPurify from 'dompurify';

	interface Props {
		message: Email | null;
		open?: boolean;
		onClose?: () => void;
	}

	let { message, open = $bindable(false), onClose }: Props = $props();

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

	function handleClose() {
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
</script>

<svelte:window onkeydown={handleEscapeKey} />

{#if open && message}
	<div
		class="fixed inset-0 z-50 bg-black/50 backdrop-blur-sm flex items-center justify-center p-4"
		onclick={handleBackdropClick}
		role="dialog"
		aria-modal="true"
	>
		<div
			class="relative w-full max-w-4xl max-h-[90vh] bg-white dark:bg-gray-800 rounded-2xl shadow-2xl overflow-hidden flex flex-col"
		>
			<!-- Header with actions -->
			<div
				class="flex-shrink-0 px-6 py-4 border-b border-gray-200 dark:border-gray-700 flex items-center gap-2"
			>
				<Button variant="ghost" size="sm">
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

				<div class="flex-1"></div>

				<Button variant="ghost" size="icon">
					<Archive class="h-4 w-4" />
				</Button>
				<Button variant="ghost" size="icon">
					<Trash2 class="h-4 w-4" />
				</Button>
				<Button variant="ghost" size="icon">
					<Star
						class={message.is_starred ? 'h-4 w-4 fill-yellow-400 text-yellow-400' : 'h-4 w-4'}
					/>
				</Button>
				<Button variant="ghost" size="icon">
					<MoreVertical class="h-4 w-4" />
				</Button>
				<div class="w-px h-6 bg-gray-300 dark:bg-gray-600 mx-2"></div>
				<Button variant="ghost" size="icon" onclick={handleClose}>
					<X class="h-5 w-5" />
				</Button>
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
						<Avatar email={message.from.email} name={message.from.name} size="lg" />

						<div class="flex-1">
							<div class="flex items-center justify-between">
								<div>
									<div class="font-semibold text-gray-900 dark:text-gray-100">
										{message.from.name || message.from.email}
									</div>
									<div class="text-sm text-gray-600 dark:text-gray-400">
										{message.from.email}
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

					<!-- Attachments (placeholder) -->
					{#if message.has_attachments}
						<div
							class="mt-8 p-4 border border-gray-200 dark:border-gray-700 rounded-lg bg-gray-50 dark:bg-gray-700/50"
						>
							<div class="flex items-center gap-2 text-sm text-gray-700 dark:text-gray-300 mb-2">
								<Download class="h-4 w-4" />
								<span class="font-medium">Attachments</span>
							</div>
							<div class="text-sm text-gray-500 dark:text-gray-400">
								Attachment support coming in Phase 6
							</div>
						</div>
					{/if}
				</div>
			</div>
		</div>
	</div>
{/if}
