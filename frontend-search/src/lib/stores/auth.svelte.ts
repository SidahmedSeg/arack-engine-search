// Phase 8 - Kratos Migration: Authentication Store
// Manages user authentication state using Ory Kratos via backend proxy

import { whoami, logout as kratosLogout, type KratosSession } from '$lib/api/kratos';

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
	 * Check if user has an active session
	 * This should be called on app initialization
	 */
	async checkSession() {
		this.state.isLoading = true;
		this.state.error = null;

		try {
			const session: KratosSession = await whoami();

			if (session.authenticated) {
				this.state.user = {
					id: session.id,
					email: session.email,
					firstName: session.first_name || '',
					lastName: session.last_name || ''
				};
				this.state.isAuthenticated = true;
			} else {
				this.clearSession();
			}
		} catch (error: any) {
			// No active session or network error
			this.clearSession();

			// Only log error if it's not expected (not a 401)
			if (error.message !== 'Not authenticated') {
				console.error('Session check failed:', error);
				this.state.error = 'Failed to check session';
			}
		} finally {
			this.state.isLoading = false;
		}
	}

	/**
	 * Set user after successful login/registration
	 * Called by login/register pages after Kratos flow completes
	 */
	async setAuthenticated() {
		try {
			const session: KratosSession = await whoami();

			if (session.authenticated) {
				this.state.user = {
					id: session.id,
					email: session.email,
					firstName: session.first_name || '',
					lastName: session.last_name || ''
				};
				this.state.isAuthenticated = true;
				this.state.error = null;
			}
		} catch (error: any) {
			console.error('Failed to set authenticated state:', error);
			this.state.error = 'Failed to verify authentication';
		}
	}

	/**
	 * Logout user
	 * Clears the session cookie via Ory Kratos
	 */
	async logout() {
		this.state.error = null;

		try {
			await kratosLogout();

			// Clear local state
			this.clearSession();

			// Redirect to home page
			window.location.href = '/';
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
		this.state.user = null;
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
