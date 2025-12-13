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
	crawled_at: string;
	word_count: number;
	_formatted?: FormattedResult; // Phase 7.3: Highlighted fields
}

export interface SearchResponse {
	results: SearchResult[];
	total: number;
	limit: number;
	offset: number;
	query: string;
	suggestions?: string[]; // Phase 7.2: Search suggestions for typos/zero results
}

export interface SearchFilters {
	query: string;
	limit?: number;
	offset?: number;
	min_word_count?: number;
	max_word_count?: number;
	sort_by?: 'relevance' | 'date' | 'word_count';
	order?: 'asc' | 'desc';
}

// Phase 7.1: Autocomplete types
export interface AutocompleteSuggestion {
	query: string;
	count: number;
}
