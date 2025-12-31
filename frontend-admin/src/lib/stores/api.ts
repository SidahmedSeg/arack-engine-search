import { SearchEngineAPI } from '$shared/api-client';
import { writable } from 'svelte/store';

// Get API URL from environment or use default
const API_URL = import.meta.env.VITE_API_URL || 'http://127.0.0.1:3000';

// Create and export the API client instance
export const api = new SearchEngineAPI(API_URL);

// Store for tracking API loading states
export const isLoading = writable(false);

// Store for global errors
export const globalError = writable<string | null>(null);
