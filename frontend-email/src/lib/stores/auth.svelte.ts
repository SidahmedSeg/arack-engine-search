// OAuth Authentication Store for Arack Mail
// Uses OAuth 2.0 / OIDC via Zitadel for SSO with Arack Search

import {
	getUserInfo,
	logout as oauthLogout,
	isAuthenticated as checkOAuthTokens,
	getAccessToken
} from '$lib/auth/oauth';

// User interface
export interface User {
	id: string;
	email: string;
	firstName: string;
	lastName: string;
}

// Auth state interface
interface AuthState {
	user: User | null;
	isAuthenticated: boolean;
	isLoading: boolean;
	error: string | null;
}

class AuthStore {
	private state = $state<AuthState>({
		user: null,
		isAuthenticated: false,
		isLoading: true,
		error: null
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

	/**
	 * Check if user has an active OAuth session
	 * This should be called on app initialization
	 */
	async checkSession() {
		this.state.isLoading = true;
		this.state.error = null;

		try {
			// Check if OAuth tokens exist and are valid
			if (!checkOAuthTokens()) {
				console.log('[AuthStore] No valid OAuth tokens found');
				this.clearSession();
				return;
			}

			// Get access token (with auto-refresh if needed)
			const accessToken = await getAccessToken();
			if (!accessToken) {
				console.log('[AuthStore] Failed to get access token');
				this.clearSession();
				return;
			}

			// Fetch user info from OAuth userinfo endpoint
			const userInfo = await getUserInfo();

			this.state.user = {
				id: userInfo.sub,
				email: userInfo.email || '',
				firstName: userInfo.given_name || '',
				lastName: userInfo.family_name || ''
			};
			this.state.isAuthenticated = true;

			console.log('[AuthStore] Session restored successfully');
		} catch (error: any) {
			// No active session or network error
			console.log('[AuthStore] Session check failed:', error);
			this.clearSession();

			// Only set error if it's not expected
			if (error.message !== 'Not authenticated' && error.message !== 'No refresh token available') {
				console.error('[AuthStore] Unexpected session check error:', error);
				this.state.error = 'Failed to check session';
			}
		} finally {
			this.state.isLoading = false;
		}
	}

	/**
	 * Set user after successful OAuth login
	 * Called by OAuth callback page after token exchange
	 */
	setUser(user: User) {
		this.state.user = user;
		this.state.isAuthenticated = true;
		this.state.error = null;
		console.log('[AuthStore] User authenticated:', user.email);
	}

	/**
	 * Logout user
	 * Clears OAuth tokens and redirects to Zitadel logout endpoint
	 */
	async logout() {
		this.state.error = null;

		try {
			console.log('[AuthStore] Logging out...');

			// OAuth logout (clears tokens and redirects to Zitadel end_session_endpoint)
			await oauthLogout();

			// Clear local state (logout() will redirect, so this might not execute)
			this.clearSession();
		} catch (error) {
			console.error('[AuthStore] Logout failed:', error);
			this.state.error = 'Failed to logout';

			// Clear local state anyway
			this.clearSession();

			// Redirect to home page as fallback
			window.location.href = '/';
		}
	}

	/**
	 * Clear local session state
	 */
	private clearSession() {
		this.state.user = null;
		this.state.isAuthenticated = false;
	}

	/**
	 * Refresh session data
	 * Useful after updating user profile or when tokens are refreshed
	 */
	async refreshSession() {
		await this.checkSession();
	}
}

// Export singleton instance
export const authStore = new AuthStore();
