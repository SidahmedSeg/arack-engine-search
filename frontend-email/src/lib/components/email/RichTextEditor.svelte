<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import {
		Bold,
		Italic,
		Underline,
		List,
		ListOrdered,
		Link2,
		Code,
		Type,
		ChevronDown
	} from 'lucide-svelte';
	import SmartCompose from './SmartCompose.svelte';
	import { cn } from '$lib/utils';

	interface Props {
		content?: string;
		placeholder?: string;
		class?: string;
		onUpdate?: (html: string, text: string) => void;
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

	// Module-level variable to store editor instance (bypasses Svelte reactivity)
	let editorInstance: any = null;
	let editorElement: HTMLElement;
	let smartComposeRef: any;
	let editorReady = $state(false);

	// Unique component ID for debugging
	const componentId = Math.random().toString(36).substring(7);
	console.log(`[RichTextEditor ${componentId}] Component script running`);

	// Track format states for toolbar
	let isBold = $state(false);
	let isItalic = $state(false);
	let isUnderline = $state(false);
	let isUnorderedList = $state(false);
	let isOrderedList = $state(false);
	let isLink = $state(false);
	let isCode = $state(false);

	// Font options
	let showFontMenu = $state(false);
	let currentFont = $state('Sans Serif');
	const fonts = [
		{ name: 'Sans Serif', value: 'Arial, sans-serif' },
		{ name: 'Serif', value: 'Georgia, serif' },
		{ name: 'Monospace', value: 'Courier New, monospace' },
		{ name: 'Comic Sans', value: 'Comic Sans MS, cursive' },
		{ name: 'Impact', value: 'Impact, sans-serif' },
		{ name: 'Verdana', value: 'Verdana, sans-serif' }
	];

	// Font size options
	let showSizeMenu = $state(false);
	let currentSize = $state('Normal');
	const sizes = [
		{ name: 'Small', value: '12px' },
		{ name: 'Normal', value: '14px' },
		{ name: 'Large', value: '18px' },
		{ name: 'Huge', value: '24px' }
	];

	// Track content initialization
	let lastSetContent = '';

	onMount(async () => {
		console.log(`[RichTextEditor ${componentId}] === onMount START ===`);

		try {
			console.log(`[RichTextEditor ${componentId}] editorElement:`, editorElement);

			if (!editorElement) {
				console.error(`[RichTextEditor ${componentId}] ERROR: editorElement is undefined!`);
				return;
			}

			// Dynamic imports to avoid SSR issues
			console.log(`[RichTextEditor ${componentId}] Importing Squire and DOMPurify...`);
			const [{ default: Squire }, DOMPurifyModule] = await Promise.all([
				import('squire-rte'),
				import('dompurify')
			]);
			console.log(`[RichTextEditor ${componentId}] Squire and DOMPurify imported successfully`);

			// Get DOMPurify - handle both default and named exports
			const DOMPurify = DOMPurifyModule.default || DOMPurifyModule;
			console.log(`[RichTextEditor ${componentId}] DOMPurify:`, typeof DOMPurify);

			// Store in module-level variable (not $state, just plain variable)
			console.log(`[RichTextEditor ${componentId}] Creating Squire instance...`);
			editorInstance = new Squire(editorElement, {
				blockTag: 'DIV',
				tagAttributes: {
					a: { target: '_blank', rel: 'noopener noreferrer' }
				},
				// Provide sanitizer for Squire 2.0
				sanitizeToDOMFragment: (html: string, isPaste: boolean, self: any) => {
					const doc = DOMPurify.sanitize(html, {
						RETURN_DOM_FRAGMENT: true,
						RETURN_DOM_IMPORT: true
					});
					return doc;
				}
			});

			editorReady = true;
			console.log(`[RichTextEditor ${componentId}] Squire initialized successfully!`);
			console.log(`[RichTextEditor ${componentId}] editorInstance:`, editorInstance);

			// Set initial content
			if (content) {
				editorInstance.setHTML(content);
				lastSetContent = content;
			}

			// Handle input events
			editorInstance.addEventListener('input', handleInput);
			editorInstance.addEventListener('select', updateFormatStates);
			editorInstance.addEventListener('cursor', updateFormatStates);
			editorInstance.addEventListener('pathChange', updateFormatStates);

			// Handle keyboard events for smart compose
			if (enableSmartCompose) {
				editorInstance.addEventListener('keydown', handleKeyDown);
			}

			console.log(`[RichTextEditor ${componentId}] === onMount COMPLETE ===`);
		} catch (error) {
			console.error(`[RichTextEditor ${componentId}] onMount ERROR:`, error);
		}
	});

	onDestroy(() => {
		console.log(`[RichTextEditor ${componentId}] onDestroy called`);
		if (editorInstance) {
			editorInstance.destroy();
			editorInstance = null;
		}
	});

	function handleInput() {
		if (!editorInstance) return;
		const html = editorInstance.getHTML();
		const text = getPlainText();

		if (onUpdate) {
			onUpdate(html, text);
		}

		if (enableSmartCompose && smartComposeRef) {
			smartComposeRef.onTextChange(text);
		}

		updateFormatStates();
	}

	function handleKeyDown(event: KeyboardEvent) {
		if (enableSmartCompose && smartComposeRef) {
			const handled = smartComposeRef.handleKeyDown(event);
			if (handled) {
				event.preventDefault();
			}
		}
	}

	// React to content prop changes (for reply/forward)
	// REMOVED $effect to prevent infinite loop - content is set in onMount instead

	function getPlainText(): string {
		if (!editorInstance) return '';
		try {
			const root = editorInstance.getRoot();
			return root?.innerText?.trim() || root?.textContent?.trim() || '';
		} catch (e) {
			console.error('[RichTextEditor] getPlainText error:', e);
			return '';
		}
	}

	function updateFormatStates() {
		if (!editorInstance) return;
		try {
			isBold = editorInstance.hasFormat('B') || editorInstance.hasFormat('STRONG');
			isItalic = editorInstance.hasFormat('I') || editorInstance.hasFormat('EM');
			isUnderline = editorInstance.hasFormat('U');
			isUnorderedList = editorInstance.hasFormat('UL');
			isOrderedList = editorInstance.hasFormat('OL');
			isLink = editorInstance.hasFormat('A');
			isCode = editorInstance.hasFormat('CODE') || editorInstance.hasFormat('PRE');
		} catch (e) {
			// Ignore errors during format check
		}
	}

	function execCommand(command: (editor: any) => void) {
		console.log('[RichTextEditor] execCommand called');
		console.log('[RichTextEditor] editorInstance exists:', !!editorInstance);
		console.log('[RichTextEditor] editorInstance value:', editorInstance);

		if (!editorInstance) {
			console.warn('[RichTextEditor] Editor not ready');
			return;
		}

		try {
			// Focus the editor first
			editorInstance.focus();

			// Execute command with editor reference
			command(editorInstance);
			console.log('[RichTextEditor] Command executed');

			updateFormatStates();
		} catch (e) {
			console.error('[RichTextEditor] Command error:', e);
		}
	}

	function toggleBold() {
		console.log('[RichTextEditor] toggleBold called, isBold:', isBold);
		execCommand((editor) => {
			if (isBold) {
				editor.removeBold();
			} else {
				editor.bold();
			}
		});
	}

	function toggleItalic() {
		execCommand((editor) => {
			if (isItalic) {
				editor.removeItalic();
			} else {
				editor.italic();
			}
		});
	}

	function toggleUnderline() {
		execCommand((editor) => {
			if (isUnderline) {
				editor.removeUnderline();
			} else {
				editor.underline();
			}
		});
	}

	function toggleBulletList() {
		execCommand((editor) => {
			if (isUnorderedList) {
				editor.removeList();
			} else {
				editor.makeUnorderedList();
			}
		});
	}

	function toggleOrderedList() {
		execCommand((editor) => {
			if (isOrderedList) {
				editor.removeList();
			} else {
				editor.makeOrderedList();
			}
		});
	}

	function toggleCode() {
		execCommand((editor) => {
			editor.toggleCode();
		});
	}

	function setLink() {
		if (!editorInstance) return;
		if (isLink) {
			execCommand((ed) => ed.removeLink());
		} else {
			const url = window.prompt('Enter URL:');
			if (url) {
				execCommand((ed) => ed.makeLink(url));
			}
		}
	}

	function setFont(font: { name: string; value: string }) {
		if (!editorInstance) return;
		currentFont = font.name;
		showFontMenu = false;
		execCommand((ed) => ed.setFontFace(font.value));
	}

	function setFontSize(size: { name: string; value: string }) {
		if (!editorInstance) return;
		currentSize = size.name;
		showSizeMenu = false;
		execCommand((ed) => ed.setFontSize(size.value));
	}

	function handleAcceptSuggestion(suggestion: string) {
		if (!editorInstance) return;
		editorInstance.insertHTML(' ' + suggestion);
		editorInstance.focus();
	}

	// Expose getContent method for parent component
	export function getContent() {
		// Fallback when editor is not ready - get content directly from DOM
		if (!editorInstance) {
			console.warn('[RichTextEditor] getContent called but editor is null, using DOM fallback');
			const html = editorElement?.innerHTML || '';
			const text = editorElement?.innerText?.trim() || '';
			console.log('[RichTextEditor] DOM fallback result:', { htmlLength: html.length, textLength: text.length });
			return { html, text };
		}

		const html = editorInstance.getHTML() || '';
		let text = getPlainText();

		// Fallback: strip HTML tags if needed
		if (!text && html) {
			const temp = document.createElement('div');
			temp.innerHTML = html;
			text = temp.textContent || temp.innerText || '';
			text = text.trim();
		}

		console.log('[RichTextEditor] getContent:', {
			editorExists: !!editorInstance,
			editorReady,
			htmlLength: html.length,
			textLength: text.length,
			textPreview: text.substring(0, 50)
		});

		return { html, text };
	}
</script>

<div class={cn('overflow-hidden', className)}>
	<!-- Toolbar -->
	<div class="px-4 pt-2" onclick={() => console.log('[RichTextEditor] TOOLBAR CONTAINER CLICKED')}>
		<div
			class="flex items-center gap-1 px-3 py-1.5 rounded-md w-fit flex-wrap"
			style="background-color: #F1F4FA;"
			onclick={() => console.log('[RichTextEditor] TOOLBAR INNER DIV CLICKED')}
		>
			<!-- Font Family Dropdown -->
			<div class="relative">
				<button
					type="button"
					onmousedown={(e) => e.preventDefault()}
					onclick={(e) => {
						e.stopPropagation(); // Prevent window handler from closing it
						console.log('[RichTextEditor] Font menu clicked, current:', showFontMenu);
						showFontMenu = !showFontMenu;
						showSizeMenu = false;
						console.log('[RichTextEditor] Font menu after toggle:', showFontMenu);
					}}
					class="flex items-center gap-1 px-2 py-1 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors text-xs"
					title="Font"
				>
					<Type class="h-3.5 w-3.5 text-gray-700 dark:text-gray-300" />
					<span class="text-gray-700 dark:text-gray-300 max-w-[60px] truncate">{currentFont}</span>
					<ChevronDown class="h-3 w-3 text-gray-500" />
				</button>
				{#if showFontMenu}
					<div class="absolute top-full left-0 mt-1 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-md shadow-lg z-50 min-w-[120px]" onclick={(e) => e.stopPropagation()}>
						{#each fonts as font}
							<button
								type="button"
								onmousedown={(e) => e.preventDefault()}
								onclick={() => setFont(font)}
								class="w-full text-left px-3 py-1.5 text-sm hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-300"
								style="font-family: {font.value}"
							>
								{font.name}
							</button>
						{/each}
					</div>
				{/if}
			</div>

			<!-- Font Size Dropdown -->
			<div class="relative">
				<button
					type="button"
					onmousedown={(e) => e.preventDefault()}
					onclick={(e) => {
						e.stopPropagation(); // Prevent window handler from closing it
						console.log('[RichTextEditor] Size menu clicked, current:', showSizeMenu);
						showSizeMenu = !showSizeMenu;
						showFontMenu = false;
						console.log('[RichTextEditor] Size menu after toggle:', showSizeMenu);
					}}
					class="flex items-center gap-1 px-2 py-1 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors text-xs"
					title="Font Size"
				>
					<span class="text-gray-700 dark:text-gray-300">{currentSize}</span>
					<ChevronDown class="h-3 w-3 text-gray-500" />
				</button>
				{#if showSizeMenu}
					<div class="absolute top-full left-0 mt-1 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-md shadow-lg z-50 min-w-[80px]" onclick={(e) => e.stopPropagation()}>
						{#each sizes as size}
							<button
								type="button"
								onmousedown={(e) => e.preventDefault()}
								onclick={() => setFontSize(size)}
								class="w-full text-left px-3 py-1.5 text-sm hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-300"
							>
								{size.name}
							</button>
						{/each}
					</div>
				{/if}
			</div>

			<div class="w-px h-4 bg-gray-300 dark:bg-gray-600 mx-0.5"></div>

			<!-- Bold -->
			<button
				type="button"
				onmousedown={(e) => { console.log('[RichTextEditor] BOLD MOUSEDOWN'); e.preventDefault(); }}
				onclick={(e) => { console.log('[RichTextEditor] BOLD ONCLICK'); toggleBold(); }}
				class={cn(
					'p-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors',
					isBold && 'bg-gray-200 dark:bg-gray-700'
				)}
				title="Bold (Ctrl+B)"
			>
				<Bold class="h-3.5 w-3.5 text-gray-700 dark:text-gray-300" />
			</button>

			<!-- Italic -->
			<button
				type="button"
				onmousedown={(e) => e.preventDefault()}
				onclick={toggleItalic}
				class={cn(
					'p-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors',
					isItalic && 'bg-gray-200 dark:bg-gray-700'
				)}
				title="Italic (Ctrl+I)"
			>
				<Italic class="h-3.5 w-3.5 text-gray-700 dark:text-gray-300" />
			</button>

			<!-- Underline -->
			<button
				type="button"
				onmousedown={(e) => e.preventDefault()}
				onclick={toggleUnderline}
				class={cn(
					'p-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors',
					isUnderline && 'bg-gray-200 dark:bg-gray-700'
				)}
				title="Underline (Ctrl+U)"
			>
				<Underline class="h-3.5 w-3.5 text-gray-700 dark:text-gray-300" />
			</button>

			<div class="w-px h-4 bg-gray-300 dark:bg-gray-600 mx-0.5"></div>

			<!-- Bullet List -->
			<button
				type="button"
				onmousedown={(e) => e.preventDefault()}
				onclick={toggleBulletList}
				class={cn(
					'p-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors',
					isUnorderedList && 'bg-gray-200 dark:bg-gray-700'
				)}
				title="Bullet List"
			>
				<List class="h-3.5 w-3.5 text-gray-700 dark:text-gray-300" />
			</button>

			<!-- Numbered List -->
			<button
				type="button"
				onmousedown={(e) => e.preventDefault()}
				onclick={toggleOrderedList}
				class={cn(
					'p-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors',
					isOrderedList && 'bg-gray-200 dark:bg-gray-700'
				)}
				title="Numbered List"
			>
				<ListOrdered class="h-3.5 w-3.5 text-gray-700 dark:text-gray-300" />
			</button>

			<div class="w-px h-4 bg-gray-300 dark:bg-gray-600 mx-0.5"></div>

			<!-- Link -->
			<button
				type="button"
				onmousedown={(e) => e.preventDefault()}
				onclick={setLink}
				class={cn(
					'p-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors',
					isLink && 'bg-gray-200 dark:bg-gray-700'
				)}
				title="Insert Link (Ctrl+K)"
			>
				<Link2 class="h-3.5 w-3.5 text-gray-700 dark:text-gray-300" />
			</button>

			<!-- Code -->
			<button
				type="button"
				onmousedown={(e) => e.preventDefault()}
				onclick={toggleCode}
				class={cn(
					'p-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors',
					isCode && 'bg-gray-200 dark:bg-gray-700'
				)}
				title="Code"
			>
				<Code class="h-3.5 w-3.5 text-gray-700 dark:text-gray-300" />
			</button>
		</div>
	</div>

	<!-- Editor -->
	<div
		bind:this={editorElement}
		class="squire-editor bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 min-h-[200px] p-4 focus:outline-none"
		data-placeholder={placeholder}
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

<!-- Close dropdowns when clicking outside -->
<svelte:window onclick={() => { showFontMenu = false; showSizeMenu = false; }} />

<style>
	/* Placeholder styling */
	.squire-editor:empty::before {
		content: attr(data-placeholder);
		color: #9ca3af;
		pointer-events: none;
		position: absolute;
	}

	.squire-editor:empty {
		position: relative;
	}

	.squire-editor {
		outline: none;
		line-height: 1.6;
	}

	/* Link styling */
	.squire-editor :global(a) {
		color: #2563eb;
		text-decoration: underline;
	}

	.squire-editor :global(a:hover) {
		color: #1d4ed8;
	}

	:global(.dark) .squire-editor :global(a) {
		color: #60a5fa;
	}

	/* Bold, Italic, Underline */
	.squire-editor :global(b),
	.squire-editor :global(strong) {
		font-weight: bold;
	}

	.squire-editor :global(i),
	.squire-editor :global(em) {
		font-style: italic;
	}

	.squire-editor :global(u) {
		text-decoration: underline;
	}

	/* Code styling */
	.squire-editor :global(code) {
		background-color: #f3f4f6;
		padding: 0.125rem 0.25rem;
		border-radius: 0.25rem;
		font-family: monospace;
		font-size: 0.875em;
	}

	:global(.dark) .squire-editor :global(code) {
		background-color: #374151;
	}

	.squire-editor :global(pre) {
		background-color: #f3f4f6;
		padding: 1rem;
		border-radius: 0.375rem;
		overflow-x: auto;
		font-family: monospace;
	}

	:global(.dark) .squire-editor :global(pre) {
		background-color: #1f2937;
	}

	/* List styling */
	.squire-editor :global(ul),
	.squire-editor :global(ol) {
		padding-left: 1.5rem;
		margin: 0.5rem 0;
	}

	.squire-editor :global(ul) {
		list-style-type: disc;
	}

	.squire-editor :global(ol) {
		list-style-type: decimal;
	}

	/* Blockquote for replies */
	.squire-editor :global(blockquote) {
		border-left: 3px solid #d1d5db;
		padding-left: 1rem;
		margin: 0.5rem 0;
		color: #6b7280;
	}

	:global(.dark) .squire-editor :global(blockquote) {
		border-left-color: #4b5563;
		color: #9ca3af;
	}
</style>
