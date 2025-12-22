# Phase 8: Email Provider Registration System - Complete Redesign

## Problem Statement

**Current Flow (Wrong for Email Provider)**:
```
User registers with existing email (Gmail)
→ Verify external email
→ Get @arack.io account
```

**Why This Is Wrong:**
- We're an email provider like Gmail/Outlook
- Users don't have @arack.io email yet - they're signing up to GET one
- Creates chicken-and-egg problem (need email to get email)
- External email verification is irrelevant for our use case

---

## New Registration Flow (Correct for Email Provider)

```
┌─────────────────────────────────────────────────────────────────┐
│                     REGISTRATION JOURNEY                        │
└─────────────────────────────────────────────────────────────────┘

Step 1: Choose Username
┌────────────────────────────┐
│ Choose your email address  │
│ ┌──────────────────────┐   │
│ │ desired_username     │   │  ← Check availability in real-time
│ └──────────────────────┘   │
│ @arack.io                  │
│                            │
│ ✓ Available!               │  ← Instant feedback
│                            │
└────────────────────────────┘

Step 2: Personal Information
┌────────────────────────────┐
│ Tell us about yourself     │
│ ┌──────────────────────┐   │
│ │ First Name           │   │
│ └──────────────────────┘   │
│ ┌──────────────────────┐   │
│ │ Last Name            │   │
│ └──────────────────────┘   │
│ ⚪ Male  ⚪ Female       │
│                            │
└────────────────────────────┘

Step 3: Password Creation
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

Step 4: Phone Verification (Critical!)
┌────────────────────────────┐
│ Verify your phone number   │
│ ┌──────────────────────┐   │
│ │ +1 (555) 123-4567    │   │  ← International format
│ └──────────────────────┘   │
│                            │
│ [Send Verification Code]   │
└────────────────────────────┘
         ↓
┌────────────────────────────┐
│ Enter 6-digit code         │
│ ┌──┬──┬──┬──┬──┬──┐       │
│ │ 1│ 2│ 3│ 4│ 5│ 6│       │  ← OTP Input (existing component)
│ └──┴──┴──┴──┴──┴──┘       │
│                            │
│ Didn't receive? Resend     │  ← Cooldown: 60s
│ Code expires in 5:00       │
└────────────────────────────┘
         ↓
✅ Account Created!
   Email: username@arack.io
   Status: Phone verified
```

---

## Architecture Design

### 1. Technology Stack

**Phone Verification Service** (Choose One):

| Service | Pros | Cons | Cost |
|---------|------|------|------|
| **Twilio Verify API** ⭐ | Easy integration, reliable, fraud detection | $0.05/verification | Best for production |
| **AWS SNS** | Cheap, scalable, already in AWS ecosystem | More complex setup | $0.006/SMS |
| **Vonage (Nexmo)** | Good deliverability, global coverage | Slightly more expensive | $0.0075/SMS |
| **ClickSend** | Cheap, simple API | Lower reliability | $0.04/SMS |

**Recommendation**: **Twilio Verify API**
- Built-in OTP generation and validation
- Automatic retry logic
- Fraud detection (velocity checks, IP blocking)
- Rate limiting included
- 2FA-optimized delivery routes

---

### 2. Database Schema Changes

**Update `users` table in search schema:**

```sql
-- Migration: 009_redesign_registration.sql

-- Drop email verification columns (no longer needed)
ALTER TABLE auth.users DROP COLUMN IF EXISTS email_verified;
ALTER TABLE auth.users DROP COLUMN IF EXISTS email_verification_token;
ALTER TABLE auth.users DROP COLUMN IF EXISTS email_verification_sent_at;

-- Add new fields for phone verification
ALTER TABLE auth.users
  ADD COLUMN IF NOT EXISTS username VARCHAR(50) UNIQUE NOT NULL,
  ADD COLUMN IF NOT EXISTS email VARCHAR(255) UNIQUE NOT NULL, -- username@arack.io
  ADD COLUMN IF NOT EXISTS first_name VARCHAR(100) NOT NULL,
  ADD COLUMN IF NOT EXISTS last_name VARCHAR(100) NOT NULL,
  ADD COLUMN IF NOT EXISTS gender VARCHAR(10) CHECK (gender IN ('male', 'female')),
  ADD COLUMN IF NOT EXISTS phone_number VARCHAR(20) UNIQUE NOT NULL,
  ADD COLUMN IF NOT EXISTS phone_verified BOOLEAN DEFAULT FALSE,
  ADD COLUMN IF NOT EXISTS phone_verified_at TIMESTAMP WITH TIME ZONE,
  ADD COLUMN IF NOT EXISTS phone_country_code VARCHAR(5) NOT NULL DEFAULT '+1';

-- Add unique constraint on lowercase username (case-insensitive)
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_username_lower
  ON auth.users (LOWER(username));

-- Create phone verification attempts table (rate limiting)
CREATE TABLE IF NOT EXISTS auth.phone_verification_attempts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    phone_number VARCHAR(20) NOT NULL,
    ip_address INET NOT NULL,
    attempt_type VARCHAR(20) NOT NULL, -- 'send_otp' or 'verify_otp'
    success BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    -- Indexes for rate limiting queries
    INDEX idx_phone_attempts_phone_time (phone_number, created_at),
    INDEX idx_phone_attempts_ip_time (ip_address, created_at)
);

-- Create username reservations table (prevent race conditions)
CREATE TABLE IF NOT EXISTS auth.username_reservations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) UNIQUE NOT NULL,
    session_id VARCHAR(255) NOT NULL, -- Frontend session ID
    reserved_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP WITH TIME ZONE DEFAULT (CURRENT_TIMESTAMP + INTERVAL '15 minutes'),

    INDEX idx_username_reservations_expires (expires_at)
);

-- Auto-cleanup expired reservations (PostgreSQL cron or background job)
-- DELETE FROM auth.username_reservations WHERE expires_at < NOW();
```

**Remove Ory Kratos Dependency (Complete Removal)**:

Since we're doing phone-based registration without external email verification, we should **remove Ory Kratos entirely** and use a simpler custom authentication system.

---

### 3. Backend API Design (Rust)

**New Registration Endpoints:**

```rust
// src/auth/registration.rs

use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

// ============================================================================
// Step 1: Check Username Availability
// ============================================================================

#[derive(Deserialize, Validate)]
pub struct CheckUsernameRequest {
    #[validate(
        length(min = 3, max = 30),
        regex(path = "USERNAME_REGEX", message = "Only lowercase letters, numbers, dots, underscores")
    )]
    username: String,
}

lazy_static! {
    static ref USERNAME_REGEX: Regex = Regex::new(r"^[a-z0-9._]+$").unwrap();
}

#[derive(Serialize)]
pub struct CheckUsernameResponse {
    available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    suggestions: Option<Vec<String>>, // If taken, suggest alternatives
}

pub async fn check_username_availability(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CheckUsernameRequest>,
) -> Result<Json<CheckUsernameResponse>, AppError> {
    // Validate format
    payload.validate()?;

    let username = payload.username.to_lowercase();

    // Check if username exists
    let exists = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM auth.users WHERE LOWER(username) = $1)",
        username
    )
    .fetch_one(&state.db_pool)
    .await?
    .exists
    .unwrap_or(false);

    // Check if username is reserved
    let reserved = sqlx::query!(
        "SELECT EXISTS(
            SELECT 1 FROM auth.username_reservations
            WHERE LOWER(username) = $1 AND expires_at > NOW()
        )",
        username
    )
    .fetch_one(&state.db_pool)
    .await?
    .exists
    .unwrap_or(false);

    if exists || reserved {
        // Generate suggestions: username1, username2, username.2024
        let suggestions = generate_username_suggestions(&username, &state.db_pool).await?;

        return Ok(Json(CheckUsernameResponse {
            available: false,
            suggestions: Some(suggestions),
        }));
    }

    Ok(Json(CheckUsernameResponse {
        available: true,
        suggestions: None,
    }))
}

// ============================================================================
// Step 2: Reserve Username (Prevent Race Conditions)
// ============================================================================

#[derive(Deserialize, Validate)]
pub struct ReserveUsernameRequest {
    #[validate(length(min = 3, max = 30))]
    username: String,
    session_id: String, // Generated by frontend
}

pub async fn reserve_username(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ReserveUsernameRequest>,
) -> Result<StatusCode, AppError> {
    let username = payload.username.to_lowercase();

    // Reserve for 15 minutes
    sqlx::query!(
        r#"
        INSERT INTO auth.username_reservations (username, session_id)
        VALUES ($1, $2)
        ON CONFLICT (username) DO UPDATE
        SET session_id = $2, expires_at = NOW() + INTERVAL '15 minutes'
        "#,
        username,
        payload.session_id
    )
    .execute(&state.db_pool)
    .await?;

    Ok(StatusCode::OK)
}

// ============================================================================
// Step 3: Send Phone OTP
// ============================================================================

#[derive(Deserialize, Validate)]
pub struct SendOtpRequest {
    #[validate(phone)]
    phone_number: String,
    #[validate(length(min = 2, max = 5))]
    country_code: String, // e.g., "+1", "+213"
}

pub async fn send_phone_otp(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<SendOtpRequest>,
) -> Result<StatusCode, AppError> {
    payload.validate()?;

    let full_phone = format!("{}{}", payload.country_code, payload.phone_number);
    let ip_addr = addr.ip();

    // Rate limiting: Max 3 OTP sends per phone per hour
    let recent_attempts = sqlx::query!(
        r#"
        SELECT COUNT(*) as count FROM auth.phone_verification_attempts
        WHERE phone_number = $1
          AND attempt_type = 'send_otp'
          AND created_at > NOW() - INTERVAL '1 hour'
        "#,
        full_phone
    )
    .fetch_one(&state.db_pool)
    .await?
    .count
    .unwrap_or(0);

    if recent_attempts >= 3 {
        return Err(AppError::TooManyRequests {
            message: "Too many OTP requests. Please try again later.".to_string(),
            retry_after: 3600, // 1 hour
        });
    }

    // Rate limiting: Max 10 OTP sends per IP per hour (prevent abuse)
    let ip_attempts = sqlx::query!(
        r#"
        SELECT COUNT(*) as count FROM auth.phone_verification_attempts
        WHERE ip_address = $1
          AND attempt_type = 'send_otp'
          AND created_at > NOW() - INTERVAL '1 hour'
        "#,
        ip_addr
    )
    .fetch_one(&state.db_pool)
    .await?
    .count
    .unwrap_or(0);

    if ip_attempts >= 10 {
        return Err(AppError::TooManyRequests {
            message: "Too many requests from your IP.".to_string(),
            retry_after: 3600,
        });
    }

    // Send OTP via Twilio Verify API
    let otp_sent = state.twilio_client
        .send_verification(&full_phone)
        .await?;

    // Log attempt
    sqlx::query!(
        r#"
        INSERT INTO auth.phone_verification_attempts
        (phone_number, ip_address, attempt_type, success)
        VALUES ($1, $2, 'send_otp', $3)
        "#,
        full_phone,
        ip_addr,
        otp_sent
    )
    .execute(&state.db_pool)
    .await?;

    Ok(StatusCode::OK)
}

// ============================================================================
// Step 4: Complete Registration with OTP
// ============================================================================

#[derive(Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 3, max = 30))]
    username: String,

    #[validate(length(min = 1, max = 100))]
    first_name: String,

    #[validate(length(min = 1, max = 100))]
    last_name: String,

    #[validate(custom = "validate_gender")]
    gender: String, // "male" or "female"

    #[validate(phone)]
    phone_number: String,

    country_code: String,

    #[validate(length(min = 8))]
    password: String,

    #[validate(length(equal = 6))]
    otp_code: String, // 6-digit code

    session_id: String, // For username reservation check
}

fn validate_gender(gender: &str) -> Result<(), ValidationError> {
    if gender != "male" && gender != "female" {
        return Err(ValidationError::new("Invalid gender"));
    }
    Ok(())
}

#[derive(Serialize)]
pub struct RegisterResponse {
    user_id: Uuid,
    email: String,
    access_token: String,
}

pub async fn register_user(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, AppError> {
    payload.validate()?;

    let username = payload.username.to_lowercase();
    let email = format!("{}@arack.io", username);
    let full_phone = format!("{}{}", payload.country_code, payload.phone_number);
    let ip_addr = addr.ip();

    // 1. Verify username was reserved by this session
    let reservation = sqlx::query!(
        r#"
        SELECT session_id FROM auth.username_reservations
        WHERE LOWER(username) = $1 AND expires_at > NOW()
        "#,
        username
    )
    .fetch_optional(&state.db_pool)
    .await?;

    if let Some(res) = reservation {
        if res.session_id != payload.session_id {
            return Err(AppError::BadRequest {
                message: "Username reservation expired. Please start over.".to_string(),
            });
        }
    } else {
        return Err(AppError::BadRequest {
            message: "Username not reserved.".to_string(),
        });
    }

    // 2. Verify OTP with Twilio
    let otp_valid = state.twilio_client
        .check_verification(&full_phone, &payload.otp_code)
        .await?;

    if !otp_valid {
        // Log failed attempt
        sqlx::query!(
            r#"
            INSERT INTO auth.phone_verification_attempts
            (phone_number, ip_address, attempt_type, success)
            VALUES ($1, $2, 'verify_otp', false)
            "#,
            full_phone,
            ip_addr
        )
        .execute(&state.db_pool)
        .await?;

        return Err(AppError::Unauthorized {
            message: "Invalid verification code.".to_string(),
        });
    }

    // 3. Hash password
    let password_hash = hash_password(&payload.password)?;

    // 4. Create user in transaction
    let mut tx = state.db_pool.begin().await?;

    let user_id = sqlx::query!(
        r#"
        INSERT INTO auth.users
        (username, email, first_name, last_name, gender, phone_number,
         phone_country_code, phone_verified, phone_verified_at, password_hash)
        VALUES ($1, $2, $3, $4, $5, $6, $7, true, NOW(), $8)
        RETURNING id
        "#,
        username,
        email,
        payload.first_name,
        payload.last_name,
        payload.gender,
        payload.phone_number,
        payload.country_code,
        password_hash
    )
    .fetch_one(&mut *tx)
    .await?
    .id;

    // 5. Create email account in email service (via internal API or shared function)
    // This replaces the Kratos webhook approach
    create_email_account_internal(
        &mut tx,
        user_id,
        &email,
        &payload.first_name,
        &payload.last_name,
    ).await?;

    // 6. Delete username reservation
    sqlx::query!(
        "DELETE FROM auth.username_reservations WHERE LOWER(username) = $1",
        username
    )
    .execute(&mut *tx)
    .await?;

    // 7. Log successful verification
    sqlx::query!(
        r#"
        INSERT INTO auth.phone_verification_attempts
        (phone_number, ip_address, attempt_type, success)
        VALUES ($1, $2, 'verify_otp', true)
        "#,
        full_phone,
        ip_addr
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    // 8. Generate JWT access token
    let access_token = generate_jwt_token(user_id, &email)?;

    Ok(Json(RegisterResponse {
        user_id,
        email,
        access_token,
    }))
}

// ============================================================================
// Helper: Create Email Account (Internal - Replaces Webhook)
// ============================================================================

async fn create_email_account_internal(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    email: &str,
    first_name: &str,
    last_name: &str,
) -> Result<(), AppError> {
    // Create Stalwart account via admin API
    let stalwart_user_id = create_stalwart_account(email, first_name, last_name).await?;

    // Insert into email.email_accounts
    sqlx::query!(
        r#"
        INSERT INTO email.email_accounts
        (kratos_identity_id, email_address, stalwart_user_id)
        VALUES ($1, $2, $3)
        "#,
        user_id, // Now references auth.users.id instead of Kratos identity
        email,
        stalwart_user_id
    )
    .execute(&mut **tx)
    .await?;

    // Create default mailboxes
    create_default_mailboxes(&stalwart_user_id).await?;

    Ok(())
}

// ============================================================================
// Helper: Generate Username Suggestions
// ============================================================================

async fn generate_username_suggestions(
    base_username: &str,
    db_pool: &PgPool,
) -> Result<Vec<String>, AppError> {
    let mut suggestions = Vec::new();
    let year = chrono::Utc::now().year();

    // Generate 5 suggestions
    let candidates = vec![
        format!("{}{}", base_username, rand::random::<u16>() % 1000),
        format!("{}_{}", base_username, year),
        format!("{}.{}", base_username, rand::random::<u16>() % 100),
        format!("{}_{}", base_username, rand::random::<u16>() % 1000),
        format!("{}.official", base_username),
    ];

    for candidate in candidates {
        let exists = sqlx::query!(
            "SELECT EXISTS(SELECT 1 FROM auth.users WHERE LOWER(username) = $1)",
            candidate.to_lowercase()
        )
        .fetch_one(db_pool)
        .await?
        .exists
        .unwrap_or(false);

        if !exists {
            suggestions.push(candidate);
        }

        if suggestions.len() >= 3 {
            break;
        }
    }

    Ok(suggestions)
}
```

**Twilio Client Implementation:**

```rust
// src/auth/twilio.rs

use reqwest::Client;
use serde::{Deserialize, Serialize};
use anyhow::Result;

pub struct TwilioClient {
    account_sid: String,
    auth_token: String,
    verify_service_sid: String,
    client: Client,
}

impl TwilioClient {
    pub fn new(account_sid: String, auth_token: String, verify_service_sid: String) -> Self {
        Self {
            account_sid,
            auth_token,
            verify_service_sid,
            client: Client::new(),
        }
    }

    /// Send OTP to phone number
    pub async fn send_verification(&self, phone_number: &str) -> Result<bool> {
        let url = format!(
            "https://verify.twilio.com/v2/Services/{}/Verifications",
            self.verify_service_sid
        );

        let params = [
            ("To", phone_number),
            ("Channel", "sms"), // or "call", "whatsapp"
        ];

        let response = self.client
            .post(&url)
            .basic_auth(&self.account_sid, Some(&self.auth_token))
            .form(&params)
            .send()
            .await?;

        Ok(response.status().is_success())
    }

    /// Verify OTP code
    pub async fn check_verification(&self, phone_number: &str, code: &str) -> Result<bool> {
        let url = format!(
            "https://verify.twilio.com/v2/Services/{}/VerificationCheck",
            self.verify_service_sid
        );

        let params = [
            ("To", phone_number),
            ("Code", code),
        ];

        let response = self.client
            .post(&url)
            .basic_auth(&self.account_sid, Some(&self.auth_token))
            .form(&params)
            .send()
            .await?;

        if response.status().is_success() {
            let result: VerificationResult = response.json().await?;
            return Ok(result.status == "approved");
        }

        Ok(false)
    }
}

#[derive(Deserialize)]
struct VerificationResult {
    status: String, // "pending", "approved", "canceled"
}
```

---

### 4. Frontend Implementation

**Registration Flow Components:**

```typescript
// frontend-search/src/routes/auth/register/+page.svelte

<script lang="ts">
import { OTPInput, Button, Input, Label } from '$lib/components/ui';
import { api } from '$lib/stores/api';

let step = 1; // 1=username, 2=info, 3=password, 4=phone, 5=otp
let sessionId = crypto.randomUUID();

// Step 1: Username
let username = $state('');
let usernameAvailable = $state<boolean | null>(null);
let usernameSuggestions = $state<string[]>([]);
let checkingUsername = $state(false);

// Debounced username check
const checkUsername = debounce(async (value: string) => {
    if (value.length < 3) return;

    checkingUsername = true;
    const result = await api.checkUsername({ username: value });
    usernameAvailable = result.available;
    usernameSuggestions = result.suggestions || [];
    checkingUsername = false;
}, 500);

$effect(() => {
    checkUsername(username);
});

async function reserveAndContinue() {
    await api.reserveUsername({ username, session_id: sessionId });
    step = 2;
}

// Step 2: Personal Info
let firstName = $state('');
let lastName = $state('');
let gender = $state('');

// Step 3: Password
let password = $state('');
let confirmPassword = $state('');
let passwordStrength = $derived(calculatePasswordStrength(password));

// Step 4: Phone
let countryCode = $state('+1');
let phoneNumber = $state('');
let sendingOtp = $state(false);

async function sendOtp() {
    sendingOtp = true;
    await api.sendPhoneOtp({ phone_number: phoneNumber, country_code: countryCode });
    sendingOtp = false;
    step = 5;
}

// Step 5: OTP Verification
let otpCode = $state('');
let verifying = $state(false);

async function completeRegistration() {
    verifying = true;
    try {
        const result = await api.register({
            username,
            first_name: firstName,
            last_name: lastName,
            gender,
            phone_number: phoneNumber,
            country_code: countryCode,
            password,
            otp_code: otpCode,
            session_id: sessionId
        });

        // Store token and redirect
        localStorage.setItem('access_token', result.access_token);
        goto('/');
    } catch (error) {
        alert('Invalid verification code');
    } finally {
        verifying = false;
    }
}
</script>

{#if step === 1}
<div class="max-w-md mx-auto mt-20">
    <h1 class="text-2xl font-bold mb-4">Choose your email address</h1>

    <div class="flex items-center gap-2">
        <Input bind:value={username} placeholder="desired_username" />
        <span class="text-muted-foreground">@arack.io</span>
    </div>

    {#if checkingUsername}
        <p class="text-sm text-muted-foreground mt-2">Checking availability...</p>
    {:else if usernameAvailable === true}
        <p class="text-sm text-green-600 mt-2">✓ Available!</p>
        <Button onclick={reserveAndContinue} class="mt-4 w-full">Continue</Button>
    {:else if usernameAvailable === false}
        <p class="text-sm text-red-600 mt-2">✗ Already taken</p>
        {#if usernameSuggestions.length > 0}
            <p class="text-sm mt-2">Suggestions:</p>
            <div class="flex flex-wrap gap-2 mt-1">
                {#each usernameSuggestions as suggestion}
                    <button
                        onclick={() => username = suggestion}
                        class="px-3 py-1 bg-secondary rounded-md text-sm hover:bg-secondary/80"
                    >
                        {suggestion}
                    </button>
                {/each}
            </div>
        {/if}
    {/if}
</div>

{:else if step === 2}
<div class="max-w-md mx-auto mt-20">
    <h1 class="text-2xl font-bold mb-4">Tell us about yourself</h1>

    <div class="space-y-4">
        <div>
            <Label for="first_name">First Name</Label>
            <Input id="first_name" bind:value={firstName} />
        </div>

        <div>
            <Label for="last_name">Last Name</Label>
            <Input id="last_name" bind:value={lastName} />
        </div>

        <div>
            <Label>Gender</Label>
            <div class="flex gap-4 mt-2">
                <label class="flex items-center gap-2">
                    <input type="radio" bind:group={gender} value="male" />
                    Male
                </label>
                <label class="flex items-center gap-2">
                    <input type="radio" bind:group={gender} value="female" />
                    Female
                </label>
            </div>
        </div>

        <Button
            onclick={() => step = 3}
            disabled={!firstName || !lastName || !gender}
            class="w-full"
        >
            Continue
        </Button>
    </div>
</div>

{:else if step === 3}
<div class="max-w-md mx-auto mt-20">
    <h1 class="text-2xl font-bold mb-4">Create your password</h1>

    <div class="space-y-4">
        <div>
            <Label for="password">Password (min 8 characters)</Label>
            <Input
                id="password"
                type="password"
                bind:value={password}
            />
            <div class="mt-2 flex gap-1">
                {#each Array(4) as _, i}
                    <div
                        class="h-2 flex-1 rounded-full"
                        class:bg-green-500={passwordStrength > i}
                        class:bg-gray-300={passwordStrength <= i}
                    ></div>
                {/each}
            </div>
        </div>

        <div>
            <Label for="confirm_password">Confirm Password</Label>
            <Input
                id="confirm_password"
                type="password"
                bind:value={confirmPassword}
            />
        </div>

        <Button
            onclick={() => step = 4}
            disabled={password.length < 8 || password !== confirmPassword}
            class="w-full"
        >
            Continue
        </Button>
    </div>
</div>

{:else if step === 4}
<div class="max-w-md mx-auto mt-20">
    <h1 class="text-2xl font-bold mb-4">Verify your phone number</h1>

    <div class="space-y-4">
        <div class="flex gap-2">
            <select bind:value={countryCode} class="w-24 border rounded-md px-3 py-2">
                <option value="+1">🇺🇸 +1</option>
                <option value="+213">🇩🇿 +213</option>
                <option value="+44">🇬🇧 +44</option>
                <option value="+33">🇫🇷 +33</option>
            </select>

            <Input
                bind:value={phoneNumber}
                placeholder="(555) 123-4567"
                class="flex-1"
            />
        </div>

        <Button
            onclick={sendOtp}
            disabled={phoneNumber.length < 10 || sendingOtp}
            class="w-full"
        >
            {sendingOtp ? 'Sending...' : 'Send Verification Code'}
        </Button>
    </div>
</div>

{:else if step === 5}
<div class="max-w-md mx-auto mt-20">
    <h1 class="text-2xl font-bold mb-4">Enter verification code</h1>

    <p class="text-sm text-muted-foreground mb-6">
        We sent a 6-digit code to {countryCode} {phoneNumber}
    </p>

    <OTPInput bind:value={otpCode} length={6} />

    <Button
        onclick={completeRegistration}
        disabled={otpCode.length !== 6 || verifying}
        class="w-full mt-6"
    >
        {verifying ? 'Verifying...' : 'Create Account'}
    </Button>

    <button class="text-sm text-primary mt-4 w-full">
        Didn't receive? Resend
    </button>
</div>
{/if}
```

**API Client Updates:**

```typescript
// shared/api-client/index.ts

export class SearchEngineAPI {
    // ... existing methods

    async checkUsername(data: { username: string }): Promise<{
        available: boolean;
        suggestions?: string[];
    }> {
        const response = await this.client.post('/api/auth/check-username', data);
        return response.data;
    }

    async reserveUsername(data: { username: string; session_id: string }): Promise<void> {
        await this.client.post('/api/auth/reserve-username', data);
    }

    async sendPhoneOtp(data: {
        phone_number: string;
        country_code: string;
    }): Promise<void> {
        await this.client.post('/api/auth/send-otp', data);
    }

    async register(data: {
        username: string;
        first_name: string;
        last_name: string;
        gender: string;
        phone_number: string;
        country_code: string;
        password: string;
        otp_code: string;
        session_id: string;
    }): Promise<{ user_id: string; email: string; access_token: string }> {
        const response = await this.client.post('/api/auth/register', data);
        return response.data;
    }
}
```

---

### 5. Security Considerations

**Rate Limiting:**

1. **Username Checks**: 20 per IP per minute (prevent enumeration)
2. **OTP Sends**: 3 per phone per hour (prevent SMS bombing)
3. **OTP Verifications**: 5 attempts per phone per 10 minutes (prevent brute force)
4. **IP-based Limit**: 10 OTP sends per IP per hour (prevent abuse)

**Username Validation:**

```rust
// Allowed characters: a-z, 0-9, dots, underscores
// No consecutive dots: user..name ❌
// No leading/trailing dots: .username ❌, username. ❌
// Length: 3-30 characters
// Reserved usernames: admin, support, noreply, etc.

const RESERVED_USERNAMES: &[&str] = &[
    "admin", "administrator", "root", "system",
    "support", "help", "info", "contact",
    "noreply", "no-reply", "postmaster",
    "abuse", "security", "webmaster",
];
```

**Phone Number Validation:**

```rust
use phonenumber::PhoneNumber;

fn validate_phone_number(phone: &str, country_code: &str) -> Result<String, AppError> {
    let full_number = format!("{}{}", country_code, phone);

    let parsed = PhoneNumber::parse(None, &full_number)
        .map_err(|_| AppError::BadRequest {
            message: "Invalid phone number".to_string(),
        })?;

    if !parsed.is_valid() {
        return Err(AppError::BadRequest {
            message: "Invalid phone number".to_string(),
        });
    }

    Ok(parsed.format().mode(phonenumber::Mode::E164).to_string())
}
```

**Password Hashing:**

```rust
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| AppError::InternalServerError)?
        .to_string();

    Ok(password_hash)
}

pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, AppError> {
    let parsed_hash = PasswordHash::new(password_hash)
        .map_err(|_| AppError::InternalServerError)?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}
```

---

### 6. Cost Estimation (Twilio)

**SMS OTP Costs:**

| Scenario | Cost per User | Monthly (1000 users) | Monthly (10,000 users) |
|----------|---------------|----------------------|------------------------|
| **1 OTP send** (successful) | $0.05 | $50 | $500 |
| **2 OTP sends** (1 resend) | $0.10 | $100 | $1,000 |
| **Average (1.5 sends)** | $0.075 | $75 | $750 |

**Cost Optimization:**

1. Use **Voice fallback** if SMS fails (same price)
2. Implement **60-second cooldown** between resends
3. **Warn users** about SMS costs to reduce resends
4. Consider **WhatsApp OTP** (cheaper in some regions)

---

### 7. Stalwart Email Account Creation

**Replace Kratos Webhook with Direct Creation:**

```rust
// src/email/stalwart.rs

use reqwest::Client;
use serde_json::json;

pub struct StalwartClient {
    admin_url: String,
    admin_token: String,
    client: Client,
}

impl StalwartClient {
    pub async fn create_account(
        &self,
        email: &str,
        first_name: &str,
        last_name: &str,
        password_hash: &str,
    ) -> Result<String, AppError> {
        let response = self.client
            .post(&format!("{}/api/admin/accounts", self.admin_url))
            .bearer_auth(&self.admin_token)
            .json(&json!({
                "email": email,
                "name": format!("{} {}", first_name, last_name),
                "password": password_hash,
                "quota": 5368709120, // 5GB
            }))
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;
        let stalwart_user_id = result["id"]
            .as_str()
            .ok_or(AppError::InternalServerError)?
            .to_string();

        Ok(stalwart_user_id)
    }
}
```

---

### 8. Migration Plan from Current System

**Step-by-Step Migration:**

1. **Add new columns to users table** (migration 009)
2. **Implement new registration API** (parallel to old)
3. **Add Twilio client** to AppState
4. **Update frontend** with new registration flow
5. **Deploy both systems** (old + new running together)
6. **Switch frontend** to use new registration
7. **Disable old Kratos registration** routes
8. **Remove Kratos entirely** after 30 days

**Backward Compatibility:**

```rust
// Support both old and new users during migration
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let user = sqlx::query_as!(
        User,
        "SELECT * FROM auth.users WHERE email = $1 OR username = $1",
        payload.identifier // Can be email OR username
    )
    .fetch_one(&state.db_pool)
    .await?;

    // Verify password
    if !verify_password(&payload.password, &user.password_hash)? {
        return Err(AppError::Unauthorized {
            message: "Invalid credentials".to_string(),
        });
    }

    // Check if phone verified (new users must have this)
    if !user.phone_verified {
        return Err(AppError::Forbidden {
            message: "Please verify your phone number".to_string(),
        });
    }

    let access_token = generate_jwt_token(user.id, &user.email)?;

    Ok(Json(LoginResponse {
        access_token,
        user: UserInfo {
            id: user.id,
            email: user.email,
            username: user.username,
            first_name: user.first_name,
            last_name: user.last_name,
        },
    }))
}
```

---

## Implementation Timeline

### Week 1: Backend Foundation
- ✅ Create migration 009 (new tables)
- ✅ Implement Twilio client
- ✅ Implement username availability endpoint
- ✅ Implement username reservation system
- ✅ Add phone verification rate limiting

### Week 2: Registration API
- ✅ Implement send OTP endpoint
- ✅ Implement complete registration endpoint
- ✅ Add Stalwart account creation
- ✅ Write integration tests
- ✅ Setup Twilio sandbox for testing

### Week 3: Frontend Implementation
- ✅ Create multi-step registration UI
- ✅ Add username availability checker
- ✅ Implement OTP input component (already exists!)
- ✅ Add phone number input with country selector
- ✅ Add password strength indicator

### Week 4: Testing & Deployment
- ✅ End-to-end testing
- ✅ Load testing (rate limiting)
- ✅ Deploy to staging
- ✅ Migrate existing users (data migration script)
- ✅ Deploy to production
- ✅ Monitor for issues

---

## Success Criteria

- ✅ Users can register with @arack.io email address
- ✅ Phone verification works reliably (>98% delivery)
- ✅ Username suggestions work when taken
- ✅ Rate limiting prevents abuse
- ✅ Email accounts automatically provisioned
- ✅ No Kratos dependency
- ✅ Average registration time < 2 minutes
- ✅ SMS delivery < 10 seconds (Twilio SLA)

---

## Environment Variables

```bash
# .env (add these)

# Twilio Configuration
TWILIO_ACCOUNT_SID=ACxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
TWILIO_AUTH_TOKEN=your_auth_token_here
TWILIO_VERIFY_SERVICE_SID=VAxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx

# JWT Configuration
JWT_SECRET=your-super-secret-jwt-key-32-chars-minimum
JWT_EXPIRATION=86400  # 24 hours in seconds

# Stalwart Admin API
STALWART_ADMIN_URL=http://stalwart:8080
STALWART_ADMIN_TOKEN=your-admin-token
```

---

## Next Steps

1. **Setup Twilio Account**:
   - Sign up at https://www.twilio.com
   - Create Verify Service
   - Get Account SID, Auth Token, and Verify Service SID
   - Test in sandbox mode first (free)

2. **Review Plan**:
   - Confirm this approach aligns with requirements
   - Discuss any additional fields needed
   - Decide on admin approval flow (if needed)

3. **Start Implementation**:
   - Begin with database migration
   - Implement backend endpoints
   - Build frontend components

Should we proceed with this plan? Any questions or adjustments needed?
