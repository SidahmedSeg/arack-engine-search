// Search Engine API Types

export interface FormattedResult {
	title?: string;
	content?: string;
	description?: string;
}

export interface SearchResult {
	id: string;
	url: string;
	title: string;
	content: string;
	description?: string;
	keywords?: string[];
	word_count: number;
	crawled_at: string;
	_formatted?: FormattedResult; // Phase 7.3: Highlighted fields
	image_count?: number; // Phase 9: Number of images indexed from this page
	favicon_url?: string; // Phase 9: Favicon URL extracted during crawl
}

export interface SearchResponse {
	hits: SearchResult[];
	query: string;
	processing_time_ms: number;
	total_hits: number;
	suggestions?: string[]; // Phase 7.2: Search suggestions for typos/zero results
}

export interface SearchParams {
	q: string;
	limit?: number;
	offset?: number;
	min_word_count?: number;
	max_word_count?: number;
	from_date?: string;
	to_date?: string;
	sort_by?: 'crawled_at' | 'word_count';
	sort_order?: 'asc' | 'desc';
}

export interface CrawlRequest {
	urls: string[];
	max_depth: number;
}

export interface CrawlResponse {
	message: string;
	documents_indexed: number;
	urls: string[];
}

export interface IndexStats {
	numberOfDocuments: number;
	isIndexing: boolean;
	fieldDistribution: Record<string, number>;
}

export interface HealthResponse {
	status: string;
	timestamp: string;
}

export interface ApiResponse<T> {
	success: boolean;
	data?: T;
	error?: string;
}

// Phase 7.1: Autocomplete
export interface AutocompleteSuggestion {
	query: string;
	count: number;
}

export interface AutocompleteResponse {
	suggestions: AutocompleteSuggestion[];
	processing_time_ms: number;
}

// Phase 8: Authentication Types
export type UserRole = 'admin' | 'user';

export interface User {
	id: string;
	email: string;
	role: UserRole;
	first_name: string | null;
	last_name: string | null;
	created_at: string;
	last_login: string | null;
	is_active: boolean;
}

export interface LoginRequest {
	email: string;
	password: string;
}

export interface RegisterRequest {
	email: string;
	password: string;
	first_name: string;
	last_name: string;
}

// Custom Registration (Zitadel + Email Provisioning)
export interface CustomRegisterRequest {
	first_name: string;
	last_name: string;
	date_of_birth: string;
	gender: string;
	username: string;
	password: string;
}

export interface CustomRegisterResponse {
	success: boolean;
	message: string;
	email?: string;
	user_id?: string;
}

export interface AuthResponse {
	user: User;
}

export interface AcceptInvitationRequest {
	password: string;
	first_name: string;
	last_name: string;
}

export interface InvitationVerifyResponse {
	email: string;
	role: UserRole;
	expires_at: string;
}

// Phase 9: Image Search Types
export interface ImageData {
	id: string;
	image_url: string;
	source_url: string;
	alt_text?: string;
	title?: string;
	width?: number;
	height?: number;
	page_title: string;
	page_content: string;
	domain: string;
	crawled_at: string;
	// Priority 1: Rich image signals
	is_og_image: boolean;          // True if from Open Graph metadata (high quality)
	figcaption?: string;           // Semantic caption from <figcaption> element
	srcset_url?: string;           // Highest resolution URL from srcset attribute
}

export interface ImageSearchParams {
	q: string;
	limit?: number;
	offset?: number;
	min_width?: number;
	min_height?: number;
	domain?: string;
}

export interface ImageSearchResponse {
	hits: ImageData[];
	query: string;
	processing_time_ms: number;
	total_hits: number;
}

// Phase 10: Hybrid Search Types (Semantic + Keyword)
export interface HybridResult {
	id: string;
	url: string;
	title: string;
	description?: string;
	content?: string;
	keyword_score?: number;
	semantic_score?: number;
	combined_score: number;
}

export interface HybridSearchResponse {
	hits: HybridResult[];
	query: string;
	processing_time_ms: number;
	keyword_count: number;
	semantic_count: number;
}
