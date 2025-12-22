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

// AI Features (Phase 5)
export interface SmartComposeRequest {
	partial_text: string;
	context?: {
		subject?: string;
		recipient?: string;
		is_reply: boolean;
	};
}

export interface SmartComposeSuggestion {
	text: string;
	confidence: number;
}

export interface SmartComposeResponse {
	suggestions: SmartComposeSuggestion[];
}

export interface SummarizeRequest {
	thread_id?: string;
	email_ids: string[];
}

export interface SummarizeResponse {
	summary: string;
	key_points: string[];
	action_items: string[];
	token_count: number;
}

export interface PriorityRankRequest {
	mailbox_id: string;
	email_ids: string[];
}

export interface PriorityEmail {
	email_id: string;
	priority_score: number;
	reason: string;
}

export interface PriorityRankResponse {
	ranked_emails: PriorityEmail[];
}

export interface QuotaUsage {
	used: number;
	limit: number;
	reset_at: string;
}

export interface AiQuota {
	smart_compose: QuotaUsage;
	summarization: QuotaUsage;
	priority_ranking: QuotaUsage;
}

// OAuth (Phase 8 - OIDC)
export interface OAuthStatus {
	connected: boolean;
	expires_at?: string;
	scope?: string;
}

class EmailAPIClient {
	private client: AxiosInstance;

	constructor() {
		this.client = axios.create({
			baseURL: API_URL,
			headers: {
				'Content-Type': 'application/json'
			},
			withCredentials: true // Include cookies in all requests
		});
	}

	// Account
	async getAccount(kratosId: string): Promise<{ account: EmailAccount; quota_percentage: number }> {
		const { data } = await this.client.get(`/api/mail/account`, {
			params: { kratos_id: kratosId }
		});
		return data;
	}

	// Get current user's account from session cookie
	async getMyAccount(): Promise<{ account: EmailAccount; quota_percentage: number }> {
		const { data } = await this.client.get(`/api/mail/account/me`, {
			withCredentials: true // Include cookies in the request
		});
		return data;
	}

	// Mailboxes - gets user from session cookie
	async getMailboxes(): Promise<Mailbox[]> {
		const { data } = await this.client.get(`/api/mail/mailboxes`);
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

	// Messages - gets user from session cookie
	async getMessages(
		mailboxId: string,
		limit: number = 50,
		offset: number = 0
	): Promise<{ messages: Email[]; total: number; limit: number }> {
		const { data } = await this.client.get(`/api/mail/messages`, {
			params: { mailbox_id: mailboxId, limit, offset }
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

	// WebSocket token - gets user ID from session cookie
	async getWebSocketToken(): Promise<WebSocketToken> {
		const { data } = await this.client.get(`/api/mail/ws/token`);
		return data;
	}

	// Health check
	async health(): Promise<{ status: string; phase: string; service: string; version: string }> {
		const { data } = await this.client.get(`/health`);
		return data;
	}

	// AI Features (Phase 5)
	async smartCompose(accountId: string, request: SmartComposeRequest): Promise<SmartComposeResponse> {
		const { data } = await this.client.post(`/api/mail/ai/smart-compose`, request, {
			params: { account_id: accountId }
		});
		return data;
	}

	async summarizeThread(accountId: string, request: SummarizeRequest): Promise<SummarizeResponse> {
		const { data } = await this.client.post(`/api/mail/ai/summarize`, request, {
			params: { account_id: accountId }
		});
		return data;
	}

	async priorityRank(accountId: string, request: PriorityRankRequest): Promise<PriorityRankResponse> {
		const { data } = await this.client.post(`/api/mail/ai/priority-rank`, request, {
			params: { account_id: accountId }
		});
		return data;
	}

	async getAiQuota(accountId: string): Promise<AiQuota> {
		const { data } = await this.client.get(`/api/mail/ai/quota`, {
			params: { account_id: accountId }
		});
		return data;
	}

	// OAuth (Phase 8 - OIDC)
	/**
	 * Get OAuth connection status
	 */
	async getOAuthStatus(): Promise<OAuthStatus> {
		const { data } = await this.client.get(`/api/mail/oauth/status`);
		return data;
	}

	/**
	 * Disconnect OAuth connection
	 */
	async disconnectOAuth(): Promise<void> {
		await this.client.post(`/api/mail/oauth/disconnect`);
	}

	// Note: OAuth authorize and callback are handled via browser redirects
	// No API client methods needed for those endpoints
}

// Export singleton instance
export const emailAPI = new EmailAPIClient();
