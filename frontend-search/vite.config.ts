import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import path from 'path';

export default defineConfig({
	plugins: [sveltekit()],
	server: {
		host: '127.0.0.1',
		port: 5001,
		strictPort: true
	},
	resolve: {
		alias: {
			'$shared': path.resolve(__dirname, '../shared')
		}
	}
});
