# Ory Kratos Migration Guide - Search Users

This guide provides step-by-step instructions to migrate from custom authentication to Ory Kratos for **search engine users** (regular users who register/login to use the search).

**Timeline**: 3-5 days
**Risk Level**: Medium (well-tested migration path)
**Rollback**: Fully supported

---

## Table of Contents

1. [Pre-Migration Checklist](#pre-migration-checklist)
2. [Phase 1: Kratos Configuration](#phase-1-kratos-configuration)
3. [Phase 2: User Data Migration](#phase-2-user-data-migration)
4. [Phase 3: Backend Integration](#phase-3-backend-integration)
5. [Phase 4: Frontend Integration](#phase-4-frontend-integration)
6. [Phase 5: Testing](#phase-5-testing)
7. [Phase 6: Deployment](#phase-6-deployment)
8. [Rollback Procedure](#rollback-procedure)

---

## Pre-Migration Checklist

### ✅ Prerequisites

- [ ] Kratos container running (`docker ps | grep kratos`)
- [ ] Database `kratos_db` exists and is healthy
- [ ] Backup of `engine_search.users` table created
- [ ] Backup of `engine_search.sessions` table created
- [ ] All dependencies installed
- [ ] Test environment ready

### Backup Current Data

```bash
# Backup users table
pg_dump -h localhost -p 5434 -U postgres -t users engine_search > backup_users_$(date +%Y%m%d).sql

# Backup sessions table
pg_dump -h localhost -p 5434 -U postgres -t sessions engine_search > backup_sessions_$(date +%Y%m%d).sql

# Backup entire database (optional but recommended)
pg_dump -h localhost -p 5434 -U postgres engine_search > backup_full_$(date +%Y%m%d).sql
```

---

## Phase 1: Kratos Configuration

### Step 1.1: Update Kratos Configuration

The configuration is already set up correctly at `ory/kratos/kratos.yml`. Let's verify key settings:

```bash
# Verify Kratos is running
curl -s http://127.0.0.1:4433/health/ready | jq .

# Expected: {"status":"ok"}
```

### Step 1.2: Update Frontend URLs (if needed)

Edit `ory/kratos/kratos.yml` if your frontend port is different:

```yaml
selfservice:
  default_browser_return_url: http://127.0.0.1:5001/

  flows:
    login:
      ui_url: http://127.0.0.1:5001/auth/login
      after:
        password:
          default_browser_return_url: http://127.0.0.1:5001/

    registration:
      ui_url: http://127.0.0.1:5001/auth/signup
      after:
        password:
          default_browser_return_url: http://127.0.0.1:5001/
```

### Step 1.3: Restart Kratos (if config changed)

```bash
docker-compose restart kratos
```

---

## Phase 2: User Data Migration

### Step 2.1: Create Migration Script

Create `scripts/migrate_users_to_kratos.sh`:

```bash
#!/bin/bash

# Migrate existing users from custom auth to Kratos
# This script reads users from engine_search.users and creates identities in Kratos

set -e

DB_HOST="localhost"
DB_PORT="5434"
DB_NAME="engine_search"
DB_USER="postgres"
DB_PASS="postgres"
KRATOS_ADMIN_URL="http://127.0.0.1:4434"

echo "Starting user migration to Kratos..."

# Get all active users (exclude admins - they use custom auth)
psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -t -A -F"," -c \
  "SELECT id, email, password_hash, first_name, last_name, created_at
   FROM users
   WHERE role = 'user' AND is_active = true
   ORDER BY created_at" | while IFS=',' read -r id email password_hash first_name last_name created_at; do

  echo "Migrating user: $email"

  # Create identity in Kratos
  # Note: Kratos uses bcrypt, we're using argon2id, so we'll need to handle this

  curl -s -X POST "$KRATOS_ADMIN_URL/admin/identities" \
    -H "Content-Type: application/json" \
    -d "{
      \"schema_id\": \"default\",
      \"traits\": {
        \"email\": \"$email\",
        \"first_name\": \"$first_name\",
        \"last_name\": \"$last_name\"
      },
      \"metadata_public\": {
        \"legacy_user_id\": \"$id\",
        \"migrated_from_custom_auth\": true,
        \"migrated_at\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"
      },
      \"state\": \"active\"
    }" > /dev/null

  if [ $? -eq 0 ]; then
    echo "✓ Migrated: $email"
  else
    echo "✗ Failed: $email"
  fi
done

echo "Migration complete!"
```

### Step 2.2: Handle Password Migration

**Important**: Kratos uses bcrypt by default, but we use argon2id. We have two options:

**Option A: Force Password Reset (Recommended - More Secure)**

Users will need to use "Forgot Password" on first login after migration.

**Option B: Configure Kratos to Support Argon2id**

Update `ory/kratos/kratos.yml`:

```yaml
hashers:
  algorithm: argon2  # Change from bcrypt to argon2
  argon2:
    memory: 131072  # 128 MB
    iterations: 3
    parallelism: 4
    salt_length: 16
    key_length: 32
```

Then restart Kratos:
```bash
docker-compose restart kratos
```

### Step 2.3: Enhanced Migration Script (with Password Import)

If using Option B, create `scripts/migrate_users_with_passwords.sh`:

```bash
#!/bin/bash

set -e

DB_HOST="localhost"
DB_PORT="5434"
DB_NAME="engine_search"
DB_USER="postgres"
KRATOS_ADMIN_URL="http://127.0.0.1:4434"

echo "Starting user migration with password preservation..."

# Export users to temp file
psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -t -A -F"|" -c \
  "SELECT id, email, password_hash, first_name, last_name
   FROM users
   WHERE role = 'user' AND is_active = true" > /tmp/users_to_migrate.txt

# Migrate each user
while IFS='|' read -r id email password_hash first_name last_name; do
  echo "Migrating: $email"

  # Create identity with imported password hash
  curl -s -X POST "$KRATOS_ADMIN_URL/admin/identities" \
    -H "Content-Type: application/json" \
    -d "{
      \"schema_id\": \"default\",
      \"traits\": {
        \"email\": \"$email\",
        \"first_name\": \"$first_name\",
        \"last_name\": \"$last_name\"
      },
      \"credentials\": {
        \"password\": {
          \"config\": {
            \"hashed_password\": \"$password_hash\"
          }
        }
      },
      \"metadata_public\": {
        \"legacy_user_id\": \"$id\",
        \"migrated_at\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"
      }
    }" | jq -r '.id' > /dev/null

  if [ $? -eq 0 ]; then
    echo "✓ $email migrated"
  else
    echo "✗ $email failed"
  fi
done < /tmp/users_to_migrate.txt

rm /tmp/users_to_migrate.txt
echo "Migration complete!"
```

### Step 2.4: Run Migration

```bash
# Make script executable
chmod +x scripts/migrate_users_with_passwords.sh

# Run migration (DRY RUN first - check output)
./scripts/migrate_users_with_passwords.sh

# Verify migrated users
curl -s http://127.0.0.1:4434/admin/identities | jq '.[] | {email: .traits.email, id: .id}'
```

---

## Phase 3: Backend Integration

### Step 3.1: Update API Routes

Edit `src/api/mod.rs`:

**Remove custom auth routes** (lines 167-173):

```rust
// REMOVE THESE LINES:
.route("/api/auth/register", post(register))
.route("/api/auth/login", post(login))
.route("/api/auth/logout", post(logout))
.route("/api/auth/me", get(current_user))
```

**Add Kratos proxy routes**:

```rust
// Add after line 165 (after search/autocomplete route)
// Kratos authentication routes (proxied)
.route("/api/auth/register/browser", get(init_registration))
.route("/api/auth/login/browser", get(init_login))
.route("/api/auth/logout/browser", get(init_logout))
.route("/api/auth/whoami", get(kratos_whoami))
```

### Step 3.2: Implement Kratos Proxy Handlers

Add at the end of `src/api/mod.rs` (before the closing brace):

```rust
// Kratos authentication proxy handlers

/// Initialize registration flow
async fn init_registration(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match state.kratos.init_registration_flow().await {
        Ok(flow) => {
            let response = ApiResponse::success(flow);
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to init registration flow: {}", e);
            let response = ApiResponse::error("Failed to initialize registration".to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

/// Initialize login flow
async fn init_login(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match state.kratos.init_login_flow().await {
        Ok(flow) => {
            let response = ApiResponse::success(flow);
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to init login flow: {}", e);
            let response = ApiResponse::error("Failed to initialize login".to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

/// Initialize logout flow
async fn init_logout(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let cookie_header = headers.get("cookie").and_then(|h| h.to_str().ok());

    match state.kratos.init_logout_flow(cookie_header).await {
        Ok(logout_url) => {
            let response = ApiResponse::success(serde_json::json!({
                "logout_url": logout_url
            }));
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to init logout flow: {}", e);
            let response = ApiResponse::error("Failed to initialize logout".to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

/// Get current user from Kratos session (whoami)
async fn kratos_whoami(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let cookie_header = match headers.get("cookie").and_then(|h| h.to_str().ok()) {
        Some(c) => c,
        None => {
            let response = ApiResponse::error("Not authenticated".to_string());
            return (StatusCode::UNAUTHORIZED, Json(response)).into_response();
        }
    };

    match state.kratos.whoami(cookie_header).await {
        Ok(session) => {
            if session.active {
                let response = ApiResponse::success(serde_json::json!({
                    "id": session.identity.id,
                    "email": session.identity.traits.email,
                    "first_name": session.identity.traits.first_name,
                    "last_name": session.identity.traits.last_name,
                    "authenticated": true
                }));
                (StatusCode::OK, Json(response)).into_response()
            } else {
                let response = ApiResponse::error("Session expired".to_string());
                (StatusCode::UNAUTHORIZED, Json(response)).into_response()
            }
        }
        Err(e) => {
            error!("Whoami failed: {}", e);
            let response = ApiResponse::error("Not authenticated".to_string());
            (StatusCode::UNAUTHORIZED, Json(response)).into_response()
        }
    }
}
```

### Step 3.3: Add Kratos Client Methods

Edit `src/ory/kratos.rs` and add these methods:

```rust
impl KratosClient {
    // ... existing methods ...

    /// Initialize registration flow
    pub async fn init_registration_flow(&self) -> Result<serde_json::Value> {
        let url = format!("{}/self-service/registration/api", self.public_url);

        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to initialize registration flow")?;

        if !response.status().is_success() {
            anyhow::bail!("Registration flow init failed: {}", response.status());
        }

        let flow = response.json().await
            .context("Failed to parse registration flow")?;

        Ok(flow)
    }

    /// Initialize login flow
    pub async fn init_login_flow(&self) -> Result<serde_json::Value> {
        let url = format!("{}/self-service/login/api", self.public_url);

        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to initialize login flow")?;

        if !response.status().is_success() {
            anyhow::bail!("Login flow init failed: {}", response.status());
        }

        let flow = response.json().await
            .context("Failed to parse login flow")?;

        Ok(flow)
    }

    /// Initialize logout flow
    pub async fn init_logout_flow(&self, cookie: Option<&str>) -> Result<String> {
        let url = format!("{}/self-service/logout/browser", self.public_url);

        let mut request = self.client.get(&url);

        if let Some(cookie_header) = cookie {
            request = request.header("Cookie", cookie_header);
        }

        let response = request
            .send()
            .await
            .context("Failed to initialize logout flow")?;

        if !response.status().is_success() {
            anyhow::bail!("Logout flow init failed: {}", response.status());
        }

        let logout_data: serde_json::Value = response.json().await
            .context("Failed to parse logout flow")?;

        let logout_url = logout_data["logout_url"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No logout_url in response"))?
            .to_string();

        Ok(logout_url)
    }
}
```

### Step 3.4: Rebuild Backend

```bash
cargo build --release
```

---

## Phase 4: Frontend Integration

### Step 4.1: Create Kratos Helper Library

Create `frontend-search/src/lib/api/kratos.ts`:

```typescript
import axios from 'axios';

const KRATOS_PUBLIC_URL = 'http://127.0.0.1:4433';
const BACKEND_API_URL = 'http://127.0.0.1:3000';

export interface KratosFlow {
	id: string;
	type: string;
	ui: {
		action: string;
		method: string;
		nodes: any[];
	};
}

export interface KratosSession {
	id: string;
	active: boolean;
	identity: {
		id: string;
		traits: {
			email: string;
			first_name: string;
			last_name: string;
		};
	};
}

/**
 * Initialize registration flow
 */
export async function initRegistrationFlow(): Promise<KratosFlow> {
	const response = await axios.get(`${KRATOS_PUBLIC_URL}/self-service/registration/api`, {
		withCredentials: true
	});
	return response.data;
}

/**
 * Submit registration data
 */
export async function submitRegistration(
	flowId: string,
	email: string,
	password: string,
	firstName: string,
	lastName: string
): Promise<KratosSession> {
	const response = await axios.post(
		`${KRATOS_PUBLIC_URL}/self-service/registration?flow=${flowId}`,
		{
			method: 'password',
			traits: {
				email,
				first_name: firstName,
				last_name: lastName
			},
			password
		},
		{
			withCredentials: true
		}
	);
	return response.data.session;
}

/**
 * Initialize login flow
 */
export async function initLoginFlow(): Promise<KratosFlow> {
	const response = await axios.get(`${KRATOS_PUBLIC_URL}/self-service/login/api`, {
		withCredentials: true
	});
	return response.data;
}

/**
 * Submit login credentials
 */
export async function submitLogin(
	flowId: string,
	email: string,
	password: string
): Promise<KratosSession> {
	const response = await axios.post(
		`${KRATOS_PUBLIC_URL}/self-service/login?flow=${flowId}`,
		{
			method: 'password',
			identifier: email,
			password
		},
		{
			withCredentials: true
		}
	);
	return response.data.session;
}

/**
 * Get current session (whoami)
 */
export async function whoami(): Promise<KratosSession | null> {
	try {
		const response = await axios.get(`${KRATOS_PUBLIC_URL}/sessions/whoami`, {
			withCredentials: true
		});
		return response.data;
	} catch (error) {
		return null;
	}
}

/**
 * Logout current session
 */
export async function logout(): Promise<void> {
	const response = await axios.get(`${KRATOS_PUBLIC_URL}/self-service/logout/browser`, {
		withCredentials: true
	});

	// Call the logout URL
	if (response.data?.logout_url) {
		await axios.get(response.data.logout_url, {
			withCredentials: true
		});
	}
}
```

### Step 4.2: Update Auth Store

Edit `frontend-search/src/lib/stores/auth.svelte.ts`:

```typescript
import { whoami, logout as kratosLogout } from '$lib/api/kratos';
import type { KratosSession } from '$lib/api/kratos';

interface User {
	id: string;
	email: string;
	firstName: string;
	lastName: string;
}

class AuthStore {
	user: User | null = $state(null);
	isAuthenticated = $derived(this.user !== null);
	isLoading = $state(true);

	constructor() {
		// Check authentication on init
		this.checkAuth();
	}

	async checkAuth() {
		this.isLoading = true;
		try {
			const session = await whoami();
			if (session && session.active) {
				this.user = {
					id: session.identity.id,
					email: session.identity.traits.email,
					firstName: session.identity.traits.first_name,
					lastName: session.identity.traits.last_name
				};
			} else {
				this.user = null;
			}
		} catch (error) {
			console.error('Auth check failed:', error);
			this.user = null;
		} finally {
			this.isLoading = false;
		}
	}

	async logout() {
		try {
			await kratosLogout();
			this.user = null;
			window.location.href = '/';
		} catch (error) {
			console.error('Logout failed:', error);
		}
	}

	setUser(session: KratosSession) {
		this.user = {
			id: session.identity.id,
			email: session.identity.traits.email,
			firstName: session.identity.traits.first_name,
			lastName: session.identity.traits.last_name
		};
	}
}

export const authStore = new AuthStore();
```

### Step 4.3: Update Login Page

Edit `frontend-search/src/routes/auth/login/+page.svelte`:

```svelte
<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { initLoginFlow, submitLogin } from '$lib/api/kratos';
	import { authStore } from '$lib/stores/auth.svelte';
	import Button from '$lib/components/ui/button/button.svelte';
	import Input from '$lib/components/ui/input/input.svelte';

	let email = $state('');
	let password = $state('');
	let loading = $state(false);
	let error = $state('');
	let flowId = $state('');

	onMount(async () => {
		// Initialize login flow
		try {
			const flow = await initLoginFlow();
			flowId = flow.id;
		} catch (err: any) {
			error = 'Failed to initialize login';
			console.error(err);
		}
	});

	async function handleLogin(e: Event) {
		e.preventDefault();
		if (!flowId) {
			error = 'Login flow not initialized';
			return;
		}

		loading = true;
		error = '';

		try {
			const session = await submitLogin(flowId, email, password);
			authStore.setUser(session);
			goto('/');
		} catch (err: any) {
			console.error('Login error:', err);
			error = err.response?.data?.error?.message || 'Invalid email or password';
			// Re-initialize flow on error
			const flow = await initLoginFlow();
			flowId = flow.id;
		} finally {
			loading = false;
		}
	}
</script>

<div class="min-h-screen flex items-center justify-center bg-gray-50">
	<div class="max-w-md w-full space-y-8 p-8 bg-white rounded-lg shadow">
		<div>
			<h2 class="text-3xl font-bold text-center">Sign in to 2arak</h2>
		</div>

		{#if error}
			<div class="bg-red-50 border border-red-200 text-red-800 px-4 py-3 rounded">
				{error}
			</div>
		{/if}

		<form onsubmit={handleLogin} class="space-y-6">
			<Input
				label="Email"
				type="email"
				bind:value={email}
				required
				placeholder="you@example.com"
			/>

			<Input
				label="Password"
				type="password"
				bind:value={password}
				required
				placeholder="Enter your password"
			/>

			<Button type="submit" disabled={loading} class="w-full">
				{loading ? 'Signing in...' : 'Sign in'}
			</Button>
		</form>

		<div class="text-center text-sm">
			<a href="/auth/signup" class="text-blue-600 hover:underline">
				Don't have an account? Sign up
			</a>
		</div>
	</div>
</div>
```

### Step 4.4: Update Register Page

Edit `frontend-search/src/routes/auth/signup/+page.svelte`:

```svelte
<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { initRegistrationFlow, submitRegistration } from '$lib/api/kratos';
	import { authStore } from '$lib/stores/auth.svelte';
	import Button from '$lib/components/ui/button/button.svelte';
	import Input from '$lib/components/ui/input/input.svelte';

	let email = $state('');
	let password = $state('');
	let firstName = $state('');
	let lastName = $state('');
	let loading = $state(false);
	let error = $state('');
	let flowId = $state('');

	onMount(async () => {
		try {
			const flow = await initRegistrationFlow();
			flowId = flow.id;
		} catch (err: any) {
			error = 'Failed to initialize registration';
			console.error(err);
		}
	});

	async function handleRegister(e: Event) {
		e.preventDefault();
		if (!flowId) {
			error = 'Registration flow not initialized';
			return;
		}

		loading = true;
		error = '';

		try {
			const session = await submitRegistration(flowId, email, password, firstName, lastName);
			authStore.setUser(session);
			goto('/');
		} catch (err: any) {
			console.error('Registration error:', err);
			error = err.response?.data?.error?.message || 'Registration failed';
			// Re-initialize flow on error
			const flow = await initRegistrationFlow();
			flowId = flow.id;
		} finally {
			loading = false;
		}
	}
</script>

<div class="min-h-screen flex items-center justify-center bg-gray-50">
	<div class="max-w-md w-full space-y-8 p-8 bg-white rounded-lg shadow">
		<div>
			<h2 class="text-3xl font-bold text-center">Create your account</h2>
		</div>

		{#if error}
			<div class="bg-red-50 border border-red-200 text-red-800 px-4 py-3 rounded">
				{error}
			</div>
		{/if}

		<form onsubmit={handleRegister} class="space-y-6">
			<Input
				label="First Name"
				type="text"
				bind:value={firstName}
				required
				placeholder="John"
			/>

			<Input
				label="Last Name"
				type="text"
				bind:value={lastName}
				required
				placeholder="Doe"
			/>

			<Input
				label="Email"
				type="email"
				bind:value={email}
				required
				placeholder="you@example.com"
			/>

			<Input
				label="Password"
				type="password"
				bind:value={password}
				required
				placeholder="At least 8 characters"
			/>

			<Button type="submit" disabled={loading} class="w-full">
				{loading ? 'Creating account...' : 'Create account'}
			</Button>
		</form>

		<div class="text-center text-sm">
			<a href="/auth/login" class="text-blue-600 hover:underline">
				Already have an account? Sign in
			</a>
		</div>
	</div>
</div>
```

---

## Phase 5: Testing

### Step 5.1: Test Registration

```bash
# Open browser
open http://localhost:5001/auth/signup

# Fill form and submit
# Expected: User created, auto-logged in, redirected to home
```

Verify in Kratos:
```bash
curl -s http://127.0.0.1:4434/admin/identities | jq '.[] | {email: .traits.email, id: .id}'
```

### Step 5.2: Test Login

```bash
# Open browser
open http://localhost:5001/auth/login

# Use credentials from Step 5.1
# Expected: Logged in, redirected to home
```

### Step 5.3: Test Session Validation

```bash
# While logged in, check whoami
curl -s http://127.0.0.1:4433/sessions/whoami --cookie-jar cookies.txt | jq .

# Expected: Active session with user info
```

### Step 5.4: Test Logout

```bash
# Click logout button
# Expected: Redirected to home, session cleared
```

### Step 5.5: Test Migrated Users

```bash
# Login with an old user account (from before migration)
# Expected: Login works with original password
```

---

## Phase 6: Deployment

### Step 6.1: Update Production Environment

Update production `ory/kratos/kratos.yml`:

```yaml
# Change all localhost URLs to production URLs
selfservice:
  default_browser_return_url: https://search.yoursite.com/

  flows:
    login:
      ui_url: https://search.yoursite.com/auth/login
    registration:
      ui_url: https://search.yoursite.com/auth/signup

cookies:
  domain: yoursite.com  # Change from 127.0.0.1
  same_site: Lax
  secure: true  # IMPORTANT: Set to true in production (HTTPS)

secrets:
  cookie:
    - GENERATE-LONG-RANDOM-SECRET-HERE  # Change from dev secret
  cipher:
    - GENERATE-32-CHAR-SECRET-HERE
```

### Step 6.2: Update Frontend Environment

Update frontend `.env`:
```bash
PUBLIC_KRATOS_URL=https://kratos.yoursite.com
```

### Step 6.3: Deploy

```bash
# Build backend
cargo build --release

# Build frontend
cd frontend-search && npm run build

# Deploy with docker-compose
docker-compose up -d --build
```

---

## Rollback Procedure

If anything goes wrong, you can rollback:

### Step 1: Stop Kratos Integration

In `src/api/mod.rs`, comment out Kratos routes:
```rust
// .route("/api/auth/register/browser", get(init_registration))
// .route("/api/auth/login/browser", get(init_login))
```

Restore custom auth routes:
```rust
.route("/api/auth/register", post(register))
.route("/api/auth/login", post(login))
```

### Step 2: Restore Frontend Pages

```bash
git checkout frontend-search/src/routes/auth/login/+page.svelte
git checkout frontend-search/src/routes/auth/signup/+page.svelte
git checkout frontend-search/src/lib/stores/auth.svelte.ts
```

### Step 3: Rebuild and Restart

```bash
cargo build --release
cd frontend-search && npm run build
```

### Step 4: Restore Database (if needed)

```bash
psql -h localhost -p 5434 -U postgres -d engine_search < backup_users_YYYYMMDD.sql
```

---

## Troubleshooting

### Issue: "Flow not found" error

**Solution**: Flow IDs expire after 10 minutes. Re-initialize the flow:
```typescript
const flow = await initLoginFlow();
flowId = flow.id;
```

### Issue: CORS errors

**Solution**: Verify Kratos CORS config in `kratos.yml`:
```yaml
serve:
  public:
    cors:
      allowed_origins:
        - http://localhost:5001
      allow_credentials: true
```

### Issue: Users can't login after migration

**Solution**: Check password hash algorithm matches:
```bash
# Verify Kratos hasher
curl -s http://127.0.0.1:4433/.well-known/ory/meta | jq .
```

### Issue: Session not persisting

**Solution**: Check cookies are being set:
```bash
# In browser DevTools > Application > Cookies
# Should see: ory_kratos_session
```

---

## Next Steps

After successful migration:

1. ✅ Enable email verification (already configured)
2. ✅ Enable password recovery (already configured)
3. 🔄 Configure Hydra for SSO to email/calendar (separate guide)
4. 🔄 Add social login (Google, GitHub)
5. 🔄 Add 2FA/MFA
6. 🔄 Set up monitoring and alerts

---

## Support

- **Kratos Docs**: https://www.ory.sh/docs/kratos
- **Community**: https://slack.ory.sh
- **Issues**: Create issue in project repo
