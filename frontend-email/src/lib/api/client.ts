import axios, { type AxiosInstance, type InternalAxiosRequestConfig } from 'axios';
import { getAccessToken } from '$lib/stores/token';

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
			}
		});

		// Add auth interceptor to include Bearer token in all requests
		// Phase 9: Uses SSO token from token store (set by layout from server session)
		this.client.interceptors.request.use(
			(config: InternalAxiosRequestConfig) => {
				// Skip auth for health check
				if (config.url === '/health') {
					return config;
				}

				// Get access token from SSO session (sync)
				const token = getAccessToken();
				if (token) {
					config.headers.Authorization = `Bearer ${token}`;
				}
				return config;
			},
			(error) => {
				return Promise.reject(error);
			}
		);

		// Add response interceptor to handle 401 errors
		this.client.interceptors.response.use(
			(response) => response,
			async (error) => {
				if (error.response?.status === 401) {
					// Token is invalid or expired, redirect to login
					console.error('[EmailAPI] Unauthorized - redirecting to login');
					const returnUrl = encodeURIComponent(window.location.href);
					window.location.href = `https://arack.io/auth/login?return_url=${returnUrl}`;
				}
				return Promise.reject(error);
			}
		);
	}

	// Account
	async getAccount(kratosId: string): Promise<{ account: EmailAccount; quota_percentage: number }> {
		const { data } = await this.client.get(`/api/mail/account`, {
			params: { kratos_id: kratosId }
		});
		return data;
	}

	// Get current user's account from OAuth token
	async getMyAccount(): Promise<{ account: EmailAccount; quota_percentage: number }> {
		const { data } = await this.client.get(`/api/mail/account/me`);
		return data;
	}

	// Mailboxes - gets user from OAuth token
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

	// Messages - gets user from OAuth token
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

	// WebSocket token - gets user ID from OAuth token
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

	// OAuth (Phase 8 - OIDC) - Using Zitadel OAuth tokens for SSO
	// Bearer tokens are automatically added via axios interceptor
	// If token is invalid/expired, API calls return 401 and frontend redirects to Zitadel login
}

// Export singleton instance
export const emailAPI = new EmailAPIClient();
