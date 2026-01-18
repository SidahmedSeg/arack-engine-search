import { emailAPI, type Email, type Mailbox, type AttachmentInfo } from '$lib/api/client';
import { toastStore } from './toast.svelte';

// Reply context for email replies
export interface ReplyContext {
	originalMessageId: string;
	to: string[];
	subject: string;
	inReplyTo: string;
	quotedBody: string;
}

// Email store using Svelte 5 runes
class EmailStore {
	currentMailbox = $state<string>('inbox');
	messages = $state<Email[]>([]);
	selectedMessage = $state<Email | null>(null);
	mailboxes = $state<Mailbox[]>([]);
	loading = $state(false);
	error = $state<string | null>(null);
	unreadCount = $state(0);

	// Account ID - loaded from session
	accountId = $state<string | null>(null);

	// User ID for realtime connection - derived from Kratos identity
	userId = $state<string | null>(null);

	// Account info
	accountInfo = $state<{
		email: string;
		quotaPercentage: number;
		quotaUsed: number;
		quotaTotal: number;
	} | null>(null);

	// Reply context
	replyContext = $state<ReplyContext | null>(null);

	/**
	 * Initialize account from current session
	 * Must be called on app startup
	 */
	async initialize() {
		this.loading = true;
		this.error = null;
		try {
			const result = await emailAPI.getMyAccount();
			this.accountId = result.account.id;
			this.userId = result.account.kratos_identity_id;
			this.accountInfo = {
				email: result.account.email_address,
				quotaPercentage: result.quota_percentage,
				quotaUsed: result.account.storage_used_bytes,
				quotaTotal: result.account.storage_quota_bytes
			};

			// Load mailboxes after account is initialized
			await this.loadMailboxes();
		} catch (err) {
			this.error = err instanceof Error ? err.message : 'Failed to load account';
			console.error('Error initializing email store:', err);
			throw err; // Re-throw to allow caller to handle
		} finally {
			this.loading = false;
		}
	}

	async loadMailboxes() {
		this.loading = true;
		this.error = null;
		try {
			const mailboxes = await emailAPI.getMailboxes();
			console.log('[EmailStore] Raw mailboxes from API:', mailboxes.map(m => ({ id: m.id, name: m.name, role: m.role })));

			// Sort mailboxes: system folders first in specific order, then custom folders
			const systemOrder: Record<string, number> = {
				'inbox': 0,
				'sent': 1,
				'drafts': 2,
				'trash': 3,
				'junk': 4,
				'spam': 4,
				'archive': 5
			};

			// IMPORTANT: Spread to create new array reference for Svelte 5 reactivity
		this.mailboxes = [...mailboxes].sort((a, b) => {
				// Check both role and name for matching
				const aRole = a.role?.toLowerCase() || '';
				const aName = a.name.toLowerCase();
				const bRole = b.role?.toLowerCase() || '';
				const bName = b.name.toLowerCase();

				const aOrder = systemOrder[aRole] ?? systemOrder[aName] ?? 99;
				const bOrder = systemOrder[bRole] ?? systemOrder[bName] ?? 99;

				if (aOrder !== bOrder) return aOrder - bOrder;
				return a.name.localeCompare(b.name);
			});

			console.log('[EmailStore] Sorted mailboxes:', this.mailboxes.map(m => ({ name: m.name, role: m.role })));

			// Set currentMailbox to inbox ID if not already set to a valid ID
			this.selectInbox();
		} catch (err) {
			this.error = err instanceof Error ? err.message : 'Failed to load mailboxes';
			console.error('Error loading mailboxes:', err);
		} finally {
			this.loading = false;
		}
	}

	/**
	 * Find and select the inbox mailbox
	 * This properly sets currentMailbox to the actual mailbox ID
	 */
	selectInbox() {
		const inbox = this.mailboxes.find(
			(m) => m.role?.toLowerCase() === 'inbox' || m.name.toLowerCase() === 'inbox'
		);
		if (inbox) {
			this.currentMailbox = inbox.id;
			console.log('[EmailStore] Selected inbox:', inbox.id, inbox.name);
		} else if (this.mailboxes.length > 0) {
			// Fallback to first mailbox
			this.currentMailbox = this.mailboxes[0].id;
			console.log('[EmailStore] No inbox found, selected first mailbox:', this.mailboxes[0].name);
		}
	}

	/**
	 * Get the inbox mailbox ID
	 */
	getInboxId(): string | null {
		const inbox = this.mailboxes.find(
			(m) => m.role?.toLowerCase() === 'inbox' || m.name.toLowerCase() === 'inbox'
		);
		return inbox?.id || null;
	}

	async loadMessages(mailboxId: string, limit: number = 50) {
		this.loading = true;
		this.error = null;
		this.currentMailbox = mailboxId;
		try {
			const result = await emailAPI.getMessages(mailboxId, limit);
			this.messages = result.messages;
		} catch (err) {
			this.error = err instanceof Error ? err.message : 'Failed to load messages';
			console.error('Error loading messages:', err);
		} finally {
			this.loading = false;
		}
	}

	async loadMessage(messageId: string) {
		if (!this.accountId) {
			console.error('Cannot load message: accountId not initialized');
			return;
		}
		this.loading = true;
		this.error = null;
		try {
			const message = await emailAPI.getMessage(messageId, this.accountId);
			this.selectedMessage = message;

			// Mark as read in background (don't await)
			if (!message.is_read) {
				emailAPI.markAsRead(messageId).then(() => {
					// Update local state optimistically
					const messageIndex = this.messages.findIndex(m => m.id === messageId);
					if (messageIndex !== -1) {
						this.messages[messageIndex].is_read = true;
					}
					// Update selected message state as well
					if (this.selectedMessage?.id === messageId) {
						this.selectedMessage.is_read = true;
					}
				}).catch(err => {
					console.error('Failed to mark message as read:', err);
					// Non-critical error, don't throw
				});
			}
		} catch (err) {
			this.error = err instanceof Error ? err.message : 'Failed to load message';
			console.error('Error loading message:', err);
		} finally {
			this.loading = false;
		}
	}

	selectMessage(message: Email) {
		this.selectedMessage = message;
	}

	clearSelection() {
		this.selectedMessage = null;
	}

	// Send email
	async sendEmail(
		to: string[],
		subject: string,
		bodyText: string,
		attachments?: AttachmentInfo[],
		inReplyTo?: string
	) {
		if (!this.accountId) {
			console.error('Cannot send email: accountId not initialized');
			throw new Error('Account not initialized');
		}
		this.loading = true;
		this.error = null;
		try {
			const result = await emailAPI.sendEmail(
				{
					to,
					subject,
					body_text: bodyText,
					attachments: attachments && attachments.length > 0 ? attachments : undefined,
					in_reply_to: inReplyTo
				},
				this.accountId
			);
			return result.message_id;
		} catch (err) {
			this.error = err instanceof Error ? err.message : 'Failed to send email';
			console.error('Error sending email:', err);
			throw err;
		} finally {
			this.loading = false;
		}
	}

	// Search emails
	async searchEmails(query: string) {
		if (!this.accountId) {
			console.error('Cannot search emails: accountId not initialized');
			return;
		}
		this.loading = true;
		this.error = null;
		try {
			const result = await emailAPI.searchEmails({
				query,
				account_id: this.accountId,
				limit: 50
			});
			this.messages = result.results;
		} catch (err) {
			this.error = err instanceof Error ? err.message : 'Failed to search emails';
			console.error('Error searching emails:', err);
		} finally {
			this.loading = false;
		}
	}

	// =====================
	// Real-time event handlers
	// =====================

	/**
	 * Handle new email event from Centrifugo
	 * Adds the new email to the top of the message list if in inbox
	 */
	async handleNewEmail(emailId: string, from: string, subject: string, preview: string) {
		// If we're viewing inbox, add the email to the top
		const inboxId = this.getInboxId();
		if (this.currentMailbox === inboxId) {
			// Create a placeholder email (will be replaced when fully loaded)
			const newEmail: Email = {
				id: emailId,
				subject,
				from: [{ email: from }],
				preview,
				received_at: new Date().toISOString(),
				is_read: false,
				is_starred: false,
				has_attachments: false
			};

			// Add to top of list - spread for new array reference
			this.messages = [newEmail, ...this.messages];
		}

		// Update unread count
		this.unreadCount++;

		// Refresh mailboxes to get updated counts
		this.loadMailboxes();
	}

	/**
	 * Handle email updated event from Centrifugo
	 */
	handleEmailUpdated(
		emailId: string,
		updateType: 'read' | 'unread' | 'moved' | 'deleted' | 'starred' | 'unstarred'
	) {
		const messageIndex = this.messages.findIndex((m) => m.id === emailId);
		if (messageIndex === -1) return;

		const message = this.messages[messageIndex];

		// IMPORTANT: Create new array reference for Svelte 5 reactivity
		switch (updateType) {
			case 'read':
				this.messages = this.messages.map((m, i) =>
					i === messageIndex ? { ...m, is_read: true } : m
				);
				if (!message.is_read) this.unreadCount--;
				break;
			case 'unread':
				this.messages = this.messages.map((m, i) =>
					i === messageIndex ? { ...m, is_read: false } : m
				);
				if (message.is_read) this.unreadCount++;
				break;
			case 'starred':
				this.messages = this.messages.map((m, i) =>
					i === messageIndex ? { ...m, is_starred: true } : m
				);
				break;
			case 'unstarred':
				this.messages = this.messages.map((m, i) =>
					i === messageIndex ? { ...m, is_starred: false } : m
				);
				break;
			case 'deleted':
			case 'moved':
				// Remove from current list - filter creates new array
				this.messages = this.messages.filter((m) => m.id !== emailId);
				if (!message.is_read) this.unreadCount--;
				break;
		}

		// Update selected message if it's the one being updated
		if (this.selectedMessage?.id === emailId) {
			if (updateType === 'deleted' || updateType === 'moved') {
				this.selectedMessage = null;
			} else {
				// Get updated message from the new array
				this.selectedMessage = this.messages[messageIndex] || null;
			}
		}
	}

	/**
	 * Handle mailbox updated event from Centrifugo
	 */
	handleMailboxUpdated(mailboxId: string, action: string) {
		// Refresh mailboxes to get updated state
		this.loadMailboxes();

		// If the current mailbox was affected, refresh messages
		if (this.currentMailbox === mailboxId) {
			this.loadMessages(mailboxId);
		}
	}

	/**
	 * Update unread count from mailboxes
	 */
	updateUnreadCount() {
		const inbox = this.mailboxes.find((m) => m.role === 'inbox' || m.name.toLowerCase() === 'inbox');
		this.unreadCount = inbox?.unread_emails || 0;
	}

	// =====================
	// Delete/Archive with Undo
	// =====================

	/**
	 * Move email to trash with undo capability
	 * @param messageId - ID of the message to delete
	 */
	async moveToTrash(messageId: string) {
		// Find the message
		const messageIndex = this.messages.findIndex((m) => m.id === messageId);
		if (messageIndex === -1) {
			console.error('Message not found:', messageId);
			return;
		}

		const message = this.messages[messageIndex];
		let undoCancelled = false;
		let apiCallId: number | null = null;

		// Optimistically remove from UI
		this.messages = this.messages.filter((m) => m.id !== messageId);

		// If this was the selected message, clear selection
		if (this.selectedMessage?.id === messageId) {
			this.selectedMessage = null;
		}

		// Show toast with undo action
		toastStore.success('Moved to Trash', () => {
			// Undo action: restore the message
			undoCancelled = true;

			// Cancel the API call if it hasn't executed yet
			if (apiCallId) {
				clearTimeout(apiCallId);
			}

			// Restore message to list at original position
			this.messages.splice(messageIndex, 0, message);
			this.messages = [...this.messages]; // Trigger reactivity

			console.log('[EmailStore] Undo: Restored message', messageId);
		});

		// Schedule API call after 4 seconds (gives user time to undo)
		apiCallId = setTimeout(async () => {
			if (!undoCancelled) {
				try {
					await emailAPI.moveToTrash(messageId);
					console.log('[EmailStore] Message moved to trash on server:', messageId);
				} catch (err) {
					console.error('[EmailStore] Failed to move message to trash:', err);
					// Restore message on error
					this.messages.splice(messageIndex, 0, message);
					this.messages = [...this.messages];
					toastStore.error('Failed to move to trash');
				}
			}
		}, 4000) as unknown as number;
	}

	/**
	 * Archive email with undo capability
	 * @param messageId - ID of the message to archive
	 */
	async moveToArchive(messageId: string) {
		// Find the message
		const messageIndex = this.messages.findIndex((m) => m.id === messageId);
		if (messageIndex === -1) {
			console.error('Message not found:', messageId);
			return;
		}

		const message = this.messages[messageIndex];
		let undoCancelled = false;
		let apiCallId: number | null = null;

		// Optimistically remove from UI
		this.messages = this.messages.filter((m) => m.id !== messageId);

		// If this was the selected message, clear selection
		if (this.selectedMessage?.id === messageId) {
			this.selectedMessage = null;
		}

		// Show toast with undo action
		toastStore.success('Archived', () => {
			// Undo action: restore the message
			undoCancelled = true;

			// Cancel the API call if it hasn't executed yet
			if (apiCallId) {
				clearTimeout(apiCallId);
			}

			// Restore message to list at original position
			this.messages.splice(messageIndex, 0, message);
			this.messages = [...this.messages]; // Trigger reactivity

			console.log('[EmailStore] Undo: Restored message', messageId);
		});

		// Schedule API call after 4 seconds (gives user time to undo)
		apiCallId = setTimeout(async () => {
			if (!undoCancelled) {
				try {
					await emailAPI.moveToArchive(messageId);
					console.log('[EmailStore] Message archived on server:', messageId);
				} catch (err) {
					console.error('[EmailStore] Failed to archive message:', err);
					// Restore message on error
					this.messages.splice(messageIndex, 0, message);
					this.messages = [...this.messages];
					toastStore.error('Failed to archive');
				}
			}
		}, 4000) as unknown as number;
	}

	/**
	 * Restore email from trash/archive back to inbox
	 * @param messageId - ID of the message to restore
	 */
	async restoreToInbox(messageId: string) {
		// Find the message
		const messageIndex = this.messages.findIndex((m) => m.id === messageId);
		if (messageIndex === -1) {
			console.error('Message not found:', messageId);
			return;
		}

		const message = this.messages[messageIndex];
		let undoCancelled = false;
		let apiCallId: number | null = null;

		// Optimistically remove from UI
		this.messages = this.messages.filter((m) => m.id !== messageId);

		// If this was the selected message, clear selection
		if (this.selectedMessage?.id === messageId) {
			this.selectedMessage = null;
		}

		// Show toast with undo action
		toastStore.success('Moved to Inbox', () => {
			// Undo action: restore the message
			undoCancelled = true;

			// Cancel the API call if it hasn't executed yet
			if (apiCallId) {
				clearTimeout(apiCallId);
			}

			// Restore message to list at original position
			this.messages.splice(messageIndex, 0, message);
			this.messages = [...this.messages]; // Trigger reactivity

			console.log('[EmailStore] Undo: Restored message', messageId);
		});

		// Schedule API call after 4 seconds (gives user time to undo)
		apiCallId = setTimeout(async () => {
			if (!undoCancelled) {
				try {
					await emailAPI.moveToInbox(messageId);
					console.log('[EmailStore] Message restored to inbox on server:', messageId);
				} catch (err) {
					console.error('[EmailStore] Failed to restore message to inbox:', err);
					// Restore message on error
					this.messages.splice(messageIndex, 0, message);
					this.messages = [...this.messages];
					toastStore.error('Failed to restore to inbox');
				}
			}
		}, 4000) as unknown as number;
	}

	/**
	 * Permanently delete email (only from trash)
	 * @param messageId - ID of the message to permanently delete
	 */
	async deletePermanently(messageId: string) {
		// Confirmation dialog
		if (
			!confirm(
				'Permanently delete this email? This cannot be undone.'
			)
		) {
			return;
		}

		// Find the message
		const messageIndex = this.messages.findIndex((m) => m.id === messageId);
		if (messageIndex === -1) {
			console.error('Message not found:', messageId);
			return;
		}

		// Remove from UI immediately (no undo for permanent delete)
		this.messages = this.messages.filter((m) => m.id !== messageId);

		// If this was the selected message, clear selection
		if (this.selectedMessage?.id === messageId) {
			this.selectedMessage = null;
		}

		try {
			await emailAPI.deleteMessage(messageId);
			toastStore.success('Email permanently deleted');
			console.log('[EmailStore] Message permanently deleted:', messageId);
		} catch (err) {
			console.error('[EmailStore] Failed to permanently delete message:', err);
			toastStore.error('Failed to delete email');
			// Note: We don't restore the message for permanent delete errors
			// since the server might have deleted it
		}
	}

	// =====================
	// Reply functionality
	// =====================

	/**
	 * Start replying to an email
	 * Creates reply context with pre-populated fields
	 */
	startReply(message: Email) {
		// Extract sender email for reply
		const replyTo = message.from.map((contact) => contact.email);

		// Add "Re:" prefix to subject if not already present
		const subject = message.subject.startsWith('Re:')
			? message.subject
			: `Re: ${message.subject}`;

		// Create quoted body (simple text quote)
		const quotedBody = message.body_text
			? `\n\n--- Original Message ---\nFrom: ${message.from.map((f) => f.name || f.email).join(', ')}\nDate: ${new Date(message.received_at).toLocaleString()}\nSubject: ${message.subject}\n\n${message.body_text}`
			: '';

		this.replyContext = {
			originalMessageId: message.id,
			to: replyTo,
			subject,
			inReplyTo: message.id,
			quotedBody
		};

		console.log('[EmailStore] Started reply to:', message.id);
	}

	/**
	 * Clear reply context
	 * Called when composer is closed or email is sent
	 */
	clearReplyContext() {
		this.replyContext = null;
		console.log('[EmailStore] Cleared reply context');
	}
}

// Export singleton instance
export const emailStore = new EmailStore();
