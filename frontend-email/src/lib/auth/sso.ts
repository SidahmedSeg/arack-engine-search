// Central SSO Client for account.arack.io
// Handles authentication via shared session cookie on .arack.io domain

const ACCOUNT_URL = import.meta.env.VITE_ACCOUNT_URL || 'https://account.arack.io';

// Session response from account.arack.io/api/session
export interface SSOSession {
	user_id: string;
	email: string;
	name: string;
	picture?: string;
	access_token: string;
}

// User info derived from session
export interface SSOUser {
	id: string;
	email: string;
	name: string;
	picture?: string;
}

/**
 * Check if user has an active SSO session
 * Returns session data if authenticated, null otherwise
 */
export async function getSession(): Promise<SSOSession | null> {
	try {
		const response = await fetch(`${ACCOUNT_URL}/api/session`, {
			method: 'GET',
			credentials: 'include', // Send cookies cross-origin
			headers: {
				'Accept': 'application/json'
			}
		});

		if (response.status === 401) {
			return null;
		}

		if (!response.ok) {
			console.error('[SSO] Session check failed:', response.status);
			return null;
		}

		return await response.json();
	} catch (error) {
		console.error('[SSO] Session check error:', error);
		return null;
	}
}

/**
 * Get the access token from current session
 * Returns null if not authenticated
 */
export async function getAccessToken(): Promise<string | null> {
	const session = await getSession();
	return session?.access_token ?? null;
}

/**
 * Check if user is authenticated (has valid session)
 */
export async function isAuthenticated(): Promise<boolean> {
	const session = await getSession();
	return session !== null;
}

/**
 * Redirect to SSO login page
 * After login, user will be redirected back to return_url
 */
export function login(returnUrl?: string): void {
	const url = returnUrl || window.location.href;
	const loginUrl = `${ACCOUNT_URL}/login?return_url=${encodeURIComponent(url)}`;
	window.location.href = loginUrl;
}

/**
 * Logout from SSO (clears session across all apps)
 */
export async function logout(): Promise<void> {
	try {
		await fetch(`${ACCOUNT_URL}/api/logout`, {
			method: 'POST',
			credentials: 'include'
		});
	} catch (error) {
		console.error('[SSO] Logout error:', error);
	}

	// Redirect to home after logout
	window.location.href = '/';
}

/**
 * Get user info from session
 */
export async function getUserInfo(): Promise<SSOUser | null> {
	const session = await getSession();
	if (!session) return null;

	return {
		id: session.user_id,
		email: session.email,
		name: session.name,
		picture: session.picture
	};
}
