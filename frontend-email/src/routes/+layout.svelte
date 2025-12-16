<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import { emailStore } from '$lib/stores/email.svelte';

	let { children } = $props();

	// Dark mode state (persisted in localStorage)
	let darkMode = $state(false);

	onMount(async () => {
		// Load dark mode preference from localStorage
		const stored = localStorage.getItem('darkMode');
		darkMode = stored === 'true';
		updateDarkMode();

		// Initialize email store with user account
		try {
			await emailStore.initialize();
			console.log('Email store initialized with account:', emailStore.accountInfo?.email);
		} catch (err) {
			console.error('Failed to initialize email store:', err);
			// Optionally redirect to login page if session expired
			// window.location.href = 'http://localhost:5001/auth/login';
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
