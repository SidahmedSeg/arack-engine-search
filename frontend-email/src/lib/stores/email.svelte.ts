import { emailAPI, type Email, type Mailbox } from '$lib/api/client';

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
			this.mailboxes = await emailAPI.getMailboxes();
		} catch (err) {
			this.error = err instanceof Error ? err.message : 'Failed to load mailboxes';
			console.error('Error loading mailboxes:', err);
		} finally {
			this.loading = false;
		}
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
	async sendEmail(to: string[], subject: string, bodyText: string) {
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
					body_text: bodyText
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
		if (this.currentMailbox === 'inbox') {
			// Create a placeholder email (will be replaced when fully loaded)
			const newEmail: Email = {
				id: emailId,
				subject,
				from: { email: from },
				preview,
				received_at: new Date().toISOString(),
				is_read: false,
				is_starred: false,
				has_attachments: false
			};

			// Add to top of list
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

		switch (updateType) {
			case 'read':
				this.messages[messageIndex] = { ...message, is_read: true };
				if (!message.is_read) this.unreadCount--;
				break;
			case 'unread':
				this.messages[messageIndex] = { ...message, is_read: false };
				if (message.is_read) this.unreadCount++;
				break;
			case 'starred':
				this.messages[messageIndex] = { ...message, is_starred: true };
				break;
			case 'unstarred':
				this.messages[messageIndex] = { ...message, is_starred: false };
				break;
			case 'deleted':
			case 'moved':
				// Remove from current list
				this.messages = this.messages.filter((m) => m.id !== emailId);
				if (!message.is_read) this.unreadCount--;
				break;
		}

		// Update selected message if it's the one being updated
		if (this.selectedMessage?.id === emailId) {
			if (updateType === 'deleted' || updateType === 'moved') {
				this.selectedMessage = null;
			} else {
				this.selectedMessage = this.messages[messageIndex];
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
}

// Export singleton instance
export const emailStore = new EmailStore();
