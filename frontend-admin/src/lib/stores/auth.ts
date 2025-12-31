// Phase 9 - Central SSO: Authentication Store for Admin Dashboard
// Manages user authentication state via account.arack.io shared session

import { writable, derived } from 'svelte/store';
import { getSession, login as ssoLogin, logout as ssoLogout } from '$lib/auth/sso';
import type { User } from '$shared/types';

// Auth state
interface AuthState {
	user: User | null;
	accessToken: string | null;
	loading: boolean;
	initialized: boolean;
}

function createAuthStore() {
	const { subscribe, set, update } = writable<AuthState>({
		user: null,
		accessToken: null,
		loading: false,
		initialized: false,
	});

	return {
		subscribe,

		/**
		 * Initialize auth by checking SSO session
		 */
		async initialize() {
			update((state) => ({ ...state, loading: true }));

			try {
				const session = await getSession();

				if (session) {
					// Parse name into first/last (best effort)
					const nameParts = (session.name || '').split(' ');
					const firstName = nameParts[0] || '';
					const lastName = nameParts.slice(1).join(' ') || '';

					const user: User = {
						id: session.user_id,
						email: session.email,
						first_name: firstName,
						last_name: lastName,
						role: 'admin', // Admin dashboard assumes admin role
						created_at: new Date().toISOString(),
					};

					set({
						user,
						accessToken: session.access_token,
						loading: false,
						initialized: true
					});
					console.log('[AuthStore] SSO session restored:', session.email);
				} else {
					console.log('[AuthStore] No SSO session found');
					set({ user: null, accessToken: null, loading: false, initialized: true });
				}
			} catch (error) {
				console.error('Failed to initialize auth:', error);
				set({ user: null, accessToken: null, loading: false, initialized: true });
			}
		},

		/**
		 * Login via SSO
		 * Redirects to account.arack.io login page
		 */
		login(returnUrl?: string) {
			update((state) => ({ ...state, loading: true }));
			ssoLogin(returnUrl || window.location.href);
		},

		/**
		 * Logout via SSO
		 * Clears session across all arack.io apps
		 */
		async logout() {
			update((state) => ({ ...state, loading: true }));

			try {
				await ssoLogout();
				// ssoLogout redirects, so this might not execute
				set({ user: null, accessToken: null, loading: false, initialized: true });
			} catch (error) {
				console.error('Logout failed:', error);
				set({ user: null, accessToken: null, loading: false, initialized: true });
			}
		},

		/**
		 * Get current access token
		 */
		getAccessToken(): string | null {
			let token: string | null = null;
			subscribe((state) => {
				token = state.accessToken;
			})();
			return token;
		},

		/**
		 * Clear auth state (for logout or errors)
		 */
		clear() {
			set({ user: null, accessToken: null, loading: false, initialized: true });
		},
	};
}

export const auth = createAuthStore();

// Derived stores for convenience
export const user = derived(auth, ($auth) => $auth.user);
export const isAuthenticated = derived(auth, ($auth) => $auth.user !== null);
export const isAdmin = derived(auth, ($auth) => $auth.user?.role === 'admin');
export const authLoading = derived(auth, ($auth) => $auth.loading);
export const authInitialized = derived(auth, ($auth) => $auth.initialized);
