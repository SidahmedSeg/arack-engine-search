/**
 * Centrifugo Real-time Store
 *
 * Handles WebSocket connection to Centrifugo for real-time email notifications.
 */

import { Centrifuge, type Subscription, type PublicationContext } from 'centrifuge';
import { emailAPI } from '$lib/api/client';

// Centrifugo WebSocket URL
const CENTRIFUGO_URL = import.meta.env.VITE_CENTRIFUGO_URL || 'ws://localhost:8001/connection/websocket';

// Event types from backend
export interface NewEmailEvent {
	type: 'new_email';
	email_id: string;
	from: string;
	subject: string;
	preview: string;
	timestamp: string;
}

export interface EmailUpdatedEvent {
	type: 'email_updated';
	email_id: string;
	update_type: 'read' | 'unread' | 'moved' | 'deleted' | 'starred' | 'unstarred';
	timestamp: string;
}

export interface MailboxUpdatedEvent {
	type: 'mailbox_updated';
	mailbox_id: string;
	action: string;
	timestamp: string;
}

export type RealtimeEvent = NewEmailEvent | EmailUpdatedEvent | MailboxUpdatedEvent;

// Event handlers type
export interface RealtimeEventHandlers {
	onNewEmail?: (event: NewEmailEvent) => void;
	onEmailUpdated?: (event: EmailUpdatedEvent) => void;
	onMailboxUpdated?: (event: MailboxUpdatedEvent) => void;
	onConnectionStateChange?: (connected: boolean) => void;
}

class RealtimeStore {
	private centrifuge: Centrifuge | null = null;
	private subscription: Subscription | null = null;
	private handlers: RealtimeEventHandlers = {};
	private userId: string | null = null;
	private reconnectAttempts = 0;
	private maxReconnectAttempts = 5;

	// State using Svelte 5 runes
	connected = $state(false);
	connecting = $state(false);
	error = $state<string | null>(null);
	lastEvent = $state<RealtimeEvent | null>(null);
	notificationsEnabled = $state(false);

	/**
	 * Initialize and connect to Centrifugo
	 */
	async connect(userId: string, handlers: RealtimeEventHandlers = {}) {
		if (this.centrifuge && this.connected) {
			console.log('Already connected to Centrifugo');
			return;
		}

		this.userId = userId;
		this.handlers = handlers;
		this.connecting = true;
		this.error = null;

		try {
			// Get WebSocket token from backend (userId is extracted from session)
			const { token, channel } = await emailAPI.getWebSocketToken();

			// Create Centrifuge client
			this.centrifuge = new Centrifuge(CENTRIFUGO_URL, {
				token,
				debug: import.meta.env.DEV
			});

			// Setup connection event handlers
			this.setupConnectionHandlers();

			// Connect to Centrifugo
			this.centrifuge.connect();

			// Subscribe to user channel
			await this.subscribeToChannel(channel);

			// Request notification permission
			await this.requestNotificationPermission();

			console.log(`Connected to Centrifugo, subscribed to ${channel}`);
		} catch (err) {
			this.error = err instanceof Error ? err.message : 'Failed to connect to Centrifugo';
			console.error('Centrifugo connection error:', err);
			this.connecting = false;
		}
	}

	/**
	 * Setup Centrifuge connection event handlers
	 */
	private setupConnectionHandlers() {
		if (!this.centrifuge) return;

		this.centrifuge.on('connected', () => {
			this.connected = true;
			this.connecting = false;
			this.reconnectAttempts = 0;
			this.error = null;
			this.handlers.onConnectionStateChange?.(true);
			console.log('Centrifugo connected');
		});

		this.centrifuge.on('disconnected', () => {
			this.connected = false;
			this.handlers.onConnectionStateChange?.(false);
			console.log('Centrifugo disconnected');
		});

		this.centrifuge.on('error', (ctx) => {
			this.error = ctx.error?.message || 'Connection error';
			console.error('Centrifugo error:', ctx);
		});
	}

	/**
	 * Subscribe to user's email channel
	 */
	private async subscribeToChannel(channel: string) {
		if (!this.centrifuge) return;

		this.subscription = this.centrifuge.newSubscription(channel);

		// Handle incoming publications
		this.subscription.on('publication', (ctx: PublicationContext) => {
			this.handlePublication(ctx.data as RealtimeEvent);
		});

		this.subscription.on('subscribed', () => {
			console.log(`Subscribed to channel: ${channel}`);
		});

		this.subscription.on('error', (ctx) => {
			console.error(`Subscription error for ${channel}:`, ctx);
		});

		this.subscription.subscribe();
	}

	/**
	 * Handle incoming publication events
	 */
	private handlePublication(event: RealtimeEvent) {
		this.lastEvent = event;

		switch (event.type) {
			case 'new_email':
				this.handleNewEmail(event);
				break;
			case 'email_updated':
				this.handleEmailUpdated(event);
				break;
			case 'mailbox_updated':
				this.handleMailboxUpdated(event);
				break;
			default:
				console.warn('Unknown event type:', event);
		}
	}

	/**
	 * Handle new email event
	 */
	private handleNewEmail(event: NewEmailEvent) {
		console.log('New email received:', event);

		// Call handler if registered
		this.handlers.onNewEmail?.(event);

		// Show desktop notification
		this.showNotification(
			`New email from ${event.from}`,
			event.subject,
			event.preview
		);
	}

	/**
	 * Handle email updated event
	 */
	private handleEmailUpdated(event: EmailUpdatedEvent) {
		console.log('Email updated:', event);
		this.handlers.onEmailUpdated?.(event);
	}

	/**
	 * Handle mailbox updated event
	 */
	private handleMailboxUpdated(event: MailboxUpdatedEvent) {
		console.log('Mailbox updated:', event);
		this.handlers.onMailboxUpdated?.(event);
	}

	/**
	 * Request notification permission
	 */
	async requestNotificationPermission(): Promise<boolean> {
		if (typeof window === 'undefined' || !('Notification' in window)) {
			console.log('Notifications not supported');
			return false;
		}

		if (Notification.permission === 'granted') {
			this.notificationsEnabled = true;
			return true;
		}

		if (Notification.permission !== 'denied') {
			const permission = await Notification.requestPermission();
			this.notificationsEnabled = permission === 'granted';
			return this.notificationsEnabled;
		}

		return false;
	}

	/**
	 * Show desktop notification
	 */
	private showNotification(title: string, subject: string, body: string) {
		if (!this.notificationsEnabled || typeof window === 'undefined') return;

		try {
			const notification = new Notification(title, {
				body: `${subject}\n${body.substring(0, 100)}...`,
				icon: '/arackmail.svg',
				tag: 'arack-mail',
				requireInteraction: false
			});

			// Auto-close after 5 seconds
			setTimeout(() => notification.close(), 5000);

			// Handle click
			notification.onclick = () => {
				window.focus();
				notification.close();
			};
		} catch (err) {
			console.error('Failed to show notification:', err);
		}
	}

	/**
	 * Disconnect from Centrifugo
	 */
	disconnect() {
		if (this.subscription) {
			this.subscription.unsubscribe();
			this.subscription = null;
		}

		if (this.centrifuge) {
			this.centrifuge.disconnect();
			this.centrifuge = null;
		}

		this.connected = false;
		this.connecting = false;
		this.userId = null;
		this.handlers = {};
		console.log('Disconnected from Centrifugo');
	}

	/**
	 * Check if connected
	 */
	isConnected(): boolean {
		return this.connected;
	}

	/**
	 * Register event handlers
	 */
	setHandlers(handlers: RealtimeEventHandlers) {
		this.handlers = { ...this.handlers, ...handlers };
	}

	/**
	 * Reconnect with current user
	 */
	async reconnect() {
		if (this.userId) {
			this.disconnect();
			await this.connect(this.userId, this.handlers);
		}
	}
}

// Export singleton instance
export const realtimeStore = new RealtimeStore();
