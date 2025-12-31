# Phase 8: Authentication & Security - Implementation Plan

## Overview
Implement a two-tiered authentication system:
1. **Regular Users**: Self-registration and login
2. **Admin Users**: Login with member invitation capabilities

## Architecture

### Database Schema

#### Users Table
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'user', -- 'admin' or 'user'
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
    last_login TIMESTAMP,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    invited_by UUID REFERENCES users(id),
    email_verified BOOLEAN NOT NULL DEFAULT FALSE,
    verification_token VARCHAR(255),
    reset_token VARCHAR(255),
    reset_token_expires TIMESTAMP
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_role ON users(role);
```

#### Invitations Table
```sql
CREATE TABLE invitations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL,
    token VARCHAR(255) UNIQUE NOT NULL,
    invited_by UUID NOT NULL REFERENCES users(id),
    role VARCHAR(50) NOT NULL DEFAULT 'user',
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- 'pending', 'accepted', 'expired'
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP NOT NULL,
    accepted_at TIMESTAMP
);

CREATE INDEX idx_invitations_token ON invitations(token);
CREATE INDEX idx_invitations_email ON invitations(email);
CREATE INDEX idx_invitations_status ON invitations(status);
```

#### Sessions Table (Optional - for token blacklisting)
```sql
CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    token_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMP NOT NULL,
    revoked BOOLEAN NOT NULL DEFAULT FALSE,
    ip_address VARCHAR(45),
    user_agent TEXT
);

CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_token_hash ON sessions(token_hash);
CREATE INDEX idx_sessions_expires_at ON sessions(expires_at);
```

### Backend Structure

```
src/
├── auth/
│   ├── mod.rs                 # Public API
│   ├── password.rs            # Password hashing/verification (argon2)
│   ├── jwt.rs                 # JWT token generation/validation
│   ├── middleware.rs          # Auth middleware for protected routes
│   └── models.rs              # Auth-related types
├── db/
│   ├── mod.rs
│   ├── users.rs               # User database operations
│   └── invitations.rs         # Invitation database operations
└── api/
    ├── auth.rs                # Auth endpoints
    └── admin.rs               # Admin-only endpoints
```

## Phase 8.1: Database Setup & Core Auth Module

### Tasks:
1. Create migration files for users, invitations, and sessions tables
2. Implement `src/auth/password.rs`:
   - `hash_password()` using argon2
   - `verify_password()`
3. Implement `src/auth/jwt.rs`:
   - `generate_token()` with user claims
   - `validate_token()`
   - Token expiry configuration (15min access, 7day refresh)
4. Implement `src/auth/models.rs`:
   - `User` struct
   - `UserRole` enum (Admin, User)
   - `Claims` struct for JWT payload
   - `AuthResponse` with tokens

### Dependencies to add:
```toml
[dependencies]
argon2 = "0.5"
jsonwebtoken = "9"
rand = "0.8"
validator = { version = "0.18", features = ["derive"] }
```

## Phase 8.2: User Registration & Login

### Endpoints:

#### POST `/api/auth/register` (Public)
**Request**:
```json
{
  "email": "user@example.com",
  "password": "SecurePass123!",
  "first_name": "John",
  "last_name": "Doe"
}
```

**Validation**:
- Email format validation
- Password strength (min 8 chars, uppercase, lowercase, number, special char)
- Duplicate email check

**Response**:
```json
{
  "success": true,
  "data": {
    "user": {
      "id": "uuid",
      "email": "user@example.com",
      "role": "user",
      "first_name": "John",
      "last_name": "Doe"
    },
    "access_token": "jwt_token",
    "refresh_token": "jwt_refresh_token"
  }
}
```

#### POST `/api/auth/login` (Public)
**Request**:
```json
{
  "email": "user@example.com",
  "password": "SecurePass123!"
}
```

**Response**: Same as register

#### POST `/api/auth/refresh` (Public)
**Request**:
```json
{
  "refresh_token": "jwt_refresh_token"
}
```

**Response**:
```json
{
  "success": true,
  "data": {
    "access_token": "new_jwt_token",
    "refresh_token": "new_refresh_token"
  }
}
```

#### POST `/api/auth/logout` (Authenticated)
**Headers**: `Authorization: Bearer <token>`

**Response**:
```json
{
  "success": true,
  "data": {
    "message": "Logged out successfully"
  }
}
```

#### GET `/api/auth/me` (Authenticated)
**Headers**: `Authorization: Bearer <token>`

**Response**:
```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "email": "user@example.com",
    "role": "user",
    "first_name": "John",
    "last_name": "Doe",
    "created_at": "2025-12-10T10:00:00Z"
  }
}
```

### Implementation Files:

**`src/auth/middleware.rs`**:
```rust
pub async fn require_auth(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract token from Authorization header
    // Validate JWT token
    // Fetch user from database
    // Add user to request extensions
    // Call next middleware
}

pub async fn require_admin(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // First require_auth
    // Then check if user.role == UserRole::Admin
}
```

**`src/db/users.rs`**:
```rust
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub async fn create_user(&self, email: &str, password_hash: &str, first_name: &str, last_name: &str) -> Result<User>;
    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>>;
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>>;
    pub async fn update_last_login(&self, id: Uuid) -> Result<()>;
    pub async fn update_password(&self, id: Uuid, password_hash: &str) -> Result<()>;
}
```

## Phase 8.3: Admin Invite System

### Endpoints:

#### POST `/api/admin/invitations` (Admin Only)
**Headers**: `Authorization: Bearer <admin_token>`

**Request**:
```json
{
  "email": "newmember@example.com",
  "role": "user",
  "expires_in_days": 7
}
```

**Response**:
```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "email": "newmember@example.com",
    "token": "invitation_token",
    "invite_url": "http://app.com/accept-invite?token=invitation_token",
    "expires_at": "2025-12-17T10:00:00Z"
  }
}
```

#### GET `/api/admin/invitations` (Admin Only)
**Headers**: `Authorization: Bearer <admin_token>`

**Query Params**: `?status=pending&page=1&limit=20`

**Response**:
```json
{
  "success": true,
  "data": {
    "invitations": [
      {
        "id": "uuid",
        "email": "newmember@example.com",
        "status": "pending",
        "invited_by": {
          "id": "uuid",
          "email": "admin@example.com",
          "first_name": "Admin"
        },
        "created_at": "2025-12-10T10:00:00Z",
        "expires_at": "2025-12-17T10:00:00Z"
      }
    ],
    "total": 10,
    "page": 1,
    "limit": 20
  }
}
```

#### DELETE `/api/admin/invitations/:id` (Admin Only)
**Headers**: `Authorization: Bearer <admin_token>`

**Response**:
```json
{
  "success": true,
  "data": {
    "message": "Invitation revoked successfully"
  }
}
```

#### POST `/api/auth/accept-invite` (Public)
**Request**:
```json
{
  "token": "invitation_token",
  "password": "SecurePass123!",
  "first_name": "Jane",
  "last_name": "Smith"
}
```

**Validation**:
- Token exists and not expired
- Token not already accepted
- Password strength validation

**Response**:
```json
{
  "success": true,
  "data": {
    "user": {
      "id": "uuid",
      "email": "newmember@example.com",
      "role": "user",
      "first_name": "Jane",
      "last_name": "Smith"
    },
    "access_token": "jwt_token",
    "refresh_token": "jwt_refresh_token"
  }
}
```

#### GET `/api/auth/verify-invite/:token` (Public)
**Response**:
```json
{
  "success": true,
  "data": {
    "valid": true,
    "email": "newmember@example.com",
    "expires_at": "2025-12-17T10:00:00Z"
  }
}
```

### Implementation Files:

**`src/db/invitations.rs`**:
```rust
pub struct InvitationRepository {
    pool: PgPool,
}

impl InvitationRepository {
    pub async fn create_invitation(&self, email: &str, invited_by: Uuid, role: UserRole, expires_at: DateTime<Utc>) -> Result<Invitation>;
    pub async fn find_by_token(&self, token: &str) -> Result<Option<Invitation>>;
    pub async fn list_invitations(&self, status: Option<String>, page: usize, limit: usize) -> Result<(Vec<InvitationWithInviter>, usize)>;
    pub async fn mark_as_accepted(&self, id: Uuid) -> Result<()>;
    pub async fn delete_invitation(&self, id: Uuid) -> Result<()>;
    pub async fn cleanup_expired(&self) -> Result<usize>; // Background job
}
```

## Phase 8.4: Admin User Management

### Endpoints:

#### GET `/api/admin/users` (Admin Only)
**Headers**: `Authorization: Bearer <admin_token>`

**Query Params**: `?role=user&is_active=true&page=1&limit=20&search=john`

**Response**:
```json
{
  "success": true,
  "data": {
    "users": [
      {
        "id": "uuid",
        "email": "user@example.com",
        "role": "user",
        "first_name": "John",
        "last_name": "Doe",
        "created_at": "2025-12-10T10:00:00Z",
        "last_login": "2025-12-10T15:00:00Z",
        "is_active": true
      }
    ],
    "total": 50,
    "page": 1,
    "limit": 20
  }
}
```

#### GET `/api/admin/users/:id` (Admin Only)
**Response**:
```json
{
  "success": true,
  "data": {
    "id": "uuid",
    "email": "user@example.com",
    "role": "user",
    "first_name": "John",
    "last_name": "Doe",
    "created_at": "2025-12-10T10:00:00Z",
    "last_login": "2025-12-10T15:00:00Z",
    "is_active": true,
    "invited_by": {
      "id": "uuid",
      "email": "admin@example.com",
      "first_name": "Admin"
    },
    "statistics": {
      "total_searches": 150,
      "total_clicks": 45,
      "crawl_jobs_created": 5
    }
  }
}
```

#### PATCH `/api/admin/users/:id` (Admin Only)
**Request**:
```json
{
  "is_active": false,
  "role": "admin"
}
```

**Response**:
```json
{
  "success": true,
  "data": {
    "message": "User updated successfully"
  }
}
```

#### DELETE `/api/admin/users/:id` (Admin Only)
**Response**:
```json
{
  "success": true,
  "data": {
    "message": "User deleted successfully"
  }
}
```

#### GET `/api/admin/stats` (Admin Only)
**Response**:
```json
{
  "success": true,
  "data": {
    "total_users": 50,
    "active_users": 45,
    "admin_users": 3,
    "pending_invitations": 5,
    "users_registered_last_30_days": 12,
    "most_active_users": [
      {
        "id": "uuid",
        "email": "user@example.com",
        "first_name": "John",
        "total_searches": 500
      }
    ]
  }
}
```

## Phase 8.5: Frontend - Login & Registration UI

### New Pages:

#### `frontend-admin/src/routes/login/+page.svelte`
**Features**:
- Email/password form
- "Remember me" checkbox
- Link to registration page
- Password visibility toggle
- Error handling with toast notifications
- Loading states

#### `frontend-admin/src/routes/register/+page.svelte`
**Features**:
- Email, password, first name, last name fields
- Password strength indicator
- Confirm password field
- Link to login page
- Terms & conditions checkbox

#### `frontend-admin/src/routes/accept-invite/+page.svelte`
**Features**:
- Token validation on mount
- Display invited email
- Password setup form
- First name, last name fields
- Automatic login after acceptance

### Updated Components:

#### `frontend-admin/src/lib/components/Sidebar.svelte`
**Changes**:
- Show user info at top (avatar, name, email)
- Add logout button
- Hide admin-only items for non-admin users

#### `frontend-admin/src/lib/stores/auth.ts` (NEW)
```typescript
import { writable } from 'svelte/store';

interface User {
  id: string;
  email: string;
  role: 'admin' | 'user';
  first_name: string;
  last_name: string;
}

interface AuthState {
  user: User | null;
  access_token: string | null;
  refresh_token: string | null;
  isAuthenticated: boolean;
}

export const authStore = writable<AuthState>({
  user: null,
  access_token: null,
  refresh_token: null,
  isAuthenticated: false,
});

export function login(user: User, access_token: string, refresh_token: string) {
  authStore.set({
    user,
    access_token,
    refresh_token,
    isAuthenticated: true,
  });
  localStorage.setItem('access_token', access_token);
  localStorage.setItem('refresh_token', refresh_token);
}

export function logout() {
  authStore.set({
    user: null,
    access_token: null,
    refresh_token: null,
    isAuthenticated: false,
  });
  localStorage.removeItem('access_token');
  localStorage.removeItem('refresh_token');
}

export async function refreshAuth() {
  const refresh_token = localStorage.getItem('refresh_token');
  if (!refresh_token) return;

  try {
    const response = await fetch('http://127.0.0.1:3000/api/auth/refresh', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ refresh_token }),
    });

    const data = await response.json();
    if (data.success) {
      localStorage.setItem('access_token', data.data.access_token);
      localStorage.setItem('refresh_token', data.data.refresh_token);
    } else {
      logout();
    }
  } catch {
    logout();
  }
}
```

#### `frontend-admin/src/lib/api.ts` (UPDATE)
Add authentication helper:
```typescript
export function getAuthHeaders(): HeadersInit {
  const token = localStorage.getItem('access_token');
  return {
    'Content-Type': 'application/json',
    ...(token ? { Authorization: `Bearer ${token}` } : {}),
  };
}

export async function authenticatedFetch(url: string, options: RequestInit = {}) {
  const response = await fetch(url, {
    ...options,
    headers: {
      ...getAuthHeaders(),
      ...options.headers,
    },
  });

  if (response.status === 401) {
    // Token expired, try refresh
    await refreshAuth();
    // Retry request
    return fetch(url, {
      ...options,
      headers: {
        ...getAuthHeaders(),
        ...options.headers,
      },
    });
  }

  return response;
}
```

## Phase 8.6: Frontend - Admin User Management

### New Pages:

#### `frontend-admin/src/routes/users/+page.svelte`
**Features**:
- User list table with search/filter
- Role filter (Admin, User)
- Status filter (Active, Inactive)
- User statistics (total, active, etc.)
- Edit user role/status
- Delete user confirmation modal
- Pagination

#### `frontend-admin/src/routes/users/[id]/+page.svelte`
**Features**:
- User profile details
- Activity statistics
- Edit form
- Delete button with confirmation

#### `frontend-admin/src/routes/invitations/+page.svelte`
**Features**:
- Create invitation button with modal
- Invitation list table
- Status badges (Pending, Accepted, Expired)
- Copy invite URL button
- Revoke invitation button
- Expiry countdown for pending invitations

### Sidebar Update:
Add new navigation items (admin-only):
```typescript
{ href: '/users', icon: Users, label: 'User Management' },
{ href: '/invitations', icon: Mail, label: 'Invitations' },
```

## Phase 8.7: Security Enhancements

### Implement:

1. **Rate Limiting**:
   - Login attempts: 5 per 15 minutes per IP
   - Registration: 3 per hour per IP
   - Password reset: 3 per hour per email

2. **Input Validation**:
   - Email format validation
   - Password strength requirements
   - XSS protection (sanitize all inputs)
   - SQL injection protection (use parameterized queries)

3. **CORS Configuration**:
   - Whitelist specific origins
   - Credentials: true for authenticated requests

4. **Security Headers** (using tower-http):
   - X-Frame-Options: DENY
   - X-Content-Type-Options: nosniff
   - Strict-Transport-Security
   - Content-Security-Policy

5. **Password Reset Flow**:
   - POST `/api/auth/forgot-password` (sends reset email)
   - POST `/api/auth/reset-password` (with token)
   - Token expiry: 1 hour

6. **Email Verification** (Optional):
   - Send verification email on registration
   - GET `/api/auth/verify-email/:token`
   - Resend verification email endpoint

## Phase 8.8: Testing

### Unit Tests:
- Password hashing and verification
- JWT token generation and validation
- User repository CRUD operations
- Invitation repository operations

### Integration Tests:
- Registration flow
- Login flow
- Invitation flow (create, accept, expire)
- Protected endpoint access
- Role-based access control

### API Tests with `curl`:
```bash
# Register user
curl -X POST http://127.0.0.1:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"Test123!","first_name":"Test","last_name":"User"}'

# Login
curl -X POST http://127.0.0.1:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"Test123!"}'

# Get current user (with token)
curl -X GET http://127.0.0.1:3000/api/auth/me \
  -H "Authorization: Bearer <access_token>"

# Admin: Create invitation
curl -X POST http://127.0.0.1:3000/api/admin/invitations \
  -H "Authorization: Bearer <admin_token>" \
  -H "Content-Type: application/json" \
  -d '{"email":"newuser@example.com","role":"user","expires_in_days":7}'
```

## Implementation Order

### Sprint 1 (Phase 8.1-8.2):
1. Database migrations
2. Core auth module (password, JWT)
3. User registration endpoint
4. Login endpoint
5. Auth middleware
6. `/api/auth/me` endpoint

### Sprint 2 (Phase 8.3):
1. Invitation database operations
2. Admin invitation endpoints
3. Accept invitation endpoint
4. Token validation endpoint

### Sprint 3 (Phase 8.4):
1. Admin user management endpoints
2. User listing with filters
3. User update/delete endpoints
4. Admin statistics endpoint

### Sprint 4 (Phase 8.5):
1. Login page UI
2. Registration page UI
3. Accept invitation page UI
4. Auth store and API helpers
5. Protected route guards

### Sprint 5 (Phase 8.6):
1. User management page UI
2. User detail page UI
3. Invitations page UI
4. Sidebar updates with user info

### Sprint 6 (Phase 8.7-8.8):
1. Rate limiting implementation
2. Security headers
3. Password reset flow
4. Testing suite

## Configuration

### Environment Variables:
```env
# JWT Configuration
JWT_SECRET=your-secret-key-here-use-strong-random-string
JWT_ACCESS_EXPIRY=900          # 15 minutes in seconds
JWT_REFRESH_EXPIRY=604800      # 7 days in seconds

# Admin Seed
ADMIN_EMAIL=admin@example.com
ADMIN_PASSWORD=AdminPass123!
ADMIN_FIRST_NAME=Admin
ADMIN_LAST_NAME=User

# Email (for invitations - Phase 8.9 optional)
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USER=your-email@gmail.com
SMTP_PASSWORD=your-app-password
SMTP_FROM=noreply@yourdomain.com
```

### Admin Seed Script:
Create initial admin user via migration or CLI command:
```rust
// src/bin/seed_admin.rs
#[tokio::main]
async fn main() -> Result<()> {
    let db_pool = /* connect to DB */;
    let user_repo = UserRepository::new(db_pool);

    let email = env::var("ADMIN_EMAIL")?;
    let password = env::var("ADMIN_PASSWORD")?;
    let password_hash = hash_password(&password).await?;

    user_repo.create_user(
        &email,
        &password_hash,
        &env::var("ADMIN_FIRST_NAME")?,
        &env::var("ADMIN_LAST_NAME")?,
        UserRole::Admin,
    ).await?;

    println!("Admin user created: {}", email);
    Ok(())
}
```

## Success Criteria

- [ ] Users can self-register with email/password
- [ ] Users can login and receive JWT tokens
- [ ] Protected endpoints require valid tokens
- [ ] Admins can invite new members via email
- [ ] Invited users can accept invitations and create accounts
- [ ] Admins can view/manage all users
- [ ] Admins can view/manage all invitations
- [ ] Role-based access control working (Admin vs User)
- [ ] Frontend login/registration UI functional
- [ ] Frontend admin user management UI functional
- [ ] Token refresh mechanism working
- [ ] Rate limiting preventing abuse
- [ ] All security headers configured
- [ ] Password hashing using argon2
- [ ] Unit and integration tests passing

## Future Enhancements (Phase 8.9+)

- Email verification for new registrations
- Password reset via email
- Two-factor authentication (TOTP)
- OAuth integration (Google, GitHub)
- API key management for programmatic access
- User activity logs
- IP whitelist/blacklist
- Account lockout after failed attempts
- Email notifications for security events
- User sessions management (view/revoke)
