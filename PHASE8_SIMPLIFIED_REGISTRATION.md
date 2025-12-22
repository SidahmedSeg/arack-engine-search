# Phase 8: Simplified Email Provider Registration (Keep Kratos)

## Executive Summary

**Keep Ory Kratos** - It's already integrated and handles auth flows perfectly.

**Add Username Selection** - Users choose `username@arack.io` during registration.

**No Phone Verification** - Use email verification to the newly created @arack.io address.

---

## Registration Flow Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                     REGISTRATION JOURNEY                        │
└─────────────────────────────────────────────────────────────────┘

Step 1: Personal Information
┌────────────────────────────┐
│ Create your account        │
│ ┌──────────────────────┐   │
│ │ First Name           │   │
│ └──────────────────────┘   │
│ ┌──────────────────────┐   │
│ │ Last Name            │   │
│ └──────────────────────┘   │
│ ┌──────────────────────┐   │
│ │ Date of Birth        │   │  ← DD/MM/YYYY picker
│ └──────────────────────┘   │
│ ⚪ Male  ⚪ Female       │
└────────────────────────────┘

Step 2: Choose Email Address
┌────────────────────────────┐
│ Choose your email address  │
│                            │
│ Suggestions:               │
│ ✓ john.doe@arack.io       │  ← Based on first.last
│ ✓ johndoe@arack.io        │  ← Based on firstname+lastname
│                            │
│ Or create custom:          │
│ ┌──────────────────────┐   │
│ │ custom_username      │   │  ← Real-time availability check
│ └──────────────────────┘   │
│ @arack.io                  │
│                            │
│ ✓ Available!               │
└────────────────────────────┘

Step 3: Create Password
┌────────────────────────────┐
│ Create your password       │
│ ┌──────────────────────┐   │
│ │ Password (min 8)     │   │
│ └──────────────────────┘   │
│ ┌──────────────────────┐   │
│ │ Confirm Password     │   │
│ └──────────────────────┘   │
│                            │
│ Password strength: 🟩🟩🟩⬜ │
└────────────────────────────┘
         ↓
✅ Registration Submitted to Kratos
         ↓
🔄 Kratos Webhook Triggers:
   - Creates user in auth.users
   - Creates email account in Stalwart
   - Sends verification email to username@arack.io
         ↓
📧 User logs into webmail (mail.arack.io)
         ↓
✅ Clicks verification link
         ↓
✅ Account Activated!
```

---

## Why Keep Ory Kratos?

### ✅ Advantages:

1. **Already Integrated** - Flows, sessions, cookies all working
2. **Battle-Tested** - Production-grade security (OWASP compliant)
3. **Self-Service Flows** - Registration, login, recovery, verification
4. **Flexible Identity Schema** - Can add custom traits (username, DOB, gender)
5. **Webhook System** - Already triggers email provisioning
6. **Session Management** - Handles cookies, CSRF, session expiry
7. **Future-Proof** - Easy to add OAuth, 2FA, passkeys later

### ❌ Why NOT Remove It:

- Would have to rebuild all auth flows from scratch
- Session management is complex (JWT alone isn't enough for web apps)
- CSRF protection, rate limiting, account recovery all built-in
- Already configured with PostgreSQL, migrations done

---

## Architecture Overview

```
┌──────────────────────────────────────────────────────────────┐
│                         FRONTEND                             │
│                                                              │
│  Registration Form → Kratos Registration Flow                │
│  - Username availability check (custom API)                  │
│  - Password strength indicator                               │
│  - Auto-fill username suggestions                            │
└──────────────────────────┬───────────────────────────────────┘
                           │
                           ▼
┌──────────────────────────────────────────────────────────────┐
│                      ORY KRATOS                              │
│                                                              │
│  1. Receives registration (username@arack.io, password)      │
│  2. Validates identity schema (first_name, last_name, etc)   │
│  3. Creates identity in identities table                     │
│  4. Triggers webhook → Search & Email Services               │
│  5. Creates verification flow                                │
│  6. Sends verification email via courier                     │
└──────────────────────────┬───────────────────────────────────┘
                           │
              ┌────────────┴─────────────┐
              ▼                          ▼
┌──────────────────────┐    ┌──────────────────────┐
│  SEARCH SERVICE      │    │  EMAIL SERVICE       │
│  (Port 3000)         │    │  (Port 3001)         │
│                      │    │                      │
│  Webhook:            │    │  Webhook:            │
│  /internal/auth/     │    │  /internal/mail/     │
│   user-created       │    │   provision          │
│                      │    │                      │
│  Creates:            │    │  Creates:            │
│  - auth.users        │    │  - Stalwart account  │
│  - preferences       │    │  - email.accounts    │
│                      │    │  - Default mailboxes │
└──────────────────────┘    └──────────────────────┘
```

---

## Database Schema Updates

### 1. Update Kratos Identity Schema

**File**: `/opt/arack/ory/kratos/identity.schema.json`

```json
{
  "$id": "https://arack.io/schemas/identity.schema.json",
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Arack User",
  "type": "object",
  "properties": {
    "traits": {
      "type": "object",
      "properties": {
        "email": {
          "type": "string",
          "format": "email",
          "title": "Email Address",
          "minLength": 3,
          "maxLength": 320,
          "ory.sh/kratos": {
            "credentials": {
              "password": {
                "identifier": true
              }
            },
            "verification": {
              "via": "email"
            },
            "recovery": {
              "via": "email"
            }
          }
        },
        "username": {
          "type": "string",
          "title": "Username",
          "minLength": 3,
          "maxLength": 30,
          "pattern": "^[a-z0-9._]+$",
          "description": "Username part of email (before @arack.io)"
        },
        "first_name": {
          "type": "string",
          "title": "First Name",
          "minLength": 1,
          "maxLength": 100
        },
        "last_name": {
          "type": "string",
          "title": "Last Name",
          "minLength": 1,
          "maxLength": 100
        },
        "date_of_birth": {
          "type": "string",
          "format": "date",
          "title": "Date of Birth",
          "description": "Format: YYYY-MM-DD"
        },
        "gender": {
          "type": "string",
          "enum": ["male", "female"],
          "title": "Gender"
        }
      },
      "required": [
        "email",
        "username",
        "first_name",
        "last_name",
        "date_of_birth",
        "gender"
      ],
      "additionalProperties": false
    }
  }
}
```

### 2. Update auth.users Table

**Migration**: `migrations/009_add_user_fields.sql`

```sql
-- Add new columns to auth.users
ALTER TABLE auth.users
  ADD COLUMN IF NOT EXISTS username VARCHAR(50) UNIQUE,
  ADD COLUMN IF NOT EXISTS date_of_birth DATE,
  ADD COLUMN IF NOT EXISTS gender VARCHAR(10) CHECK (gender IN ('male', 'female'));

-- Create index for username lookups (case-insensitive)
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_username_lower
  ON auth.users (LOWER(username));

-- Update existing users (set username from email if null)
UPDATE auth.users
SET username = SPLIT_PART(email, '@', 1)
WHERE username IS NULL;

-- Make username NOT NULL after backfill
ALTER TABLE auth.users
  ALTER COLUMN username SET NOT NULL;

-- Create table for username availability cache (performance optimization)
CREATE TABLE IF NOT EXISTS auth.username_availability_cache (
    username VARCHAR(50) PRIMARY KEY,
    available BOOLEAN NOT NULL,
    checked_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    -- Auto-expire cache after 5 minutes
    INDEX idx_username_cache_expires (checked_at)
);

-- Function to check username availability
CREATE OR REPLACE FUNCTION check_username_available(check_username TEXT)
RETURNS BOOLEAN AS $$
BEGIN
    RETURN NOT EXISTS (
        SELECT 1 FROM auth.users WHERE LOWER(username) = LOWER(check_username)
    ) AND NOT EXISTS (
        SELECT 1 FROM email.email_accounts WHERE LOWER(email_address) = LOWER(check_username || '@arack.io')
    );
END;
$$ LANGUAGE plpgsql;
```

---

## Backend Implementation

### 1. Username Availability API

**File**: `search/api/username.rs` (new)

```rust
use axum::{
    extract::{State, Json, Query},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use validator::Validate;
use regex::Regex;

lazy_static! {
    static ref USERNAME_REGEX: Regex = Regex::new(r"^[a-z0-9._]+$").unwrap();
    static ref RESERVED_USERNAMES: Vec<&'static str> = vec![
        "admin", "administrator", "root", "system",
        "support", "help", "info", "contact",
        "noreply", "no-reply", "postmaster",
        "abuse", "security", "webmaster", "hostmaster",
        "mailer-daemon", "nobody", "www", "ftp",
    ];
}

// ============================================================================
// Check Username Availability
// ============================================================================

#[derive(Deserialize, Validate)]
pub struct CheckUsernameQuery {
    #[validate(
        length(min = 3, max = 30),
        custom = "validate_username_format"
    )]
    username: String,
}

fn validate_username_format(username: &str) -> Result<(), validator::ValidationError> {
    // Only lowercase letters, numbers, dots, underscores
    if !USERNAME_REGEX.is_match(username) {
        return Err(validator::ValidationError::new("invalid_format"));
    }

    // No consecutive dots
    if username.contains("..") {
        return Err(validator::ValidationError::new("consecutive_dots"));
    }

    // No leading/trailing dots or underscores
    if username.starts_with('.') || username.ends_with('.') ||
       username.starts_with('_') || username.ends_with('_') {
        return Err(validator::ValidationError::new("invalid_start_end"));
    }

    // Check reserved usernames
    if RESERVED_USERNAMES.contains(&username.to_lowercase().as_str()) {
        return Err(validator::ValidationError::new("reserved"));
    }

    Ok(())
}

#[derive(Serialize)]
pub struct CheckUsernameResponse {
    available: bool,
    email: String, // Full email address
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>, // Why it's not available
}

pub async fn check_username_availability(
    State(state): State<Arc<AppState>>,
    Query(query): Query<CheckUsernameQuery>,
) -> Result<Json<CheckUsernameResponse>, AppError> {
    query.validate()?;

    let username = query.username.to_lowercase();
    let email = format!("{}@arack.io", username);

    // Check cache first (5-minute TTL)
    let cached = sqlx::query!(
        r#"
        SELECT available FROM auth.username_availability_cache
        WHERE LOWER(username) = $1
          AND checked_at > NOW() - INTERVAL '5 minutes'
        "#,
        username
    )
    .fetch_optional(&state.db_pool)
    .await?;

    if let Some(cache) = cached {
        return Ok(Json(CheckUsernameResponse {
            available: cache.available.unwrap_or(false),
            email: email.clone(),
            reason: if cache.available.unwrap_or(false) {
                None
            } else {
                Some("Username already taken".to_string())
            },
        }));
    }

    // Check database
    let available = sqlx::query!(
        "SELECT check_username_available($1) as available",
        username
    )
    .fetch_one(&state.db_pool)
    .await?
    .available
    .unwrap_or(false);

    // Update cache
    sqlx::query!(
        r#"
        INSERT INTO auth.username_availability_cache (username, available)
        VALUES ($1, $2)
        ON CONFLICT (username) DO UPDATE
        SET available = $2, checked_at = NOW()
        "#,
        username,
        available
    )
    .execute(&state.db_pool)
    .await?;

    Ok(Json(CheckUsernameResponse {
        available,
        email,
        reason: if available {
            None
        } else {
            Some("Username already taken".to_string())
        },
    }))
}

// ============================================================================
// Generate Username Suggestions
// ============================================================================

#[derive(Deserialize, Validate)]
pub struct SuggestUsernamesRequest {
    #[validate(length(min = 1, max = 100))]
    first_name: String,

    #[validate(length(min = 1, max = 100))]
    last_name: String,
}

#[derive(Serialize)]
pub struct SuggestUsernamesResponse {
    suggestions: Vec<UsernameSuggestion>,
}

#[derive(Serialize)]
pub struct UsernameSuggestion {
    username: String,
    email: String,
    available: bool,
}

pub async fn suggest_usernames(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SuggestUsernamesRequest>,
) -> Result<Json<SuggestUsernamesResponse>, AppError> {
    payload.validate()?;

    let first = normalize_for_username(&payload.first_name);
    let last = normalize_for_username(&payload.last_name);

    // Generate candidate usernames
    let candidates = vec![
        format!("{}.{}", first, last),           // john.doe
        format!("{}{}", first, last),            // johndoe
        format!("{}.{}", first.chars().next().unwrap_or('a'), last), // j.doe
        format!("{}{}", first, last.chars().next().unwrap_or('a')),  // johnd
        format!("{}_{}", first, last),           // john_doe
    ];

    let mut suggestions = Vec::new();

    for username in candidates {
        // Validate format
        if username.len() < 3 || username.len() > 30 {
            continue;
        }
        if !USERNAME_REGEX.is_match(&username) {
            continue;
        }

        let email = format!("{}@arack.io", username);

        // Check availability
        let available = sqlx::query!(
            "SELECT check_username_available($1) as available",
            username
        )
        .fetch_one(&state.db_pool)
        .await?
        .available
        .unwrap_or(false);

        suggestions.push(UsernameSuggestion {
            username,
            email,
            available,
        });

        // Return first 2 available suggestions
        if suggestions.iter().filter(|s| s.available).count() >= 2 {
            break;
        }
    }

    // If we don't have 2 available, generate numbered variations
    if suggestions.iter().filter(|s| s.available).count() < 2 {
        let base = format!("{}{}", first, last);
        for i in 1..=99 {
            let candidate = format!("{}{}", base, i);
            let email = format!("{}@arack.io", candidate);

            let available = sqlx::query!(
                "SELECT check_username_available($1) as available",
                candidate
            )
            .fetch_one(&state.db_pool)
            .await?
            .available
            .unwrap_or(false);

            if available {
                suggestions.push(UsernameSuggestion {
                    username: candidate,
                    email,
                    available,
                });

                if suggestions.iter().filter(|s| s.available).count() >= 2 {
                    break;
                }
            }
        }
    }

    Ok(Json(SuggestUsernamesResponse { suggestions }))
}

// Helper: Normalize name for username (remove accents, special chars)
fn normalize_for_username(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .filter_map(|c| {
            // Remove accents
            match c {
                'á' | 'à' | 'â' | 'ä' | 'ã' | 'å' => Some('a'),
                'é' | 'è' | 'ê' | 'ë' => Some('e'),
                'í' | 'ì' | 'î' | 'ï' => Some('i'),
                'ó' | 'ò' | 'ô' | 'ö' | 'õ' => Some('o'),
                'ú' | 'ù' | 'û' | 'ü' => Some('u'),
                'ñ' => Some('n'),
                'ç' => Some('c'),
                _ if c.is_ascii_lowercase() => Some(c),
                _ => None,
            }
        })
        .collect()
}
```

**Add routes to `search/api/mod.rs`:**

```rust
// Add to router
.route("/api/auth/check-username", get(username::check_username_availability))
.route("/api/auth/suggest-usernames", post(username::suggest_usernames))
```

### 2. Update Webhook Handlers

**Update `search/api/webhooks.rs`** to store additional fields:

```rust
#[derive(Deserialize)]
pub struct KratosWebhookPayload {
    identity: KratosIdentity,
}

#[derive(Deserialize)]
pub struct KratosIdentity {
    id: Uuid,
    traits: IdentityTraits,
}

#[derive(Deserialize)]
pub struct IdentityTraits {
    email: String,
    username: String,
    first_name: String,
    last_name: String,
    date_of_birth: String, // YYYY-MM-DD
    gender: String,
}

pub async fn handle_user_created(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<KratosWebhookPayload>,
) -> impl IntoResponse {
    let identity = &payload.identity;

    // Parse date of birth
    let dob = NaiveDate::parse_from_str(&identity.traits.date_of_birth, "%Y-%m-%d")
        .map_err(|_| AppError::BadRequest {
            message: "Invalid date of birth format".to_string(),
        })?;

    // Create user record with all fields
    sqlx::query!(
        r#"
        INSERT INTO auth.users
        (id, email, username, first_name, last_name, date_of_birth, gender, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
        ON CONFLICT (id) DO NOTHING
        "#,
        identity.id,
        identity.traits.email,
        identity.traits.username,
        identity.traits.first_name,
        identity.traits.last_name,
        dob,
        identity.traits.gender,
    )
    .execute(&state.db_pool)
    .await?;

    tracing::info!(
        "User created: {} ({}@arack.io)",
        identity.id,
        identity.traits.username
    );

    (StatusCode::OK, Json(json!({"success": true})))
}
```

**Update `email/provisioning/mod.rs`** similarly.

---

## Frontend Implementation

### 1. Registration Form Component

**File**: `frontend-search/src/routes/auth/register/+page.svelte`

```svelte
<script lang="ts">
import { goto } from '$app/navigation';
import { Button, Input, Label, Card } from '$lib/components/ui';
import { api } from '$lib/stores/api';
import { ory } from '$lib/api/ory';

// Step tracking
let currentStep = $state(1);
const totalSteps = 3;

// Step 1: Personal Information
let firstName = $state('');
let lastName = $state('');
let dateOfBirth = $state('');
let gender = $state('');

// Step 2: Username Selection
let selectedUsername = $state('');
let customUsername = $state('');
let suggestions = $state<Array<{username: string; email: string; available: boolean}>>([]);
let checkingCustom = $state(false);
let customAvailable = $state<boolean | null>(null);

// Step 3: Password
let password = $state('');
let confirmPassword = $state('');

// Password strength calculation
const passwordStrength = $derived(() => {
    if (password.length === 0) return 0;
    let strength = 0;
    if (password.length >= 8) strength++;
    if (password.length >= 12) strength++;
    if (/[a-z]/.test(password) && /[A-Z]/.test(password)) strength++;
    if (/\d/.test(password)) strength++;
    if (/[^a-zA-Z0-9]/.test(password)) strength++;
    return Math.min(4, strength);
});

// Form validation
const step1Valid = $derived(
    firstName.trim() !== '' &&
    lastName.trim() !== '' &&
    dateOfBirth !== '' &&
    gender !== ''
);

const step2Valid = $derived(
    selectedUsername !== '' ||
    (customUsername !== '' && customAvailable === true)
);

const step3Valid = $derived(
    password.length >= 8 &&
    password === confirmPassword
);

// Load username suggestions when moving to step 2
async function loadSuggestions() {
    if (firstName && lastName) {
        const result = await api.suggestUsernames({
            first_name: firstName,
            last_name: lastName
        });
        suggestions = result.suggestions;

        // Auto-select first available
        const firstAvailable = suggestions.find(s => s.available);
        if (firstAvailable) {
            selectedUsername = firstAvailable.username;
        }
    }
    currentStep = 2;
}

// Check custom username availability (debounced)
let debounceTimer: any;
function checkCustomUsername() {
    clearTimeout(debounceTimer);

    if (customUsername.length < 3) {
        customAvailable = null;
        return;
    }

    checkingCustom = true;

    debounceTimer = setTimeout(async () => {
        try {
            const result = await api.checkUsername({ username: customUsername });
            customAvailable = result.available;
        } catch (error) {
            customAvailable = false;
        } finally {
            checkingCustom = false;
        }
    }, 500);
}

$effect(() => {
    if (customUsername) {
        checkCustomUsername();
    }
});

// Submit registration to Kratos
async function submitRegistration() {
    try {
        const finalUsername = selectedUsername || customUsername;
        const email = `${finalUsername}@arack.io`;

        // Create registration flow via Kratos
        const { data: flow } = await ory.createBrowserRegistrationFlow();

        // Submit registration
        await ory.updateRegistrationFlow({
            flow: flow.id,
            updateRegistrationFlowBody: {
                method: 'password',
                password: password,
                traits: {
                    email: email,
                    username: finalUsername,
                    first_name: firstName,
                    last_name: lastName,
                    date_of_birth: dateOfBirth,
                    gender: gender
                }
            }
        });

        // Success - redirect to verification page
        goto('/auth/verify-email');
    } catch (error) {
        console.error('Registration failed:', error);
        alert('Registration failed. Please try again.');
    }
}
</script>

<div class="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4">
    <Card.Root class="w-full max-w-md">
        <Card.Header>
            <Card.Title>Create your Arack account</Card.Title>
            <Card.Description>
                Step {currentStep} of {totalSteps}
            </Card.Description>
        </Card.Header>

        <Card.Content>
            <!-- Progress Bar -->
            <div class="mb-6">
                <div class="flex gap-2">
                    {#each Array(totalSteps) as _, i}
                        <div
                            class="h-2 flex-1 rounded-full transition-colors"
                            class:bg-primary={i < currentStep}
                            class:bg-gray-200={i >= currentStep}
                        ></div>
                    {/each}
                </div>
            </div>

            {#if currentStep === 1}
                <!-- Step 1: Personal Information -->
                <div class="space-y-4">
                    <div class="grid grid-cols-2 gap-4">
                        <div>
                            <Label for="firstName">First Name</Label>
                            <Input
                                id="firstName"
                                bind:value={firstName}
                                placeholder="John"
                                required
                            />
                        </div>
                        <div>
                            <Label for="lastName">Last Name</Label>
                            <Input
                                id="lastName"
                                bind:value={lastName}
                                placeholder="Doe"
                                required
                            />
                        </div>
                    </div>

                    <div>
                        <Label for="dateOfBirth">Date of Birth</Label>
                        <Input
                            id="dateOfBirth"
                            type="date"
                            bind:value={dateOfBirth}
                            max={new Date().toISOString().split('T')[0]}
                            required
                        />
                    </div>

                    <div>
                        <Label>Gender</Label>
                        <div class="flex gap-4 mt-2">
                            <label class="flex items-center gap-2 cursor-pointer">
                                <input
                                    type="radio"
                                    bind:group={gender}
                                    value="male"
                                    class="w-4 h-4"
                                />
                                <span>Male</span>
                            </label>
                            <label class="flex items-center gap-2 cursor-pointer">
                                <input
                                    type="radio"
                                    bind:group={gender}
                                    value="female"
                                    class="w-4 h-4"
                                />
                                <span>Female</span>
                            </label>
                        </div>
                    </div>

                    <Button
                        onclick={loadSuggestions}
                        disabled={!step1Valid}
                        class="w-full"
                    >
                        Continue
                    </Button>
                </div>

            {:else if currentStep === 2}
                <!-- Step 2: Username Selection -->
                <div class="space-y-4">
                    <div>
                        <Label>Suggested email addresses</Label>
                        <div class="space-y-2 mt-2">
                            {#each suggestions.filter(s => s.available).slice(0, 2) as suggestion}
                                <label
                                    class="flex items-center gap-3 p-3 border rounded-lg cursor-pointer hover:bg-gray-50 transition-colors"
                                    class:border-primary={selectedUsername === suggestion.username}
                                    class:bg-primary/5={selectedUsername === suggestion.username}
                                >
                                    <input
                                        type="radio"
                                        bind:group={selectedUsername}
                                        value={suggestion.username}
                                        class="w-4 h-4"
                                    />
                                    <div class="flex-1">
                                        <div class="font-medium">{suggestion.email}</div>
                                        {#if selectedUsername === suggestion.username}
                                            <div class="text-xs text-green-600">✓ Selected</div>
                                        {/if}
                                    </div>
                                </label>
                            {/each}
                        </div>
                    </div>

                    <div class="relative">
                        <Label>Or create custom username</Label>
                        <div class="flex items-center gap-2 mt-2">
                            <div class="relative flex-1">
                                <Input
                                    bind:value={customUsername}
                                    placeholder="custom_username"
                                    oninput={() => {
                                        selectedUsername = '';
                                        checkCustomUsername();
                                    }}
                                />
                                <span class="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground">
                                    @arack.io
                                </span>
                            </div>
                        </div>
                        {#if checkingCustom}
                            <p class="text-sm text-muted-foreground mt-1">
                                Checking availability...
                            </p>
                        {:else if customUsername.length >= 3}
                            {#if customAvailable}
                                <p class="text-sm text-green-600 mt-1">
                                    ✓ Available!
                                </p>
                            {:else}
                                <p class="text-sm text-red-600 mt-1">
                                    ✗ Already taken
                                </p>
                            {/if}
                        {/if}
                    </div>

                    <div class="flex gap-2">
                        <Button
                            variant="outline"
                            onclick={() => currentStep = 1}
                            class="flex-1"
                        >
                            Back
                        </Button>
                        <Button
                            onclick={() => currentStep = 3}
                            disabled={!step2Valid}
                            class="flex-1"
                        >
                            Continue
                        </Button>
                    </div>
                </div>

            {:else if currentStep === 3}
                <!-- Step 3: Password -->
                <div class="space-y-4">
                    <div>
                        <Label for="password">Password</Label>
                        <Input
                            id="password"
                            type="password"
                            bind:value={password}
                            placeholder="Minimum 8 characters"
                            required
                        />
                        {#if password.length > 0}
                            <div class="mt-2 flex gap-1">
                                {#each Array(4) as _, i}
                                    <div
                                        class="h-2 flex-1 rounded-full transition-colors"
                                        class:bg-red-500={passwordStrength === 1 && i === 0}
                                        class:bg-orange-500={passwordStrength === 2 && i <= 1}
                                        class:bg-yellow-500={passwordStrength === 3 && i <= 2}
                                        class:bg-green-500={passwordStrength === 4 && i <= 3}
                                        class:bg-gray-200={i >= passwordStrength}
                                    ></div>
                                {/each}
                            </div>
                            <p class="text-xs text-muted-foreground mt-1">
                                {#if passwordStrength === 1}Weak
                                {:else if passwordStrength === 2}Fair
                                {:else if passwordStrength === 3}Good
                                {:else if passwordStrength === 4}Strong
                                {/if}
                            </p>
                        {/if}
                    </div>

                    <div>
                        <Label for="confirmPassword">Confirm Password</Label>
                        <Input
                            id="confirmPassword"
                            type="password"
                            bind:value={confirmPassword}
                            placeholder="Re-enter password"
                            required
                        />
                        {#if confirmPassword.length > 0}
                            {#if password === confirmPassword}
                                <p class="text-sm text-green-600 mt-1">✓ Passwords match</p>
                            {:else}
                                <p class="text-sm text-red-600 mt-1">✗ Passwords don't match</p>
                            {/if}
                        {/if}
                    </div>

                    <div class="flex gap-2">
                        <Button
                            variant="outline"
                            onclick={() => currentStep = 2}
                            class="flex-1"
                        >
                            Back
                        </Button>
                        <Button
                            onclick={submitRegistration}
                            disabled={!step3Valid}
                            class="flex-1"
                        >
                            Create Account
                        </Button>
                    </div>
                </div>
            {/if}
        </Card.Content>

        <Card.Footer>
            <p class="text-sm text-muted-foreground text-center w-full">
                Already have an account?
                <a href="/auth/login" class="text-primary hover:underline">
                    Sign in
                </a>
            </p>
        </Card.Footer>
    </Card.Root>
</div>
```

### 2. API Client Updates

**File**: `shared/api-client/index.ts`

```typescript
export class SearchEngineAPI {
    // ... existing methods

    async checkUsername(data: { username: string }): Promise<{
        available: boolean;
        email: string;
        reason?: string;
    }> {
        const response = await this.client.get('/api/auth/check-username', {
            params: data
        });
        return response.data;
    }

    async suggestUsernames(data: {
        first_name: string;
        last_name: string;
    }): Promise<{
        suggestions: Array<{
            username: string;
            email: string;
            available: boolean;
        }>;
    }> {
        const response = await this.client.post('/api/auth/suggest-usernames', data);
        return response.data;
    }
}
```

---

## Email Verification Flow

### The Challenge

When user registers with `username@arack.io`, the email account needs to exist BEFORE Kratos sends verification email.

### Solution: Two-Phase Provisioning

**Phase 1: Create Email Account Immediately** (Webhook)
```rust
// email/provisioning/mod.rs

pub async fn provision_email_account(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<KratosWebhookPayload>,
) -> impl IntoResponse {
    // 1. Create Stalwart account IMMEDIATELY (before verification)
    let stalwart_user_id = state.stalwart_client
        .create_account(&identity.traits.email, &identity.traits.first_name, &identity.traits.last_name)
        .await?;

    // 2. Store in database
    sqlx::query!(
        r#"
        INSERT INTO email.email_accounts
        (kratos_identity_id, email_address, stalwart_user_id, is_active)
        VALUES ($1, $2, $3, false)  -- NOT active until verified
        "#,
        identity.id,
        identity.traits.email,
        stalwart_user_id
    )
    .execute(&state.db_pool)
    .await?;

    // 3. Create mailboxes
    state.jmap_client.create_default_mailboxes(&stalwart_user_id).await?;

    Ok(StatusCode::OK)
}
```

**Phase 2: Activate After Verification**

Add new webhook endpoint:

```rust
// search/api/webhooks.rs

pub async fn handle_email_verified(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<KratosWebhookPayload>,
) -> impl IntoResponse {
    let identity_id = payload.identity.id;

    // Activate email account
    sqlx::query!(
        r#"
        UPDATE email.email_accounts
        SET is_active = true
        WHERE kratos_identity_id = $1
        "#,
        identity_id
    )
    .execute(&state.db_pool)
    .await?;

    tracing::info!("Email account activated for user {}", identity_id);

    (StatusCode::OK, Json(json!({"success": true})))
}
```

**Update Kratos Config** (`ory/kratos/kratos.yml`):

```yaml
webhooks:
  - hook: after_registration
    targets:
      - http://search-service:3000/internal/auth/user-created
      - http://email-service:3001/internal/mail/provision

  - hook: after_verification
    targets:
      - http://search-service:3000/internal/auth/email-verified
      - http://email-service:3001/internal/mail/activate
```

---

## User Journey Example

```
1. User visits https://arack.io/auth/register

2. Fills Step 1:
   - First Name: John
   - Last Name: Doe
   - Date of Birth: 1990-01-15
   - Gender: Male

3. Step 2 shows suggestions:
   ✓ john.doe@arack.io (Available)
   ✓ johndoe@arack.io (Available)

   User selects: john.doe@arack.io

4. Step 3: Creates password
   - Password: MySecurePass123!
   - Confirm: MySecurePass123!
   - Strength: Strong 🟩🟩🟩🟩

5. Clicks "Create Account"
   → Kratos creates identity
   → Webhook creates email account (john.doe@arack.io)
   → Kratos sends verification email to john.doe@arack.io

6. User sees: "Check your email for verification link"

7. User goes to https://mail.arack.io
   - Logs in with john.doe@arack.io / MySecurePass123!
   - Finds verification email in inbox
   - Clicks verification link

8. Email account activated!
   - Can now use search engine
   - Can send/receive emails
```

---

## Implementation Timeline

### Week 1: Database & Backend Foundation
- ✅ Update Kratos identity schema
- ✅ Create migration 009 (new columns)
- ✅ Implement username availability API
- ✅ Implement username suggestions API
- ✅ Update webhook handlers
- ✅ Add email verification webhook

### Week 2: Frontend Implementation
- ✅ Create multi-step registration form
- ✅ Add username selection UI
- ✅ Add password strength indicator
- ✅ Add date picker for DOB
- ✅ Integrate with Kratos flows

### Week 3: Testing & Polish
- ✅ End-to-end registration testing
- ✅ Username availability caching
- ✅ Reserved username validation
- ✅ Error handling and UX polish
- ✅ Load testing

### Week 4: Deployment
- ✅ Deploy to staging
- ✅ Update Kratos config on VPS
- ✅ Test email verification flow
- ✅ Deploy to production
- ✅ Monitor for issues

---

## Environment Variables (No Changes Needed!)

Kratos already configured with:
- PostgreSQL DSN ✅
- SMTP (RESEND) ✅
- Webhooks ✅
- CORS ✅

Only additions needed:
```bash
# .env (backend)
# No new variables needed! Everything works with existing setup.
```

---

## Migration from Current System

**No breaking changes!** This is purely additive:

1. Existing users keep working (have email field)
2. New users get username field populated
3. Old registration form still works (if kept)
4. Can run both in parallel during transition

---

## Success Criteria

- ✅ Users can choose username@arack.io
- ✅ Real-time availability checking works
- ✅ Username suggestions based on name
- ✅ Password strength indicator functional
- ✅ Email accounts created automatically
- ✅ Verification email received in webmail
- ✅ Account activated after verification
- ✅ No breaking changes to existing users
- ✅ Average registration time < 3 minutes
- ✅ Username check response time < 200ms

---

## Next Steps

1. **Review Plan** - Confirm this approach works for you
2. **Update Identity Schema** - Add new fields to Kratos
3. **Implement Backend** - Username availability APIs
4. **Build Frontend** - Multi-step registration form
5. **Test End-to-End** - Full registration → verification flow
6. **Deploy** - Update VPS configuration

**Advantages of Keeping Kratos:**
- ✅ Already working and integrated
- ✅ No code rewrite needed
- ✅ Just add custom fields to identity schema
- ✅ Webhook system already provisioning emails
- ✅ Verification flows already implemented
- ✅ Session management handled
- ✅ Future-proof (can add OAuth, 2FA easily)

Ready to start implementation? Should I begin with Week 1 tasks?
