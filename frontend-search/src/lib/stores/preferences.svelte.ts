// Phase 8.6: User Preferences Store
// Manages user preferences (theme, results per page, analytics opt-out)

import axios from 'axios';

const API_URL = import.meta.env.VITE_API_URL || 'https://api.arack.io';

// User preferences interface matching backend
export interface UserPreferences {
	id: string;
	kratos_identity_id: string;
	theme: 'light' | 'dark';
	results_per_page: number;
	analytics_opt_out: boolean;
	created_at: string;
	updated_at: string;
}

// Update request interface
interface UpdatePreferencesRequest {
	theme?: 'light' | 'dark';
	results_per_page?: number;
	analytics_opt_out?: boolean;
}

class PreferencesStore {
	private state = $state<UserPreferences | null>(null);
	private loading = $state(false);
	private error = $state<string | null>(null);

	// Getters for reactive state
	get preferences() {
		return this.state;
	}

	get isLoading() {
		return this.loading;
	}

	get errorMessage() {
		return this.error;
	}

	// Convenience getters
	get theme(): 'light' | 'dark' {
		return this.state?.theme || 'light';
	}

	get resultsPerPage(): number {
		return this.state?.results_per_page || 20;
	}

	get analyticsOptOut(): boolean {
		return this.state?.analytics_opt_out || false;
	}

	/**
	 * Load user preferences from backend
	 * Creates default preferences if they don't exist
	 */
	async load() {
		this.loading = true;
		this.error = null;

		try {
			const response = await axios.get<{ data: UserPreferences }>(
				`${API_URL}/api/ory/preferences`,
				{ withCredentials: true }
			);

			this.state = response.data.data;
		} catch (error: any) {
			console.error('Failed to load preferences:', error);
			this.error = error.response?.data?.error || 'Failed to load preferences';
		} finally {
			this.loading = false;
		}
	}

	/**
	 * Update user preferences
	 * @param updates - Partial preferences to update
	 */
	async update(updates: UpdatePreferencesRequest) {
		this.loading = true;
		this.error = null;

		try {
			const response = await axios.post<{ data: UserPreferences }>(
				`${API_URL}/api/ory/preferences`,
				updates,
				{ withCredentials: true }
			);

			this.state = response.data.data;
			return true;
		} catch (error: any) {
			console.error('Failed to update preferences:', error);
			this.error = error.response?.data?.error || 'Failed to update preferences';
			return false;
		} finally {
			this.loading = false;
		}
	}

	/**
	 * Toggle theme between light and dark
	 */
	async toggleTheme() {
		const newTheme = this.theme === 'light' ? 'dark' : 'light';
		return await this.update({ theme: newTheme });
	}

	/**
	 * Update results per page
	 */
	async updateResultsPerPage(count: number) {
		if (count < 10 || count > 100) {
			this.error = 'Results per page must be between 10 and 100';
			return false;
		}
		return await this.update({ results_per_page: count });
	}

	/**
	 * Toggle analytics opt-out
	 */
	async toggleAnalyticsOptOut() {
		return await this.update({ analytics_opt_out: !this.analyticsOptOut });
	}

	/**
	 * Clear local preferences state
	 */
	clear() {
		this.state = null;
		this.error = null;
	}
}

// Export singleton instance
export const preferencesStore = new PreferencesStore();
