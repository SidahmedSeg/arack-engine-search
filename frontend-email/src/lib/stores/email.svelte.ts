import { emailAPI, type Email, type Mailbox } from '$lib/api/client';

// Email store using Svelte 5 runes
class EmailStore {
	currentMailbox = $state<string>('inbox');
	messages = $state<Email[]>([]);
	selectedMessage = $state<Email | null>(null);
	mailboxes = $state<Mailbox[]>([]);
	loading = $state(false);
	error = $state<string | null>(null);

	// Mock account ID for Phase 4 (will come from auth in production)
	accountId = 'test-account-123';

	async loadMailboxes() {
		this.loading = true;
		this.error = null;
		try {
			this.mailboxes = await emailAPI.getMailboxes(this.accountId);
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
			const result = await emailAPI.getMessages(this.accountId, mailboxId, limit);
			this.messages = result.messages;
		} catch (err) {
			this.error = err instanceof Error ? err.message : 'Failed to load messages';
			console.error('Error loading messages:', err);
		} finally {
			this.loading = false;
		}
	}

	async loadMessage(messageId: string) {
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
}

// Export singleton instance
export const emailStore = new EmailStore();
