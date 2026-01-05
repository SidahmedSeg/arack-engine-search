// src/routes/(authenticated)/+layout.server.ts
// Server-side data loading for authenticated routes
// Phase 9: Uses arack_session cookie for email-service API calls

import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = async ({ fetch, cookies, locals }) => {
	// hooks.server.ts already validated arack_session and populated locals
	const sessionCookie = cookies.get('arack_session');
	const accessToken = locals.accessToken;

	if (!sessionCookie) {
		console.error('[layout.server] No session cookie');
		return {
			account: null,
			quotaPercentage: 0,
			mailboxes: [],
			accessToken: null,
			wsToken: null,
			wsChannel: null
		};
	}

	try {
		// Fetch account info using session cookie
		// Email-service validates cookie with account-service
		const accountResponse = await fetch('https://api-mail.arack.io/api/mail/account/me', {
			headers: {
				'Cookie': `arack_session=${sessionCookie}`
			}
		});

		if (!accountResponse.ok) {
			console.error('[layout.server] Failed to fetch account:', accountResponse.status);
			throw new Error('Failed to fetch account info');
		}

		const accountData = await accountResponse.json();

		// Fetch mailboxes using session cookie
		const mailboxesResponse = await fetch('https://api-mail.arack.io/api/mail/mailboxes', {
			headers: {
				'Cookie': `arack_session=${sessionCookie}`
			}
		});

		if (!mailboxesResponse.ok) {
			console.error('[layout.server] Failed to fetch mailboxes:', mailboxesResponse.status);
			throw new Error('Failed to fetch mailboxes');
		}

		const mailboxesData = await mailboxesResponse.json();

		// Fetch WebSocket token server-side (Phase 9 fix)
		// This avoids client-side cross-subdomain cookie issues
		let wsToken: string | null = null;
		let wsChannel: string | null = null;
		try {
			const wsResponse = await fetch('https://api-mail.arack.io/api/mail/ws/token', {
				headers: {
					'Cookie': `arack_session=${sessionCookie}`
				}
			});
			if (wsResponse.ok) {
				const wsData = await wsResponse.json();
				wsToken = wsData.token;
				wsChannel = wsData.channel;
				console.log('[layout.server] WebSocket token fetched successfully');
			} else {
				console.error('[layout.server] Failed to fetch WS token:', wsResponse.status);
			}
		} catch (wsErr) {
			console.error('[layout.server] Error fetching WS token:', wsErr);
		}

		return {
			account: accountData.account,
			quotaPercentage: accountData.quota_percentage,
			mailboxes: mailboxesData.mailboxes,
			accessToken: accessToken, // Pass SSO token for client-side API calls
			wsToken,
			wsChannel
		};
	} catch (err) {
		console.error('[layout.server] Error loading data:', err);
		return {
			account: null,
			quotaPercentage: 0,
			mailboxes: [],
			accessToken: accessToken,
			wsToken: null,
			wsChannel: null
		};
	}
};
