
// this file is generated — do not edit it


declare module "svelte/elements" {
	export interface HTMLAttributes<T> {
		'data-sveltekit-keepfocus'?: true | '' | 'off' | undefined | null;
		'data-sveltekit-noscroll'?: true | '' | 'off' | undefined | null;
		'data-sveltekit-preload-code'?:
			| true
			| ''
			| 'eager'
			| 'viewport'
			| 'hover'
			| 'tap'
			| 'off'
			| undefined
			| null;
		'data-sveltekit-preload-data'?: true | '' | 'hover' | 'tap' | 'off' | undefined | null;
		'data-sveltekit-reload'?: true | '' | 'off' | undefined | null;
		'data-sveltekit-replacestate'?: true | '' | 'off' | undefined | null;
	}
}

export {};


declare module "$app/types" {
	export interface AppTypes {
		RouteId(): "/" | "/drafts" | "/inbox" | "/sent" | "/trash";
		RouteParams(): {
			
		};
		LayoutParams(): {
			"/": Record<string, never>;
			"/drafts": Record<string, never>;
			"/inbox": Record<string, never>;
			"/sent": Record<string, never>;
			"/trash": Record<string, never>
		};
		Pathname(): "/" | "/drafts" | "/drafts/" | "/inbox" | "/inbox/" | "/sent" | "/sent/" | "/trash" | "/trash/";
		ResolvedPathname(): `${"" | `/${string}`}${ReturnType<AppTypes['Pathname']>}`;
		Asset(): "/arackmail.svg" | string & {};
	}
}