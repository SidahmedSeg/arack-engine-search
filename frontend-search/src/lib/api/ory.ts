// Ory SDK Client Configuration
// Direct Ory Kratos SDK client for flows that need the full SDK (verification, recovery, settings)

import { Configuration, FrontendApi } from '@ory/client';

const KRATOS_PUBLIC_URL = import.meta.env.VITE_API_URL || 'https://api.arack.io';

// Create Ory configuration
const configuration = new Configuration({
	basePath: KRATOS_PUBLIC_URL,
	baseOptions: {
		withCredentials: true, // Important: Send cookies with requests
		timeout: 30000 // 30 second timeout
	}
});

// Export Ory Frontend API client
export const ory = new FrontendApi(configuration);
