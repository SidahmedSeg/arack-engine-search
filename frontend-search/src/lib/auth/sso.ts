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
 * Redirect to SSO login page (OAuth flow - fallback)
 * After login, user will be redirected back to return_url
 */
export function login(returnUrl?: string): void {
	const url = returnUrl || window.location.href;
	const loginUrl = `${ACCOUNT_URL}/login?return_url=${encodeURIComponent(url)}`;
	window.location.href = loginUrl;
}

// Login response from account.arack.io/api/login
export interface LoginResponse {
	success: boolean;
	user: SSOUser;
	// OAuth flow fields (when authRequest is provided)
	callbackUrl?: string;
	accessToken?: string;
}

// Registration request
export interface RegisterRequest {
	firstName: string;
	lastName: string;
	gender?: string;
	birthDate?: string;
	email: string;
	password: string;
	confirmPassword: string;
}

// Registration response
export interface RegisterResponse {
	success: boolean;
	user: SSOUser;
	email: string;
}

// Email availability check response
export interface EmailCheckResponse {
	available: boolean;
	email: string;
}

// Email suggestions response
export interface EmailSuggestionsResponse {
	suggestions: string[];
}

/**
 * Login with email and password
 * Returns user data on success, throws on failure
 * @param authRequest - Optional OAuth auth request ID for token exchange flow
 */
export async function loginWithCredentials(email: string, password: string, authRequest?: string): Promise<LoginResponse> {
	const response = await fetch(`${ACCOUNT_URL}/api/login`, {
		method: 'POST',
		credentials: 'include',
		headers: {
			'Content-Type': 'application/json',
			'Accept': 'application/json'
		},
		body: JSON.stringify({ email, password, authRequest })
	});

	if (!response.ok) {
		const error = await response.json().catch(() => ({ error: 'Login failed' }));
		throw new Error(error.error || 'Invalid email or password');
	}

	return await response.json();
}

/**
 * Register a new user
 * Creates local account and Stalwart email account
 */
export async function register(data: RegisterRequest): Promise<RegisterResponse> {
	const response = await fetch(`${ACCOUNT_URL}/api/register`, {
		method: 'POST',
		credentials: 'include',
		headers: {
			'Content-Type': 'application/json',
			'Accept': 'application/json'
		},
		body: JSON.stringify(data)
	});

	if (!response.ok) {
		const error = await response.json().catch(() => ({ error: 'Registration failed' }));
		throw new Error(error.error || 'Failed to create account');
	}

	return await response.json();
}

/**
 * Check if an email is available for registration
 */
export async function checkEmailAvailability(email: string): Promise<EmailCheckResponse> {
	const response = await fetch(`${ACCOUNT_URL}/api/register/check-email`, {
		method: 'POST',
		headers: {
			'Content-Type': 'application/json',
			'Accept': 'application/json'
		},
		body: JSON.stringify({ email })
	});

	if (!response.ok) {
		throw new Error('Failed to check email availability');
	}

	return await response.json();
}

/**
 * Get email suggestions based on name
 */
export async function getEmailSuggestions(firstName: string, lastName: string): Promise<string[]> {
	const params = new URLSearchParams({ firstName, lastName });
	const response = await fetch(`${ACCOUNT_URL}/api/register/suggestions?${params}`, {
		method: 'GET',
		headers: {
			'Accept': 'application/json'
		}
	});

	if (!response.ok) {
		throw new Error('Failed to get email suggestions');
	}

	const data: EmailSuggestionsResponse = await response.json();
	return data.suggestions;
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
