import axios, { type AxiosInstance } from 'axios';

const API_URL = import.meta.env.VITE_EMAIL_API_URL || 'http://localhost:3001';

export interface EmailAccount {
	id: string;
	kratos_identity_id: string;
	email_address: string;
	stalwart_user_id: string;
	storage_quota_bytes: number;
	storage_used_bytes: number;
	is_active: boolean;
}

export interface Mailbox {
	id: string;
	name: string;
	role?: string;
	total_emails: number;
	unread_emails: number;
}

export interface EmailContact {
	email: string;
	name?: string;
}

export interface Email {
	id: string;
	subject: string;
	from: EmailContact;
	to?: EmailContact[];
	cc?: EmailContact[];
	preview: string;
	received_at: string;
	is_read: boolean;
	is_starred: boolean;
	has_attachments: boolean;
	body_html?: string;
	body_text?: string;
}

export interface SendEmailRequest {
	to: string[];
	cc?: string[];
	subject: string;
	body_text: string;
	body_html?: string;
	attachments?: AttachmentInfo[];
}

export interface AttachmentInfo {
	filename: string;
	content_type: string;
	size: number;
	blob_id?: string;
}

export interface EmailSearchParams {
	query: string;
	account_id: string;
	mailbox_id?: string;
	from?: string;
	is_read?: boolean;
	limit?: number;
	offset?: number;
}

export interface SearchResults {
	query: string;
	results: Email[];
	total: number;
}

export interface WebSocketToken {
	token: string;
	channel: string;
}

class EmailAPIClient {
	private client: AxiosInstance;

	constructor() {
		this.client = axios.create({
			baseURL: API_URL,
			headers: {
				'Content-Type': 'application/json'
			}
		});
	}

	// Account
	async getAccount(kratosId: string): Promise<{ account: EmailAccount; quota_percentage: number }> {
		const { data } = await this.client.get(`/api/mail/account`, {
			params: { kratos_id: kratosId }
		});
		return data;
	}

	// Mailboxes
	async getMailboxes(accountId: string): Promise<Mailbox[]> {
		const { data } = await this.client.get(`/api/mail/mailboxes`, {
			params: { account_id: accountId }
		});
		return data.mailboxes;
	}

	async createMailbox(accountId: string, name: string, parentId?: string): Promise<{ mailbox_id: string }> {
		const { data } = await this.client.post(`/api/mail/mailboxes`, {
			account_id: accountId,
			name,
			parent_id: parentId
		});
		return data;
	}

	// Messages
	async getMessages(
		accountId: string,
		mailboxId: string,
		limit: number = 50,
		offset: number = 0
	): Promise<{ messages: Email[]; total: number; limit: number }> {
		const { data } = await this.client.get(`/api/mail/messages`, {
			params: { account_id: accountId, mailbox_id: mailboxId, limit, offset }
		});
		return data;
	}

	async getMessage(messageId: string, accountId: string): Promise<Email> {
		const { data } = await this.client.get(`/api/mail/messages/${messageId}`, {
			params: { account_id: accountId }
		});
		return data.message;
	}

	async sendEmail(request: SendEmailRequest, accountId: string): Promise<{ message_id: string }> {
		const { data } = await this.client.post(`/api/mail/messages`, {
			...request,
			account_id: accountId
		});
		return data;
	}

	// Search
	async searchEmails(params: EmailSearchParams): Promise<SearchResults> {
		const { data } = await this.client.get(`/api/mail/search`, { params });
		return data;
	}

	// WebSocket token
	async getWebSocketToken(userId: string): Promise<WebSocketToken> {
		const { data } = await this.client.get(`/api/mail/ws/token`, {
			params: { user_id: userId }
		});
		return data;
	}

	// Health check
	async health(): Promise<{ status: string; phase: string; service: string; version: string }> {
		const { data } = await this.client.get(`/health`);
		return data;
	}
}

// Export singleton instance
export const emailAPI = new EmailAPIClient();
