# Kratos Migration - Quick Start

**Goal**: Migrate search users from custom auth to Ory Kratos

**Time**: 3-5 days

**Status**: Ready to start ✅

---

## Before You Begin

✅ **Prerequisites Met**:
- Kratos is running (check: `docker ps | grep kratos`)
- Kratos DB exists (check: `psql -h localhost -p 5434 -U postgres -l | grep kratos_db`)
- Identity schema configured (`ory/kratos/identity.schema.json`)
- Migration script ready (`scripts/migrate_users_to_kratos.sh`)

---

## Quick Migration Path

### Option 1: Full Migration (Recommended)

Follow the complete guide: **[KRATOS_MIGRATION_GUIDE.md](./KRATOS_MIGRATION_GUIDE.md)**

**Timeline**:
- Day 1: Data migration + Backend (Phase 1-3)
- Day 2: Frontend integration (Phase 4)
- Day 3: Testing (Phase 5)
- Day 4-5: Deployment + Monitoring

---

### Option 2: Test First (Low Risk)

Test migration without touching production code:

#### Step 1: Backup Data
```bash
# Backup users table
pg_dump -h localhost -p 5434 -U postgres -t users engine_search > backup_users.sql
```

#### Step 2: Configure Kratos Hasher

**Important**: We use Argon2id, Kratos defaults to bcrypt.

Edit `ory/kratos/kratos.yml`:
```yaml
hashers:
  algorithm: argon2  # Change from bcrypt
  argon2:
    memory: 131072   # 128 MB
    iterations: 3
    parallelism: 4
    salt_length: 16
    key_length: 32
```

Restart Kratos:
```bash
docker-compose restart kratos
```

#### Step 3: Run Test Migration
```bash
# Check how many users will be migrated
psql -h localhost -p 5434 -U postgres -d engine_search -c \
  "SELECT COUNT(*) FROM users WHERE role = 'user' AND is_active = true;"

# Run migration script
./scripts/migrate_users_to_kratos.sh
```

#### Step 4: Verify Migration
```bash
# Check migrated users in Kratos
curl -s http://127.0.0.1:4434/admin/identities | jq '.[] | {email: .traits.email, id: .id}'

# Test login with Kratos directly (browser)
open http://127.0.0.1:4433/self-service/login/browser
```

#### Step 5: Test Login with Existing User
```bash
# Get login flow
FLOW=$(curl -s http://127.0.0.1:4433/self-service/login/api | jq -r '.id')

# Submit login (replace with real user email/password)
curl -X POST "http://127.0.0.1:4433/self-service/login?flow=$FLOW" \
  -H "Content-Type: application/json" \
  -d '{
    "method": "password",
    "identifier": "user@example.com",
    "password": "their-original-password"
  }' \
  -c cookies.txt | jq .

# If successful, you'll see session data
```

✅ **If test login works**: Migration successful! Users can login with original passwords.

❌ **If test login fails**: Check password hash algorithm in `kratos.yml`.

---

## Migration Decision Matrix

| Current State | Action | Reason |
|---------------|--------|--------|
| **0-10 users** | Full migration today | Low risk, high benefit |
| **11-100 users** | Test first, then migrate | Medium risk, plan carefully |
| **100+ users** | Staged migration | High stakes, need rollback plan |

---

## What Changes for Users?

| Before (Custom Auth) | After (Kratos) | Impact |
|---------------------|----------------|--------|
| Email + Password login | Email + Password login | ✅ Same |
| No password reset | Password reset available | 🎁 New feature |
| No email verification | Email verification (optional) | 🎁 New feature |
| No 2FA | 2FA available (TOTP/WebAuthn) | 🎁 New feature |
| Sessions in DB | Sessions in Kratos | ⚠️ Re-login required |

**User Impact**: Near zero - they login the same way, but get new features!

---

## What Changes for Developers?

### Before (Custom Auth)
```typescript
// Login
const response = await axios.post('/api/auth/login', { email, password });
```

### After (Kratos)
```typescript
// Initialize flow
const flow = await initLoginFlow();

// Submit login
const session = await submitLogin(flow.id, email, password);
```

**Developer Impact**: Medium - Need to update frontend auth pages (see guide Phase 4)

---

## Quick Commands Reference

### Check Kratos Status
```bash
curl -s http://127.0.0.1:4433/health/ready | jq .
# Expected: {"status":"ok"}
```

### View Migrated Users
```bash
curl -s http://127.0.0.1:4434/admin/identities | jq '.[] | {email: .traits.email, id: .id}'
```

### Test Whoami (Check Session)
```bash
curl -s http://127.0.0.1:4433/sessions/whoami -b cookies.txt | jq .
```

### Delete Test User (Rollback)
```bash
# Get user ID
USER_ID=$(curl -s http://127.0.0.1:4434/admin/identities | jq -r '.[0].id')

# Delete user
curl -X DELETE "http://127.0.0.1:4434/admin/identities/$USER_ID"
```

---

## Risk Assessment

### ✅ Low Risk Items
- User data migration (reversible via backup)
- Kratos configuration (can rollback docker-compose)
- Backend API changes (git revert available)

### ⚠️ Medium Risk Items
- Frontend auth pages (save originals before editing)
- Session migration (users need to re-login once)

### ❌ High Risk Items
- **NONE** - All changes are reversible!

---

## Rollback Plan

If anything goes wrong:

1. **Stop using Kratos routes**:
   ```bash
   git checkout src/api/mod.rs  # Restore custom auth routes
   cargo build --release
   ```

2. **Restore frontend**:
   ```bash
   git checkout frontend-search/src/routes/auth/
   git checkout frontend-search/src/lib/stores/auth.svelte.ts
   ```

3. **Restore database** (if needed):
   ```bash
   psql -h localhost -p 5434 -U postgres -d engine_search < backup_users.sql
   ```

Total rollback time: **< 5 minutes**

---

## Support & Documentation

- **Full Guide**: [KRATOS_MIGRATION_GUIDE.md](./KRATOS_MIGRATION_GUIDE.md)
- **Kratos Docs**: https://www.ory.sh/docs/kratos
- **Kratos APIs**: https://www.ory.sh/docs/kratos/reference/api
- **Community Slack**: https://slack.ory.sh

---

## Next Steps

### Today
1. ✅ Read this quick start
2. ✅ Backup database
3. ✅ Run test migration
4. ✅ Test login with migrated user

### This Week
1. Follow full migration guide
2. Update backend API routes
3. Update frontend auth pages
4. Test all auth flows

### After Migration
1. Configure Hydra for SSO (email/calendar)
2. Enable email verification
3. Enable password recovery
4. Add social login (Google, GitHub)
5. Add 2FA/MFA

---

## Ready to Start?

```bash
# Step 1: Backup
pg_dump -h localhost -p 5434 -U postgres engine_search > backup_$(date +%Y%m%d).sql

# Step 2: Run migration
./scripts/migrate_users_to_kratos.sh

# Step 3: Verify
curl -s http://127.0.0.1:4434/admin/identities | jq length

# Step 4: Read full guide
cat KRATOS_MIGRATION_GUIDE.md
```

Good luck! 🚀
