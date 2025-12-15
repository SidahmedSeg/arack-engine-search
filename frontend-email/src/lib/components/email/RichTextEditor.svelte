<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { Editor } from '@tiptap/core';
	import StarterKit from '@tiptap/starter-kit';
	import Link from '@tiptap/extension-link';
	import Placeholder from '@tiptap/extension-placeholder';
	import {
		Bold,
		Italic,
		Underline as UnderlineIcon,
		List,
		ListOrdered,
		Link2,
		Code,
		Sparkles
	} from 'lucide-svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import SmartCompose from './SmartCompose.svelte';
	import { cn } from '$lib/utils';

	interface Props {
		content?: string;
		placeholder?: string;
		class?: string;
		onUpdate?: (html: string, text: string) => void;
		// Smart compose props
		enableSmartCompose?: boolean;
		accountId?: string;
		subject?: string;
		recipient?: string;
		isReply?: boolean;
	}

	let {
		content = '',
		placeholder = 'Write your message...',
		class: className,
		onUpdate,
		enableSmartCompose = false,
		accountId = '',
		subject = '',
		recipient = '',
		isReply = false
	}: Props = $props();

	let editorElement: HTMLElement;
	let editor: Editor | null = null;
	let smartComposeRef: any;

	onMount(() => {
		editor = new Editor({
			element: editorElement,
			extensions: [
				StarterKit.configure({
					heading: {
						levels: [1, 2, 3]
					}
				}),
				Link.configure({
					openOnClick: false,
					HTMLAttributes: {
						class: 'text-primary-600 dark:text-primary-400 underline hover:text-primary-700'
					}
				}),
				Placeholder.configure({
					placeholder
				})
			],
			content,
			editorProps: {
				attributes: {
					class:
						'prose dark:prose-invert prose-sm max-w-none focus:outline-none min-h-[200px] p-4'
				},
				handleKeyDown: (view, event) => {
					// Handle smart compose keyboard shortcuts
					if (enableSmartCompose && smartComposeRef) {
						return smartComposeRef.handleKeyDown(event);
					}
					return false;
				}
			},
			onUpdate: ({ editor }) => {
				const html = editor.getHTML();
				const text = editor.getText();

				if (onUpdate) {
					onUpdate(html, text);
				}

				// Trigger smart compose on text change
				if (enableSmartCompose && smartComposeRef) {
					smartComposeRef.onTextChange(text);
				}
			}
		});
	});

	onDestroy(() => {
		if (editor) {
			editor.destroy();
		}
	});

	function toggleBold() {
		editor?.chain().focus().toggleBold().run();
	}

	function toggleItalic() {
		editor?.chain().focus().toggleItalic().run();
	}

	function toggleBulletList() {
		editor?.chain().focus().toggleBulletList().run();
	}

	function toggleOrderedList() {
		editor?.chain().focus().toggleOrderedList().run();
	}

	function toggleCodeBlock() {
		editor?.chain().focus().toggleCodeBlock().run();
	}

	function setLink() {
		const url = window.prompt('Enter URL:');
		if (url) {
			editor?.chain().focus().setLink({ href: url }).run();
		}
	}

	function getContent() {
		return {
			html: editor?.getHTML() || '',
			text: editor?.getText() || ''
		};
	}

	// Handle smart compose suggestion acceptance
	function handleAcceptSuggestion(suggestion: string) {
		if (!editor) return;

		// Insert suggestion at the end of current content
		const currentHTML = editor.getHTML();
		editor.commands.setContent(currentHTML + ' ' + suggestion);

		// Focus editor
		editor.commands.focus('end');
	}

	// Expose methods for parent component
	export { getContent };
</script>

<div class={cn('overflow-hidden', className)}>
	<!-- Toolbar -->
	<div class="px-4 pt-2">
		<div
			class="flex items-center gap-1 px-3 py-1.5 rounded-md w-fit"
			style="background-color: #F1F4FA;"
		>
		<button
			onclick={toggleBold}
			class={cn(
				'p-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors',
				editor?.isActive('bold') && 'bg-gray-200 dark:bg-gray-700'
			)}
		>
			<Bold class="h-3.5 w-3.5 text-gray-700 dark:text-gray-300" />
		</button>

		<button
			onclick={toggleItalic}
			class={cn(
				'p-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors',
				editor?.isActive('italic') && 'bg-gray-200 dark:bg-gray-700'
			)}
		>
			<Italic class="h-3.5 w-3.5 text-gray-700 dark:text-gray-300" />
		</button>

		<div class="w-px h-4 bg-gray-300 dark:bg-gray-600 mx-0.5"></div>

		<button
			onclick={toggleBulletList}
			class={cn(
				'p-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors',
				editor?.isActive('bulletList') && 'bg-gray-200 dark:bg-gray-700'
			)}
		>
			<List class="h-3.5 w-3.5 text-gray-700 dark:text-gray-300" />
		</button>

		<button
			onclick={toggleOrderedList}
			class={cn(
				'p-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors',
				editor?.isActive('orderedList') && 'bg-gray-200 dark:bg-gray-700'
			)}
		>
			<ListOrdered class="h-3.5 w-3.5 text-gray-700 dark:text-gray-300" />
		</button>

		<div class="w-px h-4 bg-gray-300 dark:bg-gray-600 mx-0.5"></div>

		<button
			onclick={setLink}
			class={cn(
				'p-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors',
				editor?.isActive('link') && 'bg-gray-200 dark:bg-gray-700'
			)}
		>
			<Link2 class="h-3.5 w-3.5 text-gray-700 dark:text-gray-300" />
		</button>

		<button
			onclick={toggleCodeBlock}
			class={cn(
				'p-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors',
				editor?.isActive('codeBlock') && 'bg-gray-200 dark:bg-gray-700'
			)}
		>
			<Code class="h-3.5 w-3.5 text-gray-700 dark:text-gray-300" />
		</button>
		</div>
	</div>

	<!-- Editor -->
	<div
		bind:this={editorElement}
		class="bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100"
	></div>

	<!-- Smart Compose -->
	{#if enableSmartCompose && accountId}
		<div class="px-4 pb-2">
			<SmartCompose
				bind:this={smartComposeRef}
				{accountId}
				{subject}
				{recipient}
				{isReply}
				enabled={enableSmartCompose}
				onAccept={handleAcceptSuggestion}
			/>
		</div>
	{/if}
</div>
