<script lang="ts">
	import '../../app.css';
	import { onMount } from 'svelte';
	import { emailStore } from '$lib/stores/email.svelte';
	import { realtimeStore } from '$lib/stores/realtime.svelte';
	import { setAccessToken } from '$lib/stores/token';
	import type { LayoutData } from './$types';

	let { children, data }: { children: any; data: LayoutData } = $props();

	// Dark mode state (persisted in localStorage)
	let darkMode = $state(false);

	// Store the SSO access token for API calls (Phase 9)
	// This runs immediately, before onMount, so token is available for API calls
	$effect(() => {
		if (data.accessToken) {
			setAccessToken(data.accessToken);
			console.log('[Layout] SSO access token stored for API calls');
		}
	});

	// Store the pre-fetched WebSocket token (Phase 9)
	// This ensures realtimeStore has the token before pages try to connect
	$effect(() => {
		if (data.wsToken && data.wsChannel) {
			realtimeStore.setToken(data.wsToken, data.wsChannel);
		}
	});

	onMount(async () => {
		// Load dark mode preference from localStorage
		const stored = localStorage.getItem('darkMode');
		darkMode = stored === 'true';
		updateDarkMode();

		// Initialize email store with server-loaded data (NO API calls!)
		if (data.account) {
			emailStore.accountId = data.account.id;
			emailStore.userId = data.account.kratos_identity_id;
			emailStore.accountInfo = {
				email: data.account.email_address,
				quotaPercentage: data.quotaPercentage,
				quotaUsed: data.account.storage_used_bytes,
				quotaTotal: data.account.storage_quota_bytes
			};
			emailStore.mailboxes = data.mailboxes;
			console.log('[Layout] Email store initialized from server data:', emailStore.accountInfo?.email);
		}
	});

	function updateDarkMode() {
		if (darkMode) {
			document.documentElement.classList.add('dark');
		} else {
			document.documentElement.classList.remove('dark');
		}
		localStorage.setItem('darkMode', darkMode.toString());
	}

	function toggleDarkMode() {
		darkMode = !darkMode;
		updateDarkMode();
	}

	// Expose toggle function globally for components
	if (typeof window !== 'undefined') {
		(window as any).toggleDarkMode = toggleDarkMode;
	}
</script>

{@render children()}
