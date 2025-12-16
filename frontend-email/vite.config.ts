import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [sveltekit()],
	server: {
		port: 5006,
		host: '127.0.0.1' // Bind to 127.0.0.1 for IPv4 access
	}
});
