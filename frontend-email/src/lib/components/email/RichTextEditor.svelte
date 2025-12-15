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
		Code
	} from 'lucide-svelte';
	import Button from '$lib/components/ui/Button.svelte';
	import { cn } from '$lib/utils';

	interface Props {
		content?: string;
		placeholder?: string;
		class?: string;
		onUpdate?: (html: string, text: string) => void;
	}

	let { content = '', placeholder = 'Write your message...', class: className, onUpdate }: Props = $props();

	let editorElement: HTMLElement;
	let editor: Editor | null = null;

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
				}
			},
			onUpdate: ({ editor }) => {
				if (onUpdate) {
					onUpdate(editor.getHTML(), editor.getText());
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

	// Expose methods for parent component
	export { getContent };
</script>

<div class={cn('border border-gray-300 dark:border-gray-600 rounded-md overflow-hidden', className)}>
	<!-- Toolbar -->
	<div
		class="flex items-center gap-1 px-2 py-2 border-b border-gray-300 dark:border-gray-600 bg-gray-50 dark:bg-gray-800"
	>
		<Button
			variant="ghost"
			size="icon"
			onclick={toggleBold}
			class={editor?.isActive('bold') ? 'bg-gray-200 dark:bg-gray-700' : ''}
		>
			<Bold class="h-4 w-4" />
		</Button>

		<Button
			variant="ghost"
			size="icon"
			onclick={toggleItalic}
			class={editor?.isActive('italic') ? 'bg-gray-200 dark:bg-gray-700' : ''}
		>
			<Italic class="h-4 w-4" />
		</Button>

		<div class="w-px h-6 bg-gray-300 dark:bg-gray-600 mx-1"></div>

		<Button
			variant="ghost"
			size="icon"
			onclick={toggleBulletList}
			class={editor?.isActive('bulletList') ? 'bg-gray-200 dark:bg-gray-700' : ''}
		>
			<List class="h-4 w-4" />
		</Button>

		<Button
			variant="ghost"
			size="icon"
			onclick={toggleOrderedList}
			class={editor?.isActive('orderedList') ? 'bg-gray-200 dark:bg-gray-700' : ''}
		>
			<ListOrdered class="h-4 w-4" />
		</Button>

		<div class="w-px h-6 bg-gray-300 dark:bg-gray-600 mx-1"></div>

		<Button
			variant="ghost"
			size="icon"
			onclick={setLink}
			class={editor?.isActive('link') ? 'bg-gray-200 dark:bg-gray-700' : ''}
		>
			<Link2 class="h-4 w-4" />
		</Button>

		<Button
			variant="ghost"
			size="icon"
			onclick={toggleCodeBlock}
			class={editor?.isActive('codeBlock') ? 'bg-gray-200 dark:bg-gray-700' : ''}
		>
			<Code class="h-4 w-4" />
		</Button>
	</div>

	<!-- Editor -->
	<div
		bind:this={editorElement}
		class="bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100"
	></div>
</div>
