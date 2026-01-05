// src/hooks.server.ts
// Server-side authentication check - runs BEFORE any page renders
// Phase 9: Updated to use arack_session cookie (from account-service) instead of ory_kratos_session

import { redirect } from '@sveltejs/kit';
import type { Handle } from '@sveltejs/kit';

// Account service URL for session validation (API calls)
const ACCOUNT_SERVICE_URL = 'https://account.arack.io';

// Login page URL (custom login page on main site)
const LOGIN_PAGE_URL = 'https://arack.io/auth/login';

export const handle: Handle = async ({ event, resolve }) => {
	const { url, cookies, fetch } = event;

	// Public routes that don't require authentication
	const publicRoutes = ['/oauth/auto', '/oauth/callback', '/consent'];
	const isPublicRoute = publicRoutes.some(route => url.pathname.startsWith(route));

	// Root path - will be redirected by +page.svelte
	if (url.pathname === '/') {
		return resolve(event);
	}

	// Protected routes require valid session
	const protectedRoutes = ['/inbox', '/sent', '/drafts', '/trash', '/priority', '/settings'];
	const isProtectedRoute = protectedRoutes.some(route => url.pathname.startsWith(route));

	if (isProtectedRoute) {
		// Check session status via account-service
		try {
			const sessionCookie = cookies.get('arack_session');

			if (!sessionCookie) {
				console.log('[hooks.server] No arack_session cookie, redirecting to login');
				// Redirect to main site login with return URL
				const returnUrl = encodeURIComponent(url.href);
				throw redirect(303, `${LOGIN_PAGE_URL}?return_url=${returnUrl}`);
			}

			// Validate session with account-service
			const response = await fetch(`${ACCOUNT_SERVICE_URL}/api/session`, {
				headers: {
					cookie: `arack_session=${sessionCookie}`
				}
			});

			if (!response.ok) {
				console.log('[hooks.server] Session validation failed, redirecting to login');
				const returnUrl = encodeURIComponent(url.href);
				throw redirect(303, `${LOGIN_PAGE_URL}?return_url=${returnUrl}`);
			}

			const session = await response.json();

			// Store session data in locals for handlers to use
			event.locals.user = {
				id: session.user_id,
				email: session.email,
				name: session.name
			};
			event.locals.accessToken = session.access_token;

			console.log('[hooks.server] Session valid for:', session.email);
		} catch (err) {
			// If redirect error, rethrow it
			if (err instanceof Response && err.status === 303) {
				throw err;
			}

			// Handle redirect() throwing a Redirect object
			if (err && typeof err === 'object' && 'status' in err && err.status === 303) {
				throw err;
			}

			// Otherwise, redirect to login on error
			console.error('[hooks.server] Error validating session:', err);
			const returnUrl = encodeURIComponent(url.href);
			throw redirect(303, `${LOGIN_PAGE_URL}?return_url=${returnUrl}`);
		}
	}

	return resolve(event);
};
