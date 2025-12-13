// Phase 8.6: Ory Kratos Authentication Store
// Manages user authentication state using Ory Kratos flows

import { FrontendApi, Configuration, type Session, type Identity } from '@ory/client';

// Initialize Ory Kratos client
const ory = new FrontendApi(
	new Configuration({
		basePath: 'http://127.0.0.1:4433',
		baseOptions: {
			withCredentials: true
		}
	})
);

// User interface derived from identity traits
interface User {
	id: string;
	email: string;
	firstName: string;
	lastName: string;
}

// Auth state interface
interface AuthState {
	session: Session | null;
	identity: Identity | null;
	isAuthenticated: boolean;
	isLoading: boolean;
	error: string | null;
}

class AuthStore {
	private state = $state<AuthState>({
		session: null,
		identity: null,
		isAuthenticated: false,
		isLoading: true,
		error: null
	});

	// Getters for reactive state
	get session() {
		return this.state.session;
	}

	get identity() {
		return this.state.identity;
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

	// Derived user object from identity traits
	get user(): User | null {
		if (!this.state.identity) return null;

		const traits = this.state.identity.traits as {
			email?: string;
			first_name?: string;
			last_name?: string;
		};

		return {
			id: this.state.identity.id,
			email: traits.email || '',
			firstName: traits.first_name || '',
			lastName: traits.last_name || ''
		};
	}

	/**
	 * Check if user has an active session
	 * This should be called on app initialization
	 */
	async checkSession() {
		this.state.isLoading = true;
		this.state.error = null;

		try {
			const { data: session } = await ory.toSession();

			if (session.active) {
				this.state.session = session;
				this.state.identity = session.identity;
				this.state.isAuthenticated = true;
			} else {
				this.clearSession();
			}
		} catch (error: any) {
			// No active session or network error
			this.clearSession();

			// Only set error if it's not a 401 (expected when not logged in)
			if (error?.response?.status !== 401) {
				console.error('Session check failed:', error);
				this.state.error = 'Failed to check session';
			}
		} finally {
			this.state.isLoading = false;
		}
	}

	/**
	 * Logout user by creating a logout flow
	 * This clears the session cookie via Ory Kratos
	 */
	async logout() {
		this.state.error = null;

		try {
			const { data: logoutFlow } = await ory.createBrowserLogoutFlow();

			// Navigate to logout URL to complete the logout
			window.location.href = logoutFlow.logout_url;
		} catch (error) {
			console.error('Logout failed:', error);
			this.state.error = 'Failed to logout';

			// Clear local state anyway
			this.clearSession();
		}
	}

	/**
	 * Clear local session state
	 */
	private clearSession() {
		this.state.session = null;
		this.state.identity = null;
		this.state.isAuthenticated = false;
	}

	/**
	 * Refresh session data
	 * Useful after updating user profile
	 */
	async refreshSession() {
		await this.checkSession();
	}
}

// Export singleton instance
export const authStore = new AuthStore();

// Export Ory client for use in auth flows
export { ory };

// Export types
export type { User };
