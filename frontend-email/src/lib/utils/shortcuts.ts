/**
 * Keyboard shortcuts for email application
 * Gmail-like keyboard shortcuts
 */

export interface KeyboardShortcut {
	key: string;
	description: string;
	handler: () => void;
	requiresMeta?: boolean; // Cmd/Ctrl key
	requiresShift?: boolean;
}

export class ShortcutManager {
	private shortcuts: Map<string, KeyboardShortcut> = new Map();
	private enabled = true;

	constructor() {
		this.setupListener();
	}

	private setupListener() {
		if (typeof window !== 'undefined') {
			window.addEventListener('keydown', this.handleKeyDown.bind(this));
		}
	}

	private handleKeyDown(e: KeyboardEvent) {
		if (!this.enabled) return;

		// Don't trigger shortcuts when typing in input fields
		const target = e.target as HTMLElement;
		if (
			target.tagName === 'INPUT' ||
			target.tagName === 'TEXTAREA' ||
			target.isContentEditable
		) {
			return;
		}

		// Build shortcut key
		let shortcutKey = e.key.toLowerCase();
		if (e.metaKey || e.ctrlKey) shortcutKey = `meta+${shortcutKey}`;
		if (e.shiftKey) shortcutKey = `shift+${shortcutKey}`;

		// Check if shortcut exists
		const shortcut = this.shortcuts.get(shortcutKey);
		if (shortcut) {
			e.preventDefault();
			shortcut.handler();
		}
	}

	register(key: string, description: string, handler: () => void) {
		this.shortcuts.set(key, { key, description, handler });
	}

	unregister(key: string) {
		this.shortcuts.delete(key);
	}

	disable() {
		this.enabled = false;
	}

	enable() {
		this.enabled = true;
	}

	getShortcuts(): KeyboardShortcut[] {
		return Array.from(this.shortcuts.values());
	}

	destroy() {
		if (typeof window !== 'undefined') {
			window.removeEventListener('keydown', this.handleKeyDown.bind(this));
		}
		this.shortcuts.clear();
	}
}

// Common email shortcuts
export const EMAIL_SHORTCUTS = {
	COMPOSE: 'c',
	REPLY: 'r',
	REPLY_ALL: 'a',
	FORWARD: 'f',
	ARCHIVE: 'e',
	DELETE: '#',
	STAR: 's',
	MARK_UNREAD: 'u',
	NEXT_MESSAGE: 'j',
	PREV_MESSAGE: 'k',
	OPEN_MESSAGE: 'enter',
	SEARCH: '/',
	ESCAPE: 'escape'
};
