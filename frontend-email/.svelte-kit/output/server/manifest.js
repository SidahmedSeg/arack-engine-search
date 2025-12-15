export const manifest = (() => {
function __memo(fn) {
	let value;
	return () => value ??= (value = fn());
}

return {
	appDir: "_app",
	appPath: "_app",
	assets: new Set(["arackmail.svg"]),
	mimeTypes: {".svg":"image/svg+xml"},
	_: {
		client: {start:"_app/immutable/entry/start.2Duxrq-T.js",app:"_app/immutable/entry/app.CyM4pttZ.js",imports:["_app/immutable/entry/start.2Duxrq-T.js","_app/immutable/chunks/D4mF4qri.js","_app/immutable/chunks/B4FSTR8c.js","_app/immutable/entry/app.CyM4pttZ.js","_app/immutable/chunks/B4FSTR8c.js","_app/immutable/chunks/BBx5xdYl.js","_app/immutable/chunks/BmKq7LR_.js","_app/immutable/chunks/Brwb1IUN.js","_app/immutable/chunks/ERC_K4WP.js"],stylesheets:[],fonts:[],uses_env_dynamic_public:false},
		nodes: [
			__memo(() => import('./nodes/0.js')),
			__memo(() => import('./nodes/1.js')),
			__memo(() => import('./nodes/2.js')),
			__memo(() => import('./nodes/3.js')),
			__memo(() => import('./nodes/4.js')),
			__memo(() => import('./nodes/5.js')),
			__memo(() => import('./nodes/6.js'))
		],
		remotes: {
			
		},
		routes: [
			{
				id: "/",
				pattern: /^\/$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 2 },
				endpoint: null
			},
			{
				id: "/drafts",
				pattern: /^\/drafts\/?$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 3 },
				endpoint: null
			},
			{
				id: "/inbox",
				pattern: /^\/inbox\/?$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 4 },
				endpoint: null
			},
			{
				id: "/sent",
				pattern: /^\/sent\/?$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 5 },
				endpoint: null
			},
			{
				id: "/trash",
				pattern: /^\/trash\/?$/,
				params: [],
				page: { layouts: [0,], errors: [1,], leaf: 6 },
				endpoint: null
			}
		],
		prerendered_routes: new Set([]),
		matchers: async () => {
			
			return {  };
		},
		server_assets: {}
	}
}
})();
