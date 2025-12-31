import axios, { type AxiosInstance } from 'axios';
import type {
	SearchParams,
	SearchResponse,
	CrawlRequest,
	CrawlResponse,
	IndexStats,
	HealthResponse,
	ApiResponse,
	AutocompleteResponse,
	User,
	LoginRequest,
	AuthResponse,
	AcceptInvitationRequest,
	InvitationVerifyResponse,
	ImageSearchParams,
	ImageSearchResponse,
	HybridSearchResponse,
	CustomRegisterRequest,
	CustomRegisterResponse,
} from '../types';

export class SearchEngineAPI {
	private client: AxiosInstance;

	constructor(baseUrl: string = 'https://api.arack.io') {
		this.client = axios.create({
			baseURL: baseUrl,
			headers: {
				'Content-Type': 'application/json',
			},
		});
	}

	/**
	 * Health check
	 */
	async healthCheck(): Promise<HealthResponse> {
		const response = await this.client.get<HealthResponse>('/health');
		return response.data;
	}

	/**
	 * Search indexed documents
	 */
	async search(params: SearchParams): Promise<SearchResponse> {
		const response = await this.client.get<ApiResponse<SearchResponse>>('/api/search', {
			params,
			withCredentials: true,
		});

		if (!response.data.success || !response.data.data) {
			throw new Error(response.data.error || 'Search failed');
		}

		return response.data.data;
	}

	/**
	 * Hybrid search (keyword + semantic) - Phase 10
	 */
	async hybridSearch(params: SearchParams): Promise<HybridSearchResponse> {
		const response = await this.client.get<ApiResponse<HybridSearchResponse>>('/api/search/hybrid', {
			params,
			withCredentials: true,
		});

		if (!response.data.success || !response.data.data) {
			throw new Error(response.data.error || 'Hybrid search failed');
		}

		return response.data.data;
	}

	/**
	 * Start a new crawl job
	 */
	async startCrawl(request: CrawlRequest): Promise<CrawlResponse> {
		const response = await this.client.post<ApiResponse<CrawlResponse>>('/api/crawl', request, {
			withCredentials: true,
		});

		if (!response.data.success || !response.data.data) {
			throw new Error(response.data.error || 'Crawl failed');
		}

		return response.data.data;
	}

	/**
	 * Get crawl history with pagination
	 */
	async getCrawlHistory(limit: number = 20, offset: number = 0): Promise<any> {
		const response = await this.client.get<ApiResponse<any>>('/api/crawl/history', {
			params: { limit, offset },
			withCredentials: true,
		});

		if (!response.data.success || !response.data.data) {
			throw new Error(response.data.error || 'Failed to get crawl history');
		}

		return response.data.data;
	}

	/**
	 * Get index statistics
	 */
	async getStats(): Promise<IndexStats> {
		const response = await this.client.get<ApiResponse<IndexStats>>('/api/stats', {
			withCredentials: true,
		});

		if (!response.data.success || !response.data.data) {
			throw new Error(response.data.error || 'Failed to get stats');
		}

		return response.data.data;
	}

	/**
	 * Get image index statistics
	 */
	async getImageStats(): Promise<any> {
		const response = await this.client.get<ApiResponse<any>>('/api/stats/images', {
			withCredentials: true,
		});

		if (!response.data.success || !response.data.data) {
			throw new Error(response.data.error || 'Failed to get image stats');
		}

		return response.data.data;
	}

	/**
	 * Clear the entire index
	 */
	async clearIndex(): Promise<void> {
		const response = await this.client.delete<ApiResponse<{ message: string }>>('/api/index', {
			withCredentials: true,
		});

		if (!response.data.success) {
			throw new Error(response.data.error || 'Failed to clear index');
		}
	}

	/**
	 * Get crawler metrics (Phase 6.10)
	 */
	async getCrawlerMetrics(): Promise<any> {
		const response = await this.client.get<ApiResponse<any>>('/api/crawler/metrics', {
			withCredentials: true,
		});

		if (!response.data.success || !response.data.data) {
			throw new Error(response.data.error || 'Failed to get crawler metrics');
		}

		return response.data.data;
	}

	/**
	 * Get per-domain crawler stats (Phase 6.10)
	 */
	async getCrawlerDomains(): Promise<any> {
		const response = await this.client.get<ApiResponse<any>>('/api/crawler/domains', {
			withCredentials: true,
		});

		if (!response.data.success || !response.data.data) {
			throw new Error(response.data.error || 'Failed to get crawler domains');
		}

		return response.data.data;
	}

	/**
	 * Get crawler scheduler information (Phase 6.10)
	 */
	async getCrawlerScheduler(): Promise<any> {
		const response = await this.client.get<ApiResponse<any>>('/api/crawler/scheduler', {
			withCredentials: true,
		});

		if (!response.data.success || !response.data.data) {
			throw new Error(response.data.error || 'Failed to get crawler scheduler');
		}

		return response.data.data;
	}

	/**
	 * Get autocomplete suggestions (Phase 7.1)
	 */
	async autocomplete(query: string, limit: number = 5): Promise<AutocompleteResponse> {
		const response = await this.client.get<ApiResponse<AutocompleteResponse>>(
			'/api/search/autocomplete',
			{
				params: { q: query, limit },
				withCredentials: true,
			}
		);

		if (!response.data.success || !response.data.data) {
			throw new Error(response.data.error || 'Autocomplete failed');
		}

		return response.data.data;
	}

	/**
	 * Search indexed images (Phase 9)
	 */
	async searchImages(params: ImageSearchParams): Promise<ImageSearchResponse> {
		const response = await this.client.get<ApiResponse<ImageSearchResponse>>(
			'/api/search/images',
			{
				params,
				withCredentials: true,
			}
		);

		if (!response.data.success || !response.data.data) {
			throw new Error(response.data.error || 'Image search failed');
		}

		return response.data.data;
	}

	/**
	 * Hybrid image search (keyword + semantic) - Phase 10.5
	 */
	async hybridImageSearch(params: ImageSearchParams): Promise<ImageSearchResponse> {
		const response = await this.client.get<ApiResponse<ImageSearchResponse>>(
			'/api/search/images/hybrid',
			{
				params,
				withCredentials: true,
			}
		);

		if (!response.data.success || !response.data.data) {
			throw new Error(response.data.error || 'Hybrid image search failed');
		}

		return response.data.data;
	}

	// Phase 8: Authentication Methods

	/**
	 * Login with email and password
	 */
	async login(credentials: LoginRequest): Promise<User> {
		const response = await this.client.post<ApiResponse<AuthResponse>>(
			'/api/auth/login',
			credentials,
			{
				withCredentials: true, // Important for session cookies
			}
		);

		if (!response.data.success || !response.data.data) {
			throw new Error(response.data.error || 'Login failed');
		}

		return response.data.data.user;
	}

	/**
	 * Register a new user (Custom 3-step registration via Zitadel)
	 * Creates user in Zitadel and provisions email account in Stalwart
	 */
	async register(data: CustomRegisterRequest): Promise<CustomRegisterResponse> {
		const response = await this.client.post<CustomRegisterResponse>(
			'/api/auth/register',
			data
		);

		if (!response.data.success) {
			throw new Error(response.data.message || 'Registration failed');
		}

		return response.data;
	}

	/**
	 * Logout current user
	 */
	async logout(): Promise<void> {
		const response = await this.client.post<ApiResponse<{ message: string }>>(
			'/api/auth/logout',
			{},
			{
				withCredentials: true,
			}
		);

		if (!response.data.success) {
			throw new Error(response.data.error || 'Logout failed');
		}
	}

	/**
	 * Get current authenticated user
	 */
	async getCurrentUser(): Promise<User | null> {
		try {
			const response = await this.client.get<ApiResponse<User>>('/api/auth/me', {
				withCredentials: true,
			});

			if (!response.data.success || !response.data.data) {
				return null;
			}

			return response.data.data;
		} catch (error) {
			// 401 means not authenticated
			return null;
		}
	}

	/**
	 * Verify invitation token
	 */
	async verifyInvitation(token: string): Promise<InvitationVerifyResponse> {
		const response = await this.client.get<ApiResponse<InvitationVerifyResponse>>(
			`/api/auth/invitations/${token}`
		);

		if (!response.data.success || !response.data.data) {
			throw new Error(response.data.error || 'Invalid invitation token');
		}

		return response.data.data;
	}

	/**
	 * Accept invitation and create user account
	 */
	async acceptInvitation(
		token: string,
		request: AcceptInvitationRequest
	): Promise<User> {
		const response = await this.client.post<ApiResponse<AuthResponse>>(
			`/api/auth/invitations/${token}/accept`,
			request,
			{
				withCredentials: true,
			}
		);

		if (!response.data.success || !response.data.data) {
			throw new Error(response.data.error || 'Failed to accept invitation');
		}

		return response.data.data.user;
	}

	/**
	 * Check username availability (Phase 8: Simplified Registration)
	 */
	async checkUsername(data: { username: string }): Promise<{
		available: boolean;
		email: string;
		reason?: string;
	}> {
		const response = await this.client.get<ApiResponse<{
			available: boolean;
			email: string;
			reason?: string;
		}>>('/api/auth/check-username', {
			params: data,
		});

		if (!response.data.success || !response.data.data) {
			throw new Error(response.data.error || 'Failed to check username availability');
		}

		return response.data.data;
	}

	/**
	 * Get username suggestions (Phase 8: Simplified Registration)
	 */
	async suggestUsernames(data: {
		first_name: string;
		last_name: string;
	}): Promise<{
		suggestions: Array<{
			username: string;
			email: string;
			available: boolean;
		}>;
	}> {
		const response = await this.client.post<ApiResponse<{
			suggestions: Array<{
				username: string;
				email: string;
				available: boolean;
			}>;
		}>>('/api/auth/suggest-usernames', data);

		if (!response.data.success || !response.data.data) {
			throw new Error(response.data.error || 'Failed to get username suggestions');
		}

		return response.data.data;
	}

	/**
	 * Get analytics summary
	 */
	async getAnalyticsSummary(days: number = 7): Promise<any> {
		const response = await this.client.get<ApiResponse<any>>(
			'/api/analytics/summary',
			{
				params: { days },
				withCredentials: true,
			}
		);

		if (!response.data.success || !response.data.data) {
			throw new Error(response.data.error || 'Failed to get analytics');
		}

		return response.data.data;
	}

	/**
	 * Get job status by job ID
	 */
	async getJobStatus(jobId: string): Promise<any> {
		const response = await this.client.get<ApiResponse<any>>(
			`/api/jobs/${jobId}`,
			{
				withCredentials: true,
			}
		);

		if (!response.data.success || !response.data.data) {
			throw new Error(response.data.error || 'Failed to get job status');
		}

		return response.data.data;
	}
}

// Export a default instance
export const api = new SearchEngineAPI();
