import { type ClassValue, clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';

/**
 * Merge Tailwind CSS classes with conflict resolution
 */
export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs));
}

/**
 * Format timestamp to human-readable format (Gmail-style)
 */
export function formatTimestamp(date: Date | string): string {
	const d = typeof date === 'string' ? new Date(date) : date;
	const now = new Date();
	const diff = now.getTime() - d.getTime();
	const diffDays = Math.floor(diff / (1000 * 60 * 60 * 24));

	// Today - show time
	if (diffDays === 0) {
		return d.toLocaleTimeString('en-US', { hour: 'numeric', minute: '2-digit', hour12: true });
	}

	// Yesterday
	if (diffDays === 1) {
		return 'Yesterday';
	}

	// Within a week - show day name
	if (diffDays < 7) {
		return d.toLocaleDateString('en-US', { weekday: 'short' });
	}

	// This year - show month and day
	if (d.getFullYear() === now.getFullYear()) {
		return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
	}

	// Older - show full date
	return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
}

/**
 * Generate Gravatar URL from email
 */
export function getGravatarUrl(email: string, size: number = 40): string {
	const hash = email.trim().toLowerCase();
	// Note: In production, you'd hash the email with MD5
	// For now, using a placeholder that generates consistent avatars
	return `https://www.gravatar.com/avatar/${hash}?s=${size}&d=identicon`;
}

/**
 * Get initials from name for avatar fallback
 */
export function getInitials(name: string): string {
	const parts = name.trim().split(' ');
	if (parts.length >= 2) {
		return (parts[0][0] + parts[parts.length - 1][0]).toUpperCase();
	}
	return name.substring(0, 2).toUpperCase();
}

/**
 * Truncate text to specified length
 */
export function truncate(text: string, maxLength: number): string {
	if (text.length <= maxLength) return text;
	return text.substring(0, maxLength) + '...';
}

/**
 * Format file size to human-readable format
 */
export function formatFileSize(bytes: number): string {
	if (bytes < 1024) return bytes + ' B';
	if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
	if (bytes < 1024 * 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(1) + ' MB';
	return (bytes / (1024 * 1024 * 1024)).toFixed(1) + ' GB';
}
