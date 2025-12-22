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
interface KratosFlow {
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
 * This creates a new registration flow and returns the flow ID
 * along with CSRF token and other required fields.
 */
export async function initRegistrationFlow(): Promise<KratosFlow> {
	try {
		const response = await axios.get(`${API_BASE_URL}/api/auth/flows/registration`);
		return response.data.data;
	} catch (error: any) {
		console.error('Failed to initialize registration flow:', error);
		throw new Error(error.response?.data?.error || 'Failed to initialize registration');
	}
}

/**
 * Submit registration data
 *
 * Submits the registration form to Kratos with the flow ID.
 * On success, the user is registered and logged in.
 */
export async function submitRegistration(
	flowId: string,
	data: RegistrationData
): Promise<void> {
	try {
		// Use backend proxy instead of direct Kratos call
		const response = await axios.post(
			`${API_BASE_URL}/api/auth/flows/registration`,
			{
				flow_id: flowId,
				email: data.email,
				password: data.password,
				first_name: data.first_name,
				last_name: data.last_name,
				username: data.username,
				date_of_birth: data.date_of_birth,
				gender: data.gender
			},
			{
				headers: {
					'Content-Type': 'application/json'
				},
				withCredentials: true // Important: Include cookies for session
			}
		);

		// Registration successful - session cookie is automatically set
		return response.data;
	} catch (error: any) {
		console.error('Registration failed:', error);

		// Extract error message from backend response
		const errorMsg = error.response?.data?.error || 'Registration failed';
		throw new Error(errorMsg);
	}
}

/**
 * Initialize login flow
 *
 * This creates a new login flow and returns the flow ID
 * along with CSRF token and other required fields.
 */
export async function initLoginFlow(): Promise<KratosFlow> {
	try {
		const response = await axios.get(`${API_BASE_URL}/api/auth/flows/login`);
		return response.data.data;
	} catch (error: any) {
		console.error('Failed to initialize login flow:', error);
		throw new Error(error.response?.data?.error || 'Failed to initialize login');
	}
}

/**
 * Submit login credentials
 *
 * Submits the login form to Kratos with the flow ID.
 * On success, a session cookie is set.
 */
export async function submitLogin(flowId: string, data: LoginData): Promise<void> {
	try {
		// Use backend proxy instead of direct Kratos call
		const response = await axios.post(
			`${API_BASE_URL}/api/auth/flows/login`,
			{
				flow_id: flowId,
				identifier: data.identifier,
				password: data.password
			},
			{
				headers: {
					'Content-Type': 'application/json'
				},
				withCredentials: true // Important: Include cookies for session
			}
		);

		// Login successful - session cookie is automatically set
		return response.data;
	} catch (error: any) {
		console.error('Login failed:', error);

		// Extract error message from backend response
		const errorMsg = error.response?.data?.error || 'Invalid credentials';
		throw new Error(errorMsg);
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
		const response = await axios.get(`${API_BASE_URL}/api/auth/whoami`, {
			withCredentials: true // Important: Send cookies
		});

		if (response.data.success && response.data.data) {
			return response.data.data;
		}

		throw new Error('Not authenticated');
	} catch (error: any) {
		console.error('Whoami failed:', error);
		throw new Error(error.response?.data?.error || 'Not authenticated');
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
		// Get logout URL from backend
		const response = await axios.get(`${API_BASE_URL}/api/auth/flows/logout`, {
			withCredentials: true
		});

		const logoutUrl = response.data.data.logout_url;

		// Perform logout by calling Kratos logout URL
		await axios.get(logoutUrl, {
			withCredentials: true
		});

		// Logout successful - session cookie is cleared
	} catch (error: any) {
		console.error('Logout failed:', error);
		throw new Error(error.response?.data?.error || 'Logout failed');
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
