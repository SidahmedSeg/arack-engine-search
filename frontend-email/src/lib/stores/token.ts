// src/lib/stores/token.ts
// Simple token store for SSO access token
// Phase 9: Replaces OAuth localStorage tokens with SSO session token

let accessToken: string | null = null;

/**
 * Set the access token (called from layout after server-side auth)
 */
export function setAccessToken(token: string | null) {
	accessToken = token;
}

/**
 * Get the current access token
 */
export function getAccessToken(): string | null {
	return accessToken;
}

/**
 * Check if we have a valid token
 */
export function hasToken(): boolean {
	return accessToken !== null && accessToken !== '';
}

/**
 * Clear the token (on logout)
 */
export function clearToken() {
	accessToken = null;
}
