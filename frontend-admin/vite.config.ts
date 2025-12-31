import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import path from 'path';

export default defineConfig({
	plugins: [sveltekit()],
	server: {
		port: 5002
	},
	resolve: {
		alias: {
			'$shared': path.resolve(__dirname, '../shared')
		}
	}
});
