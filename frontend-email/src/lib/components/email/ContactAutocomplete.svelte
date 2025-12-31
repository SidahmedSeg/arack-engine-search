<script lang="ts">
	import { cn } from '$lib/utils';
	import { onMount } from 'svelte';

	interface Contact {
		email: string;
		name?: string;
		frequency?: number; // How often this contact appears (for sorting)
	}

	interface Props {
		value?: string;
		placeholder?: string;
		class?: string;
		disabled?: boolean;
		required?: boolean;
	}

	let {
		value = $bindable(''),
		placeholder = 'Recipients (comma-separated)',
		class: className,
		disabled = false,
		required = false
	}: Props = $props();

	// Component state
	let inputRef: HTMLInputElement;
	let showDropdown = $state(false);
	let selectedIndex = $state(0);
	let contacts = $state<Contact[]>([]);
	let filteredContacts = $state<Contact[]>([]);

	// Parse the current input to get the last incomplete email
	let currentQuery = $derived(() => {
		const parts = value.split(/[,;]/).map((s) => s.trim());
		return parts[parts.length - 1];
	});

	// Filter contacts based on current query
	$effect(() => {
		const query = currentQuery().toLowerCase();
		if (query.length < 1) {
			filteredContacts = [];
			showDropdown = false;
			return;
		}

		filteredContacts = contacts
			.filter(
				(contact) =>
					contact.email.toLowerCase().includes(query) ||
					contact.name?.toLowerCase().includes(query)
			)
			.sort((a, b) => (b.frequency || 0) - (a.frequency || 0)) // Sort by frequency
			.slice(0, 5); // Limit to 5 suggestions

		showDropdown = filteredContacts.length > 0;
		selectedIndex = 0;
	});

	// Load contacts from localStorage (mock implementation)
	onMount(() => {
		// In production, this would fetch from API: /api/mail/contacts/autocomplete
		// For now, use a mock list stored in localStorage
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

	// Handle keyboard navigation
	function handleKeyDown(e: KeyboardEvent) {
		if (!showDropdown) return;

		switch (e.key) {
			case 'ArrowDown':
				e.preventDefault();
				selectedIndex = Math.min(selectedIndex + 1, filteredContacts.length - 1);
				break;
			case 'ArrowUp':
				e.preventDefault();
				selectedIndex = Math.max(selectedIndex - 1, 0);
				break;
			case 'Enter':
				e.preventDefault();
				if (filteredContacts[selectedIndex]) {
					selectContact(filteredContacts[selectedIndex]);
				}
				break;
			case 'Escape':
				e.preventDefault();
				showDropdown = false;
				break;
		}
	}

	// Select a contact
	function selectContact(contact: Contact) {
		const parts = value.split(/[,;]/).map((s) => s.trim());
		parts[parts.length - 1] = contact.email;
		value = parts.join(', ') + ', ';
		showDropdown = false;
		inputRef?.focus();
	}

	// Handle input blur (close dropdown after a delay to allow click)
	function handleBlur() {
		setTimeout(() => {
			showDropdown = false;
		}, 200);
	}

	// Handle input focus
	function handleFocus() {
		if (currentQuery().length >= 1) {
			showDropdown = filteredContacts.length > 0;
		}
	}
</script>

<div class={cn('relative', className)}>
	<input
		bind:this={inputRef}
		type="text"
		{placeholder}
		{disabled}
		{required}
		bind:value
		onkeydown={handleKeyDown}
		onblur={handleBlur}
		onfocus={handleFocus}
		class="flex h-9 w-full px-0 py-1 text-sm text-gray-900 dark:text-gray-100 placeholder:text-gray-400 dark:placeholder:text-gray-500 focus:outline-none disabled:cursor-not-allowed disabled:opacity-50"
	/>

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
