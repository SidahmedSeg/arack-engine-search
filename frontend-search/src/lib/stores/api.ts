import { SearchEngineAPI } from '$shared/api-client';

// Get API URL from environment variable or use production URL
const API_URL = import.meta.env.VITE_API_URL || 'https://api.arack.io';

// Create and export a single API client instance
export const api = new SearchEngineAPI(API_URL);
