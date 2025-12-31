import { format, formatDistance, parseISO } from 'date-fns';

/**
 * Format a date string to a human-readable format
 */
export function formatDate(dateString: string, formatStr: string = 'MMM d, yyyy'): string {
	try {
		const date = parseISO(dateString);
		return format(date, formatStr);
	} catch {
		return dateString;
	}
}

/**
 * Format a date as relative time (e.g., "2 hours ago")
 */
export function formatRelativeTime(dateString: string): string {
	try {
		const date = parseISO(dateString);
		return formatDistance(date, new Date(), { addSuffix: true });
	} catch {
		return dateString;
	}
}

/**
 * Extract domain from URL
 */
export function extractDomain(url: string): string {
	try {
		const urlObj = new URL(url);
		return urlObj.hostname;
	} catch {
		return url;
	}
}

/**
 * Truncate text to a maximum length with ellipsis
 */
export function truncateText(text: string, maxLength: number): string {
	if (text.length <= maxLength) {
		return text;
	}
	return text.substring(0, maxLength) + '...';
}

/**
 * Highlight search query in text
 */
export function highlightText(text: string, query: string): string {
	if (!query) return text;

	const regex = new RegExp(`(${query})`, 'gi');
	return text.replace(regex, '<mark class="bg-yellow-200">$1</mark>');
}

/**
 * Validate URL format
 */
export function isValidUrl(url: string): boolean {
	try {
		new URL(url);
		return true;
	} catch {
		return false;
	}
}

/**
 * Format number with commas (e.g., 1234 -> 1,234)
 */
export function formatNumber(num: number): string {
	return num.toLocaleString();
}

/**
 * Format file size (bytes to human-readable)
 */
export function formatBytes(bytes: number): string {
	if (bytes === 0) return '0 Bytes';

	const k = 1024;
	const sizes = ['Bytes', 'KB', 'MB', 'GB'];
	const i = Math.floor(Math.log(bytes) / Math.log(k));

	return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i];
}

/**
 * Debounce function for search input
 */
export function debounce<T extends (...args: any[]) => any>(
	func: T,
	wait: number
): (...args: Parameters<T>) => void {
	let timeout: ReturnType<typeof setTimeout> | null = null;

	return function (this: any, ...args: Parameters<T>) {
		const context = this;

		if (timeout) clearTimeout(timeout);

		timeout = setTimeout(() => {
			func.apply(context, args);
		}, wait);
	};
}

/**
 * Create query string from params object
 */
export function buildQueryString(params: Record<string, any>): string {
	const searchParams = new URLSearchParams();

	Object.entries(params).forEach(([key, value]) => {
		if (value !== undefined && value !== null && value !== '') {
			searchParams.append(key, String(value));
		}
	});

	const queryString = searchParams.toString();
	return queryString ? `?${queryString}` : '';
}
