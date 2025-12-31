/**
 * Kratos API Helper Library
 *
 * This module provides helper functions for interacting with Ory Kratos
 * authentication flows via our backend proxy endpoints.
 */

import axios from 'axios';

const API_BASE_URL = import.meta.env.VITE_API_URL || 'https://api.arack.io';
const KRATOS_PUBLIC_URL = 'http://127.0.0.1:4433';

/**
 * Kratos Flow Response
 */
export interface KratosFlow {
	id: string;
	type?: string;
	expires_at?: string;
	issued_at?: string;
	request_url?: string;
	ui: {
		action: string;
		method: string;
		nodes: Array<{
			type: string;
			group: string;
			attributes: {
				name: string;
				type?: string;
				value?: any;
				required?: boolean;
				disabled?: boolean;
			};
			messages?: Array<{
				id: number;
				text: string;
				type: string;
			}>;
		}>;
		messages?: Array<{
			id: number;
			text: string;
			type: string;
		}>;
	};
}

/**
 * Kratos Session Response
 */
export interface KratosSession {
	id: string;
	email: string;
	first_name?: string;
	last_name?: string;
	authenticated: boolean;
}

/**
 * Registration Data
 */
export interface RegistrationData {
	email: string;
	password: string;
	first_name: string;
	last_name: string;
	username: string;
	date_of_birth: string;
	gender: string;
}

/**
 * Login Data
 */
export interface LoginData {
	identifier: string;
	password: string;
}

/**
 * Initialize registration flow
 *
 * This creates a new registration flow directly from Kratos and returns the complete flow
 * including the CSRF token and action URL.
 */
export async function initRegistrationFlow(): Promise<KratosFlow> {
	try {
		// Call Kratos directly (not through backend proxy) to get proper CSRF token
		const response = await axios.get('https://auth.arack.io/self-service/registration/api', {
			withCredentials: true // Important: Get CSRF cookie
		});
		return response.data;
	} catch (error: any) {
		console.error('Failed to initialize registration flow:', error);
		throw new Error(error.response?.data?.error || 'Failed to initialize registration');
	}
}

/**
 * Submit registration data
 *
 * Submits the registration form directly to Kratos using the flow's action URL.
 * On success, the user is registered and logged in.
 *
 * Following Ory best practices: accepts complete flow object (no second fetch)
 */
export async function submitRegistration(
	flow: KratosFlow,
	data: RegistrationData
): Promise<void> {
	try {
		// Extract CSRF token from the flow object
		const csrfToken = getCsrfToken(flow);
		if (!csrfToken) {
			throw new Error('CSRF token not found in flow');
		}

		// Submit directly to Kratos using the flow's action URL
		const response = await axios.post(
			flow.ui.action,
			{
				csrf_token: csrfToken,
				method: 'password',
				'traits.email': data.email,
				password: data.password,
				'traits.first_name': data.first_name,
				'traits.last_name': data.last_name,
				'traits.username': data.username,
				'traits.date_of_birth': data.date_of_birth,
				'traits.gender': data.gender
			},
			{
				headers: {
					'Content-Type': 'application/json',
					Accept: 'application/json'
				},
				withCredentials: true // Important: Include CSRF cookie
			}
		);

		// Registration successful - session cookie is automatically set
		return response.data;
	} catch (error: any) {
		console.error('Registration failed:', error);

		// Extract error message from Kratos response
		const kratosError = error.response?.data?.error;
		if (kratosError) {
			throw new Error(`Registration failed: ${JSON.stringify(kratosError)}`);
		}

		throw new Error(error.response?.data?.message || 'Registration failed');
	}
}

/**
 * Initialize login flow
 *
 * This creates a new login flow directly from Kratos and returns the complete flow
 * including the CSRF token and action URL.
 */
export async function initLoginFlow(): Promise<KratosFlow> {
	try {
		// Call Kratos directly (not through backend proxy) to get proper CSRF token
		const response = await axios.get('https://auth.arack.io/self-service/login/api', {
			withCredentials: true // Important: Get CSRF cookie
		});
		return response.data;
	} catch (error: any) {
		console.error('Failed to initialize login flow:', error);
		throw new Error(error.response?.data?.error || 'Failed to initialize login');
	}
}

/**
 * Submit login credentials
 *
 * Submits the login form directly to Kratos using the flow's action URL.
 * On success, a session cookie is set.
 *
 * Following Ory best practices: accepts complete flow object (no second fetch)
 */
export async function submitLogin(flow: KratosFlow, data: LoginData): Promise<void> {
	try {
		// Extract CSRF token from the flow object
		const csrfToken = getCsrfToken(flow);
		if (!csrfToken) {
			throw new Error('CSRF token not found in flow');
		}

		// Submit directly to Kratos using the flow's action URL
		const response = await axios.post(
			flow.ui.action,
			{
				csrf_token: csrfToken,
				method: 'password',
				identifier: data.identifier,
				password: data.password
			},
			{
				headers: {
					'Content-Type': 'application/json',
					Accept: 'application/json'
				},
				withCredentials: true // Important: Include CSRF cookie
			}
		);

		// Login successful - session cookie is automatically set
		return response.data;
	} catch (error: any) {
		console.error('Login failed:', error);

		// Extract error message from Kratos response
		const kratosError = error.response?.data?.error;
		if (kratosError) {
			throw new Error(`Login failed: ${JSON.stringify(kratosError)}`);
		}

		throw new Error(error.response?.data?.message || 'Invalid credentials');
	}
}

/**
 * Get current session (whoami)
 *
 * Checks if the user is authenticated by validating the session cookie.
 * Returns user information if authenticated, throws error otherwise.
 */
export async function whoami(): Promise<KratosSession> {
	try {
		// Call Kratos directly to check session
		const response = await axios.get('https://auth.arack.io/sessions/whoami', {
			withCredentials: true // Important: Send session cookie
		});

		// Kratos returns session data directly
		const session = response.data;

		// Extract user information from Kratos identity
		const identity = session.identity;
		const traits = identity?.traits || {};

		return {
			id: identity?.id || '',
			email: traits.email || '',
			first_name: traits.first_name || traits.name?.first || '',
			last_name: traits.last_name || traits.name?.last || '',
			authenticated: session.active || false
		};
	} catch (error: any) {
		// 401 means not authenticated (expected for logged-out users)
		if (error.response?.status === 401) {
			throw new Error('Not authenticated');
		}

		console.error('Whoami failed:', error);
		throw new Error(error.response?.data?.error?.message || 'Not authenticated');
	}
}

/**
 * Logout current user
 *
 * Initiates the logout flow and redirects to complete logout.
 * Clears the session cookie.
 */
export async function logout(): Promise<void> {
	try {
		// Create logout flow directly from Kratos
		const response = await axios.get('https://auth.arack.io/self-service/logout/browser', {
			withCredentials: true
		});

		const logoutToken = response.data.logout_token;

		// Execute logout using the token
		await axios.get(`https://auth.arack.io/self-service/logout?token=${logoutToken}`, {
			withCredentials: true
		});

		// Logout successful - session cookie is cleared
	} catch (error: any) {
		console.error('Logout failed:', error);
		throw new Error(error.response?.data?.error?.message || 'Logout failed');
	}
}

/**
 * Check if flow has expired
 *
 * Kratos flows have a 10-minute lifetime. This helper checks if a flow
 * has expired and needs to be reinitialized.
 */
export function isFlowExpired(flow: KratosFlow): boolean {
	if (!flow.expires_at) return false;

	const expiresAt = new Date(flow.expires_at);
	const now = new Date();

	return now >= expiresAt;
}

/**
 * Extract CSRF token from flow
 *
 * Helper to extract the CSRF token from a Kratos flow.
 * This is needed for form submission.
 */
export function getCsrfToken(flow: KratosFlow): string | null {
	const csrfNode = flow.ui.nodes.find(
		(node) => node.attributes.name === 'csrf_token'
	);

	return csrfNode?.attributes.value || null;
}

/**
 * Extract error messages from flow
 *
 * Helper to extract user-friendly error messages from a Kratos flow response.
 */
export function getFlowErrors(flow: KratosFlow): string[] {
	const errors: string[] = [];

	// Global messages
	if (flow.ui.messages) {
		errors.push(...flow.ui.messages.map((msg) => msg.text));
	}

	// Field-specific messages
	flow.ui.nodes.forEach((node) => {
		if (node.messages) {
			errors.push(...node.messages.map((msg) => msg.text));
		}
	});

	return errors;
}
