<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';

	let { children } = $props();

	// Dark mode state (persisted in localStorage)
	let darkMode = $state(false);

	onMount(() => {
		// Load dark mode preference from localStorage
		const stored = localStorage.getItem('darkMode');
		darkMode = stored === 'true';
		updateDarkMode();
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
