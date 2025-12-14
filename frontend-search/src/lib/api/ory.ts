// Ory SDK Client Configuration
// Direct Ory Kratos SDK client for flows that need the full SDK (verification, recovery, settings)

import { Configuration, FrontendApi } from '@ory/client';

const KRATOS_PUBLIC_URL = 'http://127.0.0.1:4433';

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
