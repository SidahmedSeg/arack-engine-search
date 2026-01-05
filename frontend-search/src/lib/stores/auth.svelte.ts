// Phase 9 - Central SSO: Authentication Store
// Manages user authentication state via account.arack.io shared session

import {
	getSession,
	getUserInfo,
	logout as ssoLogout,
	login as ssoLogin,
	getAccessToken as ssoGetAccessToken,
	loginWithCredentials,
	register as ssoRegister,
	type SSOUser,
	type RegisterRequest
} from '$lib/auth/sso';

// User interface (compatible with previous implementation)
export interface User {
	id: string;
	email: string;
	firstName: string;
	lastName: string;
	picture?: string;
}

// Auth state interface
interface AuthState {
	user: User | null;
	isAuthenticated: boolean;
	isLoading: boolean;
	error: string | null;
	accessToken: string | null;
}

class AuthStore {
	private state = $state<AuthState>({
		user: null,
		isAuthenticated: false,
		isLoading: true,
		error: null,
		accessToken: null
	});

	// Getters for reactive state
	get user() {
		return this.state.user;
	}

	get isAuthenticated() {
		return this.state.isAuthenticated;
	}

	get isLoading() {
		return this.state.isLoading;
	}

	get error() {
		return this.state.error;
	}

	get accessToken() {
		return this.state.accessToken;
	}

	/**
	 * Check if user has an active SSO session
	 * This should be called on app initialization
	 */
	async checkSession() {
		this.state.isLoading = true;
		this.state.error = null;

		try {
			// Check SSO session via account.arack.io
			const session = await getSession();

			if (!session) {
				console.log('[AuthStore] No SSO session found');
				this.clearSession();
				return;
			}

			// Parse name into first/last (best effort)
			const nameParts = (session.name || '').split(' ');
			const firstName = nameParts[0] || '';
			const lastName = nameParts.slice(1).join(' ') || '';

			this.state.user = {
				id: session.user_id,
				email: session.email,
				firstName,
				lastName,
				picture: session.picture
			};
			this.state.accessToken = session.access_token;
			this.state.isAuthenticated = true;

			console.log('[AuthStore] SSO session restored:', session.email);
		} catch (error: any) {
			console.error('[AuthStore] Session check failed:', error);
			this.clearSession();
			this.state.error = 'Failed to check session';
		} finally {
			this.state.isLoading = false;
		}
	}

	/**
	 * Set user after successful login
	 * Called when returning from SSO login
	 */
	setUser(user: User, accessToken: string) {
		this.state.user = user;
		this.state.accessToken = accessToken;
		this.state.isAuthenticated = true;
		this.state.error = null;
		console.log('[AuthStore] User authenticated:', user.email);
	}

	/**
	 * Login via SSO (OAuth redirect - fallback)
	 * Redirects to account.arack.io login page
	 */
	login(returnUrl?: string) {
		ssoLogin(returnUrl);
	}

	/**
	 * Login with email and password
	 * Direct authentication via account-service
	 * @param authRequest - Optional OAuth auth request ID for token exchange flow
	 * @returns Object with callbackUrl if OAuth flow, undefined otherwise
	 */
	async loginWithPassword(email: string, password: string, authRequest?: string): Promise<{ callbackUrl?: string } | undefined> {
		this.state.isLoading = true;
		this.state.error = null;

		try {
			const response = await loginWithCredentials(email, password, authRequest);

			// Parse name into first/last (best effort)
			const nameParts = (response.user.name || '').split(' ');
			const firstName = nameParts[0] || '';
			const lastName = nameParts.slice(1).join(' ') || '';

			this.state.user = {
				id: response.user.id,
				email: response.user.email,
				firstName,
				lastName,
				picture: response.user.picture
			};
			this.state.isAuthenticated = true;

			// Store access token if provided (from OAuth flow)
			if (response.accessToken) {
				this.state.accessToken = response.accessToken;
			}

			console.log('[AuthStore] User logged in with password:', response.user.email);

			// Return callback URL if OAuth flow
			return response.callbackUrl ? { callbackUrl: response.callbackUrl } : undefined;
		} catch (error: any) {
			console.error('[AuthStore] Login failed:', error);
			this.state.error = error.message || 'Login failed';
			throw error;
		} finally {
			this.state.isLoading = false;
		}
	}

	/**
	 * Register a new user
	 * Creates local + Stalwart accounts
	 */
	async register(data: RegisterRequest): Promise<void> {
		this.state.isLoading = true;
		this.state.error = null;

		try {
			const response = await ssoRegister(data);

			// Parse name into first/last
			const nameParts = (response.user.name || '').split(' ');
			const firstName = nameParts[0] || data.firstName;
			const lastName = nameParts.slice(1).join(' ') || data.lastName;

			this.state.user = {
				id: response.user.id,
				email: response.email,
				firstName,
				lastName,
				picture: response.user.picture
			};
			this.state.isAuthenticated = true;

			console.log('[AuthStore] User registered:', response.email);
		} catch (error: any) {
			console.error('[AuthStore] Registration failed:', error);
			this.state.error = error.message || 'Registration failed';
			throw error;
		} finally {
			this.state.isLoading = false;
		}
	}

	/**
	 * Logout from SSO
	 * Clears session across all arack.io apps
	 */
	async logout() {
		this.state.error = null;

		try {
			console.log('[AuthStore] Logging out via SSO...');
			await ssoLogout();
			// ssoLogout redirects to home, so this might not execute
			this.clearSession();
		} catch (error) {
			console.error('[AuthStore] Logout failed:', error);
			this.state.error = 'Failed to logout';
			this.clearSession();
			window.location.href = '/';
		}
	}

	/**
	 * Get current access token
	 * Returns cached token or fetches fresh one
	 */
	async getAccessToken(): Promise<string | null> {
		if (this.state.accessToken) {
			return this.state.accessToken;
		}
		return await ssoGetAccessToken();
	}

	/**
	 * Clear local session state
	 */
	private clearSession() {
		this.state.user = null;
		this.state.accessToken = null;
		this.state.isAuthenticated = false;
	}

	/**
	 * Refresh session data from SSO
	 */
	async refreshSession() {
		await this.checkSession();
	}
}

// Export singleton instance
export const authStore = new AuthStore();
