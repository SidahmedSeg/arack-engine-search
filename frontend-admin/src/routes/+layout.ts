import { auth, authInitialized } from '$lib/stores/auth';
import { get } from 'svelte/store';
import { redirect } from '@sveltejs/kit';

export const ssr = false; // Client-side only

export async function load({ url }) {
	// Public routes that don't require authentication
	const publicRoutes = ['/login', '/invite'];
	const isPublicRoute = publicRoutes.some((route) => url.pathname.startsWith(route));

	// Initialize auth if not already initialized
	if (!get(authInitialized)) {
		await auth.initialize();
	}

	// Get current auth state
	const currentAuth = get(auth);

	// If not authenticated and trying to access protected route, redirect to login
	if (!currentAuth.user && !isPublicRoute) {
		throw redirect(303, '/login');
	}

	// If authenticated and trying to access login page, redirect to home
	if (currentAuth.user && url.pathname === '/login') {
		throw redirect(303, '/');
	}

	return {};
}
