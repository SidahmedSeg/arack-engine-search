# Authentication Guide

## Overview

This application uses **session-based authentication** with PostgreSQL-backed sessions. User registration is **invitation-only** by design - this is an admin dashboard, not a public application.

## Authentication Flow

1. **Admin Login** → Session-based authentication
2. **Invite Users** → Admin creates invitations
3. **Users Accept** → Invitation link → Create account → Auto-login
4. **No Public Registration** → Security by design

## Initial Setup - Creating Your First Admin

### Step 1: Run Database Migrations

First, ensure your database migrations are up to date:

```bash
# The migrations will run automatically when you start the server
cargo run
```

Or run migrations manually with SQLx CLI:

```bash
sqlx migrate run
```

### Step 2: Create the Initial Admin User

Run the admin seed script:

```bash
cargo run --bin seed_admin
```

This creates a default admin account:
- **Email:** `admin@example.com`
- **Password:** `admin123456`

⚠️ **IMPORTANT:** Change this password immediately after first login!

### Step 3: Login to the Dashboard

1. Navigate to: `http://localhost:5173/login`
2. Enter the default credentials above
3. You'll be redirected to the dashboard

## User Management

### Inviting New Users

As an admin, you can invite users via the API:

```bash
# Create an invitation (admin only)
curl -X POST http://localhost:3000/api/admin/invitations \
  -H "Content-Type: application/json" \
  -b cookies.txt \
  -d '{
    "email": "user@example.com",
    "role": "user",
    "expires_in_days": 7
  }'
```

The response will include a `token`. The invitation link is:
```
http://localhost:5173/invite/{token}
```

Send this link to the user - they'll be able to create their account.

### Invitation Flow

1. **Admin creates invitation** → Receives unique token
2. **Admin sends link** → `http://localhost:5173/invite/{token}`
3. **User clicks link** → Verifies invitation (email, role, expiry)
4. **User fills form** → First name, last name, password
5. **Account created** → User automatically logged in
6. **Invitation marked accepted** → Cannot be reused

## API Endpoints

### Public Endpoints

- `POST /api/auth/login` - Login with email/password
- `POST /api/auth/logout` - Logout current session
- `GET /api/auth/me` - Get current user info
- `GET /api/auth/invitations/:token` - Verify invitation token
- `POST /api/auth/invitations/:token/accept` - Accept invitation

### Admin-Only Endpoints

- `POST /api/admin/invitations` - Create invitation
- `GET /api/admin/invitations` - List all invitations
- `DELETE /api/admin/invitations/:id` - Delete/revoke invitation
- `GET /api/admin/users` - List all users
- `GET /api/admin/users/:id` - Get user details
- `POST /api/admin/users/:id` - Update user
- `DELETE /api/admin/users/:id` - Delete user

## Session Management

- **Storage:** PostgreSQL (via tower-sessions)
- **Expiry:** 7 days of inactivity
- **Cleanup:** Automatic every 60 seconds
- **Cookies:** HttpOnly, Secure (in production)

## Security Features

1. **Argon2id Password Hashing** - OWASP recommended
2. **Session-Based Auth** - Stateful, secure
3. **Invitation-Only Registration** - No public signup
4. **Role-Based Access Control** - Admin vs User
5. **Email Verification** - Ready for future implementation
6. **Password Strength Validation** - Minimum 8 characters

## Testing the Authentication

### 1. Test Login

```bash
# Login
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -c cookies.txt \
  -d '{
    "email": "admin@example.com",
    "password": "admin123456"
  }'
```

### 2. Test Authenticated Request

```bash
# Get current user (requires session)
curl http://localhost:3000/api/auth/me \
  -b cookies.txt
```

### 3. Test Admin Endpoint

```bash
# List users (admin only)
curl http://localhost:3000/api/admin/users \
  -b cookies.txt
```

### 4. Test Logout

```bash
# Logout
curl -X POST http://localhost:3000/api/auth/logout \
  -b cookies.txt
```

## Troubleshooting

### "Not authenticated" Error

- Check that you're including cookies in requests (`-b cookies.txt`)
- Verify session hasn't expired (7 days of inactivity)
- Ensure backend is running on correct port

### "Admin access required" Error

- Check user role: `GET /api/auth/me`
- Only users with `role: "admin"` can access admin endpoints

### Can't Login

- Verify email/password are correct
- Check database has the user: `SELECT * FROM users;`
- Ensure migrations ran successfully

### Invitation Token Invalid

- Check invitation hasn't expired
- Verify token is correct (exact match required)
- Check invitation status isn't already "accepted"

## Re-seeding Admin User

If you need to reset the admin account:

```bash
# Delete existing admin
psql $DATABASE_URL -c "DELETE FROM users WHERE email = 'admin@example.com';"

# Re-run seed
cargo run --bin seed_admin
```

## Production Recommendations

1. **Change default password** immediately
2. **Use environment variables** for sensitive data
3. **Enable HTTPS** for secure cookies
4. **Set strong session secrets**
5. **Implement email verification**
6. **Add password reset functionality**
7. **Enable 2FA** for admin accounts
8. **Monitor failed login attempts**
9. **Implement rate limiting** on auth endpoints
10. **Regular security audits**
