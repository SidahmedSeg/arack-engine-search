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
		ArrowLeft
	} from 'lucide-svelte';
	import Avatar from '$lib/components/ui/Avatar.svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import { formatTimestamp } from '$lib/utils';
	import type { Email } from '$lib/api/client';
	import DOMPurify from 'dompurify';

	interface Props {
		message: Email | null;
		onBack?: () => void;
	}

	let { message, onBack }: Props = $props();

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
</script>

{#if message}
	<div class="h-full flex flex-col bg-white dark:bg-gray-800">
		<!-- Header with actions -->
		<div class="flex-shrink-0 px-4 pt-3 pb-3 border-b border-gray-200 dark:border-gray-700 flex items-center gap-2">
			<!-- Back Button -->
			<Button variant="ghost" size="icon" onclick={onBack}>
				<ArrowLeft class="h-5 w-5" />
			</Button>

			<!-- Left Actions: Archive, Delete, Star, More -->
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
				<Button variant="ghost" size="sm">
					<MoreVertical class="h-4 w-4" />
				</Button>
			</div>

			<div class="flex-1"></div>

			<!-- Right Actions: Reply, Reply All, Forward -->
			<div class="flex items-center gap-1">
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
{/if}
