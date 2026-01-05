/**
 * OAuth 2.0 / OIDC Client for Arack Mail
 * Uses Authorization Code Flow with PKCE for secure authentication
 * Shares SSO with Arack Search via Zitadel
 *
 * Uses oauth4webapi v3 API
 */

import * as oauth from 'oauth4webapi';

// OAuth configuration from environment variables
const ISSUER = import.meta.env.VITE_OAUTH_ISSUER || 'https://auth.arack.io';
const CLIENT_ID = import.meta.env.VITE_OAUTH_CLIENT_ID || '353315592104640515';
const REDIRECT_URI =
	import.meta.env.VITE_OAUTH_REDIRECT_URI || 'https://mail.arack.io/oauth/callback';
const SCOPES = import.meta.env.VITE_OAUTH_SCOPES || 'openid email profile offline_access';

// Cached authorization server metadata
let authServer: oauth.AuthorizationServer | null = null;

// OAuth client configuration (Public Client with PKCE)
const client: oauth.Client = {
	client_id: CLIENT_ID,
	token_endpoint_auth_method: 'none'
};

// Client authentication for public client (no secret)
const clientAuth = oauth.None();

/**
 * Get OAuth authorization server metadata from discovery endpoint
 */
export async function getAuthServer(): Promise<oauth.AuthorizationServer> {
	if (authServer) return authServer;

	try {
		const issuer = new URL(ISSUER);
		const response = await oauth.discoveryRequest(issuer);
		authServer = await oauth.processDiscoveryResponse(issuer, response);
		return authServer;
	} catch (error) {
		console.error('[OAuth] Failed to fetch authorization server metadata:', error);
		throw new Error('Failed to initialize OAuth client');
	}
}

/**
 * Initiate OAuth login flow
 * Redirects user to Zitadel login
 */
export async function login() {
	try {
		const as = await getAuthServer();

		// Generate PKCE code verifier and challenge
		const code_verifier = oauth.generateRandomCodeVerifier();
		const code_challenge = await oauth.calculatePKCECodeChallenge(code_verifier);

		// Store code verifier in session storage (needed for token exchange)
		sessionStorage.setItem('oauth_code_verifier', code_verifier);

		// Generate state for CSRF protection
		const state = oauth.generateRandomState();
		sessionStorage.setItem('oauth_state', state);

		// Mark OAuth as pending (prevents race conditions)
		sessionStorage.setItem('oauth_pending', 'true');

		// Build authorization URL
		const authorizationUrl = new URL(as.authorization_endpoint!);
		authorizationUrl.searchParams.set('client_id', CLIENT_ID);
		authorizationUrl.searchParams.set('redirect_uri', REDIRECT_URI);
		authorizationUrl.searchParams.set('response_type', 'code');
		authorizationUrl.searchParams.set('scope', SCOPES);
		authorizationUrl.searchParams.set('code_challenge', code_challenge);
		authorizationUrl.searchParams.set('code_challenge_method', 'S256');
		authorizationUrl.searchParams.set('state', state);

		console.log('[OAuth] Redirecting to Zitadel:', authorizationUrl.toString());

		// Redirect to Zitadel
		window.location.href = authorizationUrl.toString();
	} catch (error) {
		console.error('[OAuth] Login initiation failed:', error);
		sessionStorage.removeItem('oauth_pending');
		throw error;
	}
}

/**
 * Silent OAuth login - uses prompt=none to avoid showing login UI
 * Requires existing Zitadel session (from SSO)
 * @param email - User's email for login_hint (skips account picker)
 */
export async function silentLogin(email: string) {
	try {
		const as = await getAuthServer();

		// Generate PKCE code verifier and challenge
		const code_verifier = oauth.generateRandomCodeVerifier();
		const code_challenge = await oauth.calculatePKCECodeChallenge(code_verifier);

		// Store code verifier in session storage
		sessionStorage.setItem('oauth_code_verifier', code_verifier);

		// Generate state for CSRF protection
		const state = oauth.generateRandomState();
		sessionStorage.setItem('oauth_state', state);

		// Mark that silent OAuth is in progress (prevents race conditions)
		sessionStorage.setItem('oauth_pending', 'true');

		// Build authorization URL with silent auth parameters
		const authorizationUrl = new URL(as.authorization_endpoint!);
		authorizationUrl.searchParams.set('client_id', CLIENT_ID);
		authorizationUrl.searchParams.set('redirect_uri', REDIRECT_URI);
		authorizationUrl.searchParams.set('response_type', 'code');
		authorizationUrl.searchParams.set('scope', SCOPES);
		authorizationUrl.searchParams.set('code_challenge', code_challenge);
		authorizationUrl.searchParams.set('code_challenge_method', 'S256');
		authorizationUrl.searchParams.set('state', state);

		// SILENT AUTH PARAMETERS
		authorizationUrl.searchParams.set('prompt', 'none'); // Don't show login UI
		authorizationUrl.searchParams.set('login_hint', email); // Skip account picker

		console.log('[OAuth] Silent auth redirect (prompt=none):', authorizationUrl.toString());

		// Redirect to Zitadel (will auto-complete if session exists)
		window.location.href = authorizationUrl.toString();
	} catch (error) {
		console.error('[OAuth] Silent login initiation failed:', error);
		sessionStorage.removeItem('oauth_pending');
		throw error;
	}
}

/**
 * Check if OAuth flow is currently in progress
 */
export function isOAuthPending(): boolean {
	if (typeof window === 'undefined') return false;
	return sessionStorage.getItem('oauth_pending') === 'true';
}

/**
 * Clear OAuth pending flag
 */
export function clearOAuthPending() {
	if (typeof window === 'undefined') return;
	sessionStorage.removeItem('oauth_pending');
}

/**
 * Handle OAuth callback
 * Exchange authorization code for tokens
 */
export async function handleCallback(callbackUrl: string): Promise<{
	access_token: string;
	refresh_token?: string;
	id_token?: string;
	expires_in?: number;
}> {
	try {
		const as = await getAuthServer();
		const url = new URL(callbackUrl);

		console.log('[OAuth] Processing callback URL:', url.toString());

		// Validate authorization response - throws AuthorizationResponseError on OAuth errors
		let params: URLSearchParams;
		try {
			params = oauth.validateAuthResponse(as, client, url);
		} catch (error) {
			if (error instanceof oauth.AuthorizationResponseError) {
				const oauthError = error.cause as oauth.OAuth2Error;
				console.error('[OAuth] Authorization error:', oauthError);
				throw new Error(
					`OAuth error: ${oauthError.error} - ${oauthError.error_description || 'Unknown error'}`
				);
			}
			throw error;
		}

		// Verify state (CSRF protection)
		const storedState = sessionStorage.getItem('oauth_state');
		const returnedState = params.get('state');

		console.log('[OAuth] State verification:', { stored: storedState, returned: returnedState });

		if (returnedState !== storedState) {
			throw new Error('Invalid state parameter - possible CSRF attack');
		}

		// Retrieve PKCE code verifier
		const code_verifier = sessionStorage.getItem('oauth_code_verifier');
		if (!code_verifier) {
			throw new Error('PKCE code verifier not found in session');
		}

		console.log('[OAuth] Exchanging authorization code for tokens...');

		// Exchange authorization code for tokens (public client with PKCE)
		const response = await oauth.authorizationCodeGrantRequest(
			as,
			client,
			clientAuth,
			params,
			REDIRECT_URI,
			code_verifier
		);

		// Process token response - throws ResponseBodyError on OAuth errors
		let result: oauth.TokenEndpointResponse;
		try {
			result = await oauth.processAuthorizationCodeResponse(as, client, response);
		} catch (error) {
			if (error instanceof oauth.ResponseBodyError) {
				const oauthError = error.cause as oauth.OAuth2Error;
				console.error('[OAuth] Token exchange error:', oauthError);
				throw new Error(
					`Token error: ${oauthError.error} - ${oauthError.error_description || 'Unknown error'}`
				);
			}
			throw error;
		}

		console.log('[OAuth] Token exchange successful');

		// Clean up session storage
		sessionStorage.removeItem('oauth_code_verifier');
		sessionStorage.removeItem('oauth_state');

		// Store tokens in localStorage
		localStorage.setItem('access_token', result.access_token);
		if (result.refresh_token) {
			localStorage.setItem('refresh_token', result.refresh_token);
		}
		if (result.id_token) {
			localStorage.setItem('id_token', result.id_token);
		}
		if (result.expires_in) {
			const expiresAt = Date.now() + result.expires_in * 1000;
			localStorage.setItem('token_expires_at', expiresAt.toString());
		}

		return {
			access_token: result.access_token,
			refresh_token: result.refresh_token,
			id_token: result.id_token,
			expires_in: result.expires_in
		};
	} catch (error) {
		console.error('[OAuth] Callback handling failed:', error);

		// Clean up on error
		sessionStorage.removeItem('oauth_code_verifier');
		sessionStorage.removeItem('oauth_state');

		throw error;
	}
}

/**
 * Get current user info from userinfo endpoint
 */
export async function getUserInfo(): Promise<{
	sub: string;
	email?: string;
	email_verified?: boolean;
	name?: string;
	given_name?: string;
	family_name?: string;
	picture?: string;
}> {
	const accessToken = localStorage.getItem('access_token');
	if (!accessToken) {
		throw new Error('Not authenticated - no access token');
	}

	try {
		const as = await getAuthServer();

		console.log('[OAuth] Fetching user info...');

		// Call userinfo endpoint
		const response = await oauth.userInfoRequest(as, client, accessToken);
		const userInfo = await oauth.processUserInfoResponse(
			as,
			client,
			oauth.skipSubjectCheck,
			response
		);

		console.log('[OAuth] User info retrieved successfully');

		return userInfo as {
			sub: string;
			email?: string;
			email_verified?: boolean;
			name?: string;
			given_name?: string;
			family_name?: string;
			picture?: string;
		};
	} catch (error) {
		console.error('[OAuth] Failed to fetch user info:', error);
		throw new Error('Failed to fetch user information');
	}
}

/**
 * Refresh access token using refresh token
 */
export async function refreshAccessToken(): Promise<string> {
	const refreshToken = localStorage.getItem('refresh_token');
	if (!refreshToken) {
		throw new Error('No refresh token available');
	}

	try {
		const as = await getAuthServer();

		console.log('[OAuth] Refreshing access token...');

		// Refresh token (public client)
		const response = await oauth.refreshTokenGrantRequest(as, client, clientAuth, refreshToken);

		// Process refresh token response - throws ResponseBodyError on OAuth errors
		let result: oauth.TokenEndpointResponse;
		try {
			result = await oauth.processRefreshTokenResponse(as, client, response);
		} catch (error) {
			if (error instanceof oauth.ResponseBodyError) {
				const oauthError = error.cause as oauth.OAuth2Error;
				console.error('[OAuth] Token refresh error:', oauthError);
				throw new Error(`Refresh error: ${oauthError.error}`);
			}
			throw error;
		}

		console.log('[OAuth] Access token refreshed successfully');

		// Update stored tokens
		localStorage.setItem('access_token', result.access_token);
		if (result.refresh_token) {
			localStorage.setItem('refresh_token', result.refresh_token);
		}
		if (result.expires_in) {
			const expiresAt = Date.now() + result.expires_in * 1000;
			localStorage.setItem('token_expires_at', expiresAt.toString());
		}

		return result.access_token;
	} catch (error) {
		console.error('[OAuth] Token refresh failed:', error);

		// Clear tokens on refresh failure
		clearTokens();

		throw error;
	}
}

/**
 * Logout user
 * Clears local tokens and redirects to end session endpoint
 */
export async function logout() {
	const idToken = localStorage.getItem('id_token');

	try {
		const as = await getAuthServer();

		console.log('[OAuth] Logging out...');

		// Clear local tokens first
		clearTokens();

		// Redirect to Zitadel logout endpoint if available
		if (as.end_session_endpoint && idToken) {
			const logoutUrl = new URL(as.end_session_endpoint);
			logoutUrl.searchParams.set('id_token_hint', idToken);
			logoutUrl.searchParams.set('post_logout_redirect_uri', window.location.origin);

			window.location.href = logoutUrl.toString();
		} else {
			// Fallback: just clear tokens and redirect to home
			window.location.href = '/';
		}
	} catch (error) {
		console.error('[OAuth] Logout failed:', error);

		// Still clear tokens and redirect on error
		clearTokens();
		window.location.href = '/';
	}
}

/**
 * Check if user is authenticated
 */
export function isAuthenticated(): boolean {
	const accessToken = localStorage.getItem('access_token');
	const expiresAt = localStorage.getItem('token_expires_at');

	if (!accessToken) {
		return false;
	}

	// Check if token is expired
	if (expiresAt) {
		const expiryTime = parseInt(expiresAt, 10);
		if (Date.now() >= expiryTime) {
			console.log('[OAuth] Access token expired');
			return false;
		}
	}

	return true;
}

/**
 * Check if access token needs refresh (expires in less than 5 minutes)
 */
export function needsRefresh(): boolean {
	const expiresAt = localStorage.getItem('token_expires_at');
	const refreshToken = localStorage.getItem('refresh_token');

	if (!expiresAt || !refreshToken) {
		return false;
	}

	const expiryTime = parseInt(expiresAt, 10);
	const fiveMinutes = 5 * 60 * 1000;

	return Date.now() >= expiryTime - fiveMinutes;
}

/**
 * Clear all stored tokens
 */
function clearTokens() {
	localStorage.removeItem('access_token');
	localStorage.removeItem('refresh_token');
	localStorage.removeItem('id_token');
	localStorage.removeItem('token_expires_at');
}

/**
 * Get access token (with auto-refresh if needed)
 */
export async function getAccessToken(): Promise<string | null> {
	if (!isAuthenticated()) {
		return null;
	}

	// Auto-refresh if token is about to expire
	if (needsRefresh()) {
		try {
			console.log('[OAuth] Token expiring soon, refreshing...');
			return await refreshAccessToken();
		} catch (error) {
			console.error('[OAuth] Auto-refresh failed:', error);
			return null;
		}
	}

	return localStorage.getItem('access_token');
}
