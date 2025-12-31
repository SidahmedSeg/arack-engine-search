<script lang="ts">
	import { X } from 'lucide-svelte';
	import { cn } from '$lib/utils';
	import { onMount } from 'svelte';

	interface Contact {
		email: string;
		name?: string;
		frequency?: number;
	}

	interface Props {
		emails?: string[];
		placeholder?: string;
		class?: string;
		disabled?: boolean;
		required?: boolean;
	}

	let {
		emails = $bindable([]),
		placeholder = 'Recipients',
		class: className,
		disabled = false,
		required = false
	}: Props = $props();

	// Component state
	let inputRef: HTMLInputElement;
	let inputValue = $state('');
	let showDropdown = $state(false);
	let selectedIndex = $state(0);
	let contacts = $state<Contact[]>([]);
	let filteredContacts = $state<Contact[]>([]);

	// Email validation regex
	const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

	// Filter contacts based on current input
	$effect(() => {
		const query = inputValue.toLowerCase().trim();
		if (query.length < 1) {
			filteredContacts = [];
			showDropdown = false;
			return;
		}

		// Filter out already added emails
		filteredContacts = contacts
			.filter(
				(contact) =>
					!emails.includes(contact.email) &&
					(contact.email.toLowerCase().includes(query) ||
						contact.name?.toLowerCase().includes(query))
			)
			.sort((a, b) => (b.frequency || 0) - (a.frequency || 0))
			.slice(0, 5);

		showDropdown = filteredContacts.length > 0;
		selectedIndex = 0;
	});

	// Load contacts from localStorage
	onMount(() => {
		const savedContacts = localStorage.getItem('email_contacts');
		if (savedContacts) {
			try {
				contacts = JSON.parse(savedContacts);
			} catch (e) {
				console.error('Failed to parse contacts:', e);
			}
		}

		// Default mock contacts if none exist
		if (contacts.length === 0) {
			contacts = [
				{ email: 'john.doe@example.com', name: 'John Doe', frequency: 10 },
				{ email: 'jane.smith@example.com', name: 'Jane Smith', frequency: 8 },
				{ email: 'bob.johnson@example.com', name: 'Bob Johnson', frequency: 5 },
				{ email: 'alice.williams@example.com', name: 'Alice Williams', frequency: 3 },
				{ email: 'charlie.brown@example.com', name: 'Charlie Brown', frequency: 2 }
			];
		}
	});

	// Add email to the list
	function addEmail(email: string) {
		const trimmed = email.trim().toLowerCase();
		if (trimmed && emailRegex.test(trimmed) && !emails.includes(trimmed)) {
			emails = [...emails, trimmed];
			inputValue = '';
			showDropdown = false;
		}
	}

	// Remove email from the list
	function removeEmail(email: string) {
		emails = emails.filter((e) => e !== email);
		inputRef?.focus();
	}

	// Handle keyboard events
	function handleKeyDown(e: KeyboardEvent) {
		// Handle dropdown navigation
		if (showDropdown) {
			switch (e.key) {
				case 'ArrowDown':
					e.preventDefault();
					selectedIndex = Math.min(selectedIndex + 1, filteredContacts.length - 1);
					return;
				case 'ArrowUp':
					e.preventDefault();
					selectedIndex = Math.max(selectedIndex - 1, 0);
					return;
				case 'Enter':
					e.preventDefault();
					if (filteredContacts[selectedIndex]) {
						addEmail(filteredContacts[selectedIndex].email);
					}
					return;
				case 'Escape':
					e.preventDefault();
					showDropdown = false;
					return;
			}
		}

		// Handle adding email on Space, Enter, comma, semicolon, or Tab
		if (
			e.key === ' ' ||
			e.key === 'Enter' ||
			e.key === ',' ||
			e.key === ';' ||
			e.key === 'Tab'
		) {
			const value = inputValue.trim();
			if (value) {
				e.preventDefault();
				addEmail(value);
			} else if (e.key === 'Tab') {
				// Allow tab to move focus if input is empty
				return;
			}
		}

		// Handle backspace to remove last email
		if (e.key === 'Backspace' && !inputValue && emails.length > 0) {
			emails = emails.slice(0, -1);
		}
	}

	// Select contact from dropdown
	function selectContact(contact: Contact) {
		addEmail(contact.email);
		inputRef?.focus();
	}

	// Handle blur - add email if valid
	function handleBlur() {
		setTimeout(() => {
			const value = inputValue.trim();
			if (value && emailRegex.test(value)) {
				addEmail(value);
			}
			showDropdown = false;
		}, 200);
	}

	// Handle focus
	function handleFocus() {
		if (inputValue.length >= 1) {
			showDropdown = filteredContacts.length > 0;
		}
	}

	// Handle paste - support pasting multiple emails
	function handlePaste(e: ClipboardEvent) {
		const text = e.clipboardData?.getData('text');
		if (text) {
			// Split by common delimiters
			const pastedEmails = text.split(/[,;\s\n]+/).filter((e) => e.trim());
			const validEmails = pastedEmails.filter((e) => emailRegex.test(e.trim()));

			if (validEmails.length > 0) {
				e.preventDefault();
				validEmails.forEach((email) => addEmail(email));
			}
		}
	}
</script>

<div class={cn('relative', className)}>
	<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions a11y_no_noninteractive_element_interactions -->
	<div
		class="flex flex-wrap items-center gap-1 min-h-[36px] py-1 focus-within:outline-none cursor-text"
		role="textbox"
		tabindex="-1"
		onclick={() => inputRef?.focus()}
	>
		<!-- Email chips -->
		{#each emails as email}
			<span
				class="inline-flex items-center gap-1 px-2 py-0.5 bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 rounded-full text-sm"
			>
				<span class="max-w-[200px] truncate">{email}</span>
				<button
					type="button"
					onclick={(e) => {
						e.stopPropagation();
						removeEmail(email);
					}}
					class="hover:bg-blue-200 dark:hover:bg-blue-800 rounded-full p-0.5 transition-colors"
					{disabled}
				>
					<X class="h-3 w-3" />
				</button>
			</span>
		{/each}

		<!-- Input -->
		<input
			bind:this={inputRef}
			type="text"
			{placeholder}
			{disabled}
			{required}
			bind:value={inputValue}
			onkeydown={handleKeyDown}
			onblur={handleBlur}
			onfocus={handleFocus}
			onpaste={handlePaste}
			class="flex-1 min-w-[120px] h-7 px-0 py-1 text-sm text-gray-900 dark:text-gray-100 placeholder:text-gray-400 dark:placeholder:text-gray-500 focus:outline-none disabled:cursor-not-allowed disabled:opacity-50 bg-transparent"
		/>
	</div>

	<!-- Autocomplete dropdown -->
	{#if showDropdown && filteredContacts.length > 0}
		<div
			class="absolute top-full left-0 right-0 mt-1 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-md shadow-lg z-50 max-h-60 overflow-y-auto"
		>
			{#each filteredContacts as contact, index}
				<button
					type="button"
					onclick={() => selectContact(contact)}
					class={cn(
						'w-full px-3 py-2 text-left text-sm hover:bg-gray-100 dark:hover:bg-gray-700 cursor-pointer flex items-center gap-2',
						index === selectedIndex && 'bg-gray-100 dark:bg-gray-700'
					)}
				>
					<div class="flex-1 min-w-0">
						{#if contact.name}
							<div class="font-medium text-gray-900 dark:text-gray-100 truncate">
								{contact.name}
							</div>
							<div class="text-xs text-gray-500 dark:text-gray-400 truncate">
								{contact.email}
							</div>
						{:else}
							<div class="text-gray-900 dark:text-gray-100 truncate">{contact.email}</div>
						{/if}
					</div>
				</button>
			{/each}
		</div>
	{/if}
</div>
