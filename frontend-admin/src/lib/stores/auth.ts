import { writable, derived } from 'svelte/store';
import { api } from './api';
import type { User } from '$shared/types';

// Auth state
interface AuthState {
	user: User | null;
	loading: boolean;
	initialized: boolean;
}

function createAuthStore() {
	const { subscribe, set, update } = writable<AuthState>({
		user: null,
		loading: false,
		initialized: false,
	});

	return {
		subscribe,

		/**
		 * Initialize auth by checking current session
		 */
		async initialize() {
			update((state) => ({ ...state, loading: true }));

			try {
				const user = await api.getCurrentUser();
				set({ user, loading: false, initialized: true });
			} catch (error) {
				console.error('Failed to initialize auth:', error);
				set({ user: null, loading: false, initialized: true });
			}
		},

		/**
		 * Login with email and password
		 */
		async login(email: string, password: string) {
			update((state) => ({ ...state, loading: true }));

			try {
				const user = await api.login({ email, password });
				set({ user, loading: false, initialized: true });
				return { success: true };
			} catch (error: any) {
				set({ user: null, loading: false, initialized: true });
				return {
					success: false,
					error: error.message || 'Login failed',
				};
			}
		},

		/**
		 * Logout current user
		 */
		async logout() {
			update((state) => ({ ...state, loading: true }));

			try {
				await api.logout();
				set({ user: null, loading: false, initialized: true });
			} catch (error) {
				console.error('Logout failed:', error);
				// Even if logout fails, clear local state
				set({ user: null, loading: false, initialized: true });
			}
		},

		/**
		 * Accept invitation and create account
		 */
		async acceptInvitation(
			token: string,
			password: string,
			firstName: string,
			lastName: string
		) {
			update((state) => ({ ...state, loading: true }));

			try {
				const user = await api.acceptInvitation(token, {
					password,
					first_name: firstName,
					last_name: lastName,
				});
				set({ user, loading: false, initialized: true });
				return { success: true };
			} catch (error: any) {
				set({ user: null, loading: false, initialized: true });
				return {
					success: false,
					error: error.message || 'Failed to accept invitation',
				};
			}
		},

		/**
		 * Clear auth state (for logout or errors)
		 */
		clear() {
			set({ user: null, loading: false, initialized: true });
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
