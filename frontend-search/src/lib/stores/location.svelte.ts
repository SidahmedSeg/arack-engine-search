// Location Detection Store
// Detects user location using IP-based geolocation with caching for performance

interface LocationData {
	city: string;
	region: string;
	country: string;
	countryCode: string;
	timezone: string;
	lat: number;
	lon: number;
}

interface LocationState {
	location: LocationData | null;
	isLoading: boolean;
	error: string | null;
	displayText: string;
}

const CACHE_KEY = 'user_location_cache';
const CACHE_DURATION = 24 * 60 * 60 * 1000; // 24 hours in milliseconds

class LocationStore {
	private state = $state<LocationState>({
		location: null,
		isLoading: false,
		error: null,
		displayText: 'Detecting location...'
	});

	// Getters for reactive state
	get location() {
		return this.state.location;
	}

	get isLoading() {
		return this.state.isLoading;
	}

	get error() {
		return this.state.error;
	}

	get displayText() {
		return this.state.displayText;
	}

	/**
	 * Initialize location detection
	 * Checks cache first, then fetches from API if needed
	 */
	async init() {
		// Check if already loaded or loading
		if (this.state.location || this.state.isLoading) {
			return;
		}

		// Try to load from cache first
		const cached = this.loadFromCache();
		if (cached) {
			this.state.location = cached;
			this.updateDisplayText(cached);
			return;
		}

		// Fetch from API
		await this.fetchLocation();
	}

	/**
	 * Fetch location from IP-based geolocation API
	 * Uses ip-api.com - free tier with 45 requests/minute (no API key needed)
	 */
	private async fetchLocation() {
		this.state.isLoading = true;
		this.state.error = null;

		try {
			// Using ip-api.com - no API key needed, 45 requests/minute free
			const response = await fetch('http://ip-api.com/json/?fields=status,message,country,countryCode,region,city,timezone,lat,lon', {
				method: 'GET'
			});

			if (!response.ok) {
				throw new Error('Failed to fetch location');
			}

			const data = await response.json();

			// Check if the API returned an error status
			if (data.status === 'fail') {
				throw new Error(data.message || 'Location service error');
			}

			const locationData: LocationData = {
				city: data.city || '',
				region: data.region || '',
				country: data.country || '',
				countryCode: data.countryCode || '',
				timezone: data.timezone || '',
				lat: data.lat || 0,
				lon: data.lon || 0
			};

			this.state.location = locationData;
			this.updateDisplayText(locationData);
			this.saveToCache(locationData);
		} catch (error: any) {
			console.error('Location detection failed:', error);
			this.state.error = error.message || 'Failed to detect location';
			this.state.displayText = 'Location unavailable';

			// Set a generic fallback
			this.state.location = null;
		} finally {
			this.state.isLoading = false;
		}
	}

	/**
	 * Update display text based on location data
	 */
	private updateDisplayText(location: LocationData) {
		// Format: "City, Country" or "Region, Country" or just "Country"
		if (location.city && location.country) {
			this.state.displayText = `${location.city}, ${location.country}`;
		} else if (location.region && location.country) {
			this.state.displayText = `${location.region}, ${location.country}`;
		} else if (location.country) {
			this.state.displayText = location.country;
		} else {
			this.state.displayText = 'Location detected';
		}
	}

	/**
	 * Save location to localStorage with timestamp
	 */
	private saveToCache(location: LocationData) {
		try {
			const cacheData = {
				location,
				timestamp: Date.now()
			};
			localStorage.setItem(CACHE_KEY, JSON.stringify(cacheData));
		} catch (error) {
			// Fail silently if localStorage is not available
			console.debug('Failed to cache location:', error);
		}
	}

	/**
	 * Load location from localStorage if not expired
	 */
	private loadFromCache(): LocationData | null {
		try {
			const cached = localStorage.getItem(CACHE_KEY);
			if (!cached) return null;

			const cacheData = JSON.parse(cached);
			const age = Date.now() - cacheData.timestamp;

			// Check if cache is still valid (within 24 hours)
			if (age < CACHE_DURATION) {
				return cacheData.location;
			} else {
				// Cache expired, remove it
				localStorage.removeItem(CACHE_KEY);
				return null;
			}
		} catch (error) {
			// Invalid cache data, remove it
			localStorage.removeItem(CACHE_KEY);
			return null;
		}
	}

	/**
	 * Clear cached location and refresh
	 */
	async refresh() {
		localStorage.removeItem(CACHE_KEY);
		this.state.location = null;
		await this.fetchLocation();
	}

	/**
	 * Clear location data
	 */
	clear() {
		localStorage.removeItem(CACHE_KEY);
		this.state.location = null;
		this.state.error = null;
		this.state.displayText = 'Location unavailable';
	}
}

// Export singleton instance
export const locationStore = new LocationStore();

// Export types
export type { LocationData };
