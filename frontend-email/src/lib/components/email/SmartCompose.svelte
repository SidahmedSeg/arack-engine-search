<script lang="ts">
	import { emailAPI, type SmartComposeSuggestion } from '$lib/api/client';
	import { onDestroy } from 'svelte';

	interface Props {
		accountId: string;
		subject?: string;
		recipient?: string;
		isReply?: boolean;
		enabled?: boolean;
		onAccept?: (suggestion: string) => void;
	}

	let {
		accountId,
		subject = '',
		recipient = '',
		isReply = false,
		enabled = true,
		onAccept
	}: Props = $props();

	// State
	let suggestions = $state<SmartComposeSuggestion[]>([]);
	let currentSuggestionIndex = $state(0);
	let isLoading = $state(false);
	let error = $state<string | null>(null);
	let debounceTimer: ReturnType<typeof setTimeout> | null = null;

	// Current suggestion
	let currentSuggestion = $derived(
		suggestions.length > 0 ? suggestions[currentSuggestionIndex] : null
	);

	/**
	 * Fetch smart compose suggestions
	 */
	async function fetchSuggestions(partialText: string) {
		if (!enabled || !partialText.trim() || partialText.trim().length < 10) {
			clearSuggestions();
			return;
		}

		isLoading = true;
		error = null;

		try {
			const response = await emailAPI.smartCompose(accountId, {
				partial_text: partialText,
				context: {
					subject,
					recipient,
					is_reply: isReply
				}
			});

			suggestions = response.suggestions;
			currentSuggestionIndex = 0;
		} catch (err: any) {
			error = err.message || 'Failed to fetch suggestions';
			console.error('Smart compose error:', err);
			clearSuggestions();
		} finally {
			isLoading = false;
		}
	}

	/**
	 * Handle text change with debouncing
	 */
	export function onTextChange(text: string) {
		// Clear existing timer
		if (debounceTimer) {
			clearTimeout(debounceTimer);
		}

		// Debounce for 2 seconds
		debounceTimer = setTimeout(() => {
			fetchSuggestions(text);
		}, 2000);
	}

	/**
	 * Accept current suggestion
	 */
	export function acceptSuggestion() {
		if (currentSuggestion && onAccept) {
			onAccept(currentSuggestion.text);
			clearSuggestions();
		}
	}

	/**
	 * Cycle to next suggestion
	 */
	export function nextSuggestion() {
		if (suggestions.length > 0) {
			currentSuggestionIndex = (currentSuggestionIndex + 1) % suggestions.length;
		}
	}

	/**
	 * Dismiss suggestions
	 */
	export function dismissSuggestions() {
		clearSuggestions();
	}

	/**
	 * Clear all suggestions
	 */
	function clearSuggestions() {
		suggestions = [];
		currentSuggestionIndex = 0;
		error = null;
	}

	/**
	 * Handle keyboard shortcuts
	 */
	export function handleKeyDown(event: KeyboardEvent): boolean {
		if (!currentSuggestion) return false;

		// Tab: Accept current suggestion or cycle to next
		if (event.key === 'Tab') {
			event.preventDefault();
			if (event.shiftKey) {
				// Shift+Tab: previous suggestion
				currentSuggestionIndex =
					(currentSuggestionIndex - 1 + suggestions.length) % suggestions.length;
			} else {
				// Just Tab: accept if on last suggestion, otherwise cycle
				if (currentSuggestionIndex === suggestions.length - 1) {
					acceptSuggestion();
				} else {
					nextSuggestion();
				}
			}
			return true;
		}

		// Escape: Dismiss
		if (event.key === 'Escape') {
			event.preventDefault();
			dismissSuggestions();
			return true;
		}

		return false;
	}

	// Cleanup on destroy
	onDestroy(() => {
		if (debounceTimer) {
			clearTimeout(debounceTimer);
		}
	});
</script>

<!-- Ghost text overlay -->
{#if currentSuggestion && !isLoading}
	<div class="smart-compose-overlay">
		<div class="ghost-text" role="tooltip" aria-live="polite">
			{currentSuggestion.text}
		</div>

		<!-- Hint text -->
		<div class="hint-text">
			<span class="text-xs text-gray-400 dark:text-gray-500">
				Press <kbd class="kbd">Tab</kbd> to accept
				{#if suggestions.length > 1}
					({currentSuggestionIndex + 1}/{suggestions.length})
				{/if}
				or <kbd class="kbd">Esc</kbd> to dismiss
			</span>
		</div>
	</div>
{/if}

{#if isLoading}
	<div class="loading-indicator">
		<span class="text-xs text-gray-400 dark:text-gray-500 flex items-center gap-1">
			<svg
				class="animate-spin h-3 w-3"
				xmlns="http://www.w3.org/2000/svg"
				fill="none"
				viewBox="0 0 24 24"
			>
				<circle
					class="opacity-25"
					cx="12"
					cy="12"
					r="10"
					stroke="currentColor"
					stroke-width="4"
				></circle>
				<path
					class="opacity-75"
					fill="currentColor"
					d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
				></path>
			</svg>
			AI composing...
		</span>
	</div>
{/if}

{#if error}
	<div class="error-message">
		<span class="text-xs text-red-500 dark:text-red-400">
			{error}
		</span>
	</div>
{/if}

<style>
	.smart-compose-overlay {
		position: relative;
		margin-top: 0.5rem;
	}

	.ghost-text {
		color: #9ca3af;
		font-style: italic;
		padding: 0.5rem;
		border-left: 3px solid #e5e7eb;
		margin-bottom: 0.5rem;
		background: rgba(243, 244, 246, 0.3);
		border-radius: 0.25rem;
	}

	:global(.dark) .ghost-text {
		color: #6b7280;
		border-left-color: #374151;
		background: rgba(31, 41, 55, 0.3);
	}

	.hint-text {
		text-align: right;
		padding-right: 0.5rem;
	}

	.kbd {
		display: inline-block;
		padding: 0.125rem 0.375rem;
		font-size: 0.75rem;
		font-family: monospace;
		background-color: #f3f4f6;
		border: 1px solid #d1d5db;
		border-radius: 0.25rem;
		box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
	}

	:global(.dark) .kbd {
		background-color: #374151;
		border-color: #4b5563;
	}

	.loading-indicator,
	.error-message {
		padding: 0.5rem;
		text-align: center;
	}
</style>
