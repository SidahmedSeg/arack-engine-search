import axios from 'axios';
import type { SearchResponse, SearchFilters, AutocompleteSuggestion } from './types';

const API_BASE_URL = import.meta.env.VITE_API_URL ? `${import.meta.env.VITE_API_URL}/api` : 'https://api.arack.io/api';

const api = axios.create({
	baseURL: API_BASE_URL,
	headers: {
		'Content-Type': 'application/json'
	},
	// Phase 8.6: Enable credentials for Ory cookie-based auth
	withCredentials: true
});

export interface AutocompleteResponse {
	suggestions: AutocompleteSuggestion[];
	processing_time_ms: number;
}

export async function search(filters: SearchFilters): Promise<SearchResponse> {
	const params = new URLSearchParams();

	if (filters.query) params.append('q', filters.query);
	if (filters.limit) params.append('limit', filters.limit.toString());
	if (filters.offset) params.append('offset', filters.offset.toString());
	if (filters.min_word_count) params.append('min_word_count', filters.min_word_count.toString());
	if (filters.max_word_count) params.append('max_word_count', filters.max_word_count.toString());
	if (filters.sort_by) params.append('sort_by', filters.sort_by);
	if (filters.order) params.append('order', filters.order);

	const response = await api.get(`/search?${params.toString()}`);

	// Backend returns {success: true, data: {hits, total_hits, suggestions, ...}}
	// Transform to frontend format
	const backendData = response.data.data;
	return {
		results: backendData.hits,
		total: backendData.total_hits,
		limit: filters.limit || 20,
		offset: filters.offset || 0,
		query: backendData.query,
		suggestions: backendData.suggestions // Phase 7.2: Search suggestions
	};
}

// Phase 7.1: Autocomplete function
export async function autocomplete(query: string, limit: number = 5): Promise<AutocompleteResponse> {
	const params = new URLSearchParams();
	params.append('q', query);
	params.append('limit', limit.toString());

	const response = await api.get(`/search/autocomplete?${params.toString()}`);

	// Backend returns {success: true, data: {suggestions, processing_time_ms}}
	return response.data.data;
}

export default api;
