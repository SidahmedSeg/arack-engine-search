# Cookie Domain Fix - Deployment Guide

## ✅ Changes Applied

**Files Modified:**
- `search/api/mod.rs` - Added `Domain=.arack.io` to session cookies

**Lines Changed:**
- Line 1373: Registration flow cookie (added Domain)
- Line 1429: Login flow cookie (added Domain)

**What Changed:**
```rust
// BEFORE (Missing Domain attribute)
"ory_kratos_session={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=604800"

// AFTER (Added Domain=.arack.io)
"ory_kratos_session={}; Path=/; Domain=.arack.io; HttpOnly; SameSite=Lax; Max-Age=604800"
```

---

## 🚀 Deployment Steps

### Step 1: Build the Project

```bash
cd "/Users/intelifoxdz/RS Projects/Engine_search"

# Build in release mode
cargo build --release --bin search-service
```

**Expected Output:**
```
   Compiling search v0.1.0 (/Users/intelifoxdz/RS Projects/Engine_search)
    Finished `release` profile [optimized] target(s) in X.XXs
```

**If build fails:**
- Check for syntax errors
- Run `cargo check` for detailed error messages
- Verify the changes didn't break anything

---

### Step 2: Test Locally (Optional but Recommended)

```bash
# Start the service locally
cargo run --release --bin search-service

# In another terminal, test the cookie
curl -c /tmp/test_cookie.txt -s "http://127.0.0.1:3000/api/auth/flows/login" > /dev/null
cat /tmp/test_cookie.txt | grep "Domain"
```

**Expected Output:**
```
#HttpOnly_.arack.io	TRUE	/	FALSE	...
```

**Stop the local service:**
```bash
# Press Ctrl+C in the terminal running cargo run
```

---

### Step 3: Deploy to VPS

**Option A: Using Existing Deployment Script (If Available)**

```bash
# If you have a deployment script
./deploy.sh
```

**Option B: Manual Deployment**

```bash
# 1. Package the binary
cd "/Users/intelifoxdz/RS Projects/Engine_search"
tar -czf search-service.tar.gz -C target/release search-service

# 2. Upload to VPS
scp -i ~/.ssh/id_rsa_arack search-service.tar.gz root@213.199.59.206:/tmp/

# 3. SSH to VPS
ssh -i ~/.ssh/id_rsa_arack root@213.199.59.206

# 4. Stop the service
docker stop search_engine_search_service

# 5. Extract new binary
cd /opt/arack
tar -xzf /tmp/search-service.tar.gz

# 6. Start the service
docker start search_engine_search_service

# OR if using docker-compose
docker-compose restart search-service

# 7. Verify service started
docker ps | grep search_service
docker logs search_engine_search_service --tail 20
```

**Option C: Docker Rebuild (If Service is Containerized)**

```bash
ssh -i ~/.ssh/id_rsa_arack root@213.199.59.206

cd /opt/arack

# Rebuild the Docker image
docker-compose build --no-cache search-service

# Restart the service
docker-compose up -d search-service

# Verify
docker logs search_engine_search_service --tail 50
```

---

### Step 4: Verify Deployment

```bash
# SSH to VPS
ssh -i ~/.ssh/id_rsa_arack root@213.199.59.206

# Test the cookie domain
curl -c /tmp/verify_fix.txt -s "https://api.arack.io/api/auth/flows/registration" > /dev/null
cat /tmp/verify_fix.txt

# Check service logs
docker logs search_engine_search_service --tail 50
```

**Expected Output:**
```
#HttpOnly_.arack.io	TRUE	/	FALSE	...	csrf_token_...
```

**If you see `.arack.io` → ✅ SUCCESS**
**If you see `api.arack.io` → ❌ FAILED (binary not updated)**

---

### Step 5: Browser Test

**CRITICAL: Clear ALL cookies before testing!**

1. **Open browser** (Chrome, Firefox, Safari)

2. **Press `Ctrl+Shift+Delete`** (or `Cmd+Shift+Delete` on Mac)

3. **Select:**
   - "Cookies and other site data"
   - Time range: "All time"
   - Click "Clear data"

4. **Close ENTIRE browser** (all windows)

5. **Reopen browser**

6. **Navigate to:** `https://arack.io/auth/login`

7. **Open DevTools (F12)**

8. **Login** with your credentials

9. **Check cookies:**
   - DevTools → Application → Cookies → `https://arack.io`
   - Find: `ory_kratos_session`
   - **Domain should show:** `.arack.io` or `arack.io`

10. **Navigate to:** `https://mail.arack.io`

11. **Check cookies:**
    - DevTools → Application → Cookies → `https://mail.arack.io`
    - **Should see:** Same `ory_kratos_session` cookie

12. **Test API call:**
    - Open Console on `https://mail.arack.io`
    - Run:
    ```javascript
    fetch('https://api-mail.arack.io/api/mail/account/me', {
      credentials: 'include'
    })
    .then(r => r.json())
    .then(console.log)
    ```
    - **Expected:** User account data (NOT "No session cookie found")

---

## 🔍 Troubleshooting

### Problem: Build Fails

**Error:** `error: could not compile search`

**Solution:**
```bash
# Check for syntax errors
cargo check

# Clean and rebuild
cargo clean
cargo build --release --bin search-service
```

### Problem: Cookie Still Shows api.arack.io

**Possible Causes:**
1. **Old binary running** - Service not restarted
2. **Browser cache** - Old cookies not cleared
3. **Wrong endpoint** - Calling old endpoint

**Solutions:**
```bash
# 1. Verify service restarted
docker ps | grep search_service
docker logs search_engine_search_service | tail -20

# 2. Force restart
docker restart search_engine_search_service

# 3. Clear browser cache completely
# Open DevTools → Application → Storage → Clear site data
```

### Problem: Service Won't Start After Deploy

**Check logs:**
```bash
docker logs search_engine_search_service --tail 100
```

**Common issues:**
- Database connection failed
- Port already in use
- Missing environment variables

**Rollback:**
```bash
# Restore previous binary (if you backed it up)
cd /opt/arack
docker-compose down
# Restore old binary
docker-compose up -d
```

### Problem: Cookies Work on api.arack.io but NOT mail.arack.io

**This is normal if:**
- nginx rewrite is not active
- You need BOTH fixes (Rust + nginx)

**Verify nginx rewrite:**
```bash
ssh -i ~/.ssh/id_rsa_arack root@213.199.59.206

# Check nginx config
docker exec arack_nginx cat /etc/nginx/sites-enabled/arack.io.conf | grep -A 2 "proxy_cookie_domain"

# Should show:
# proxy_cookie_domain api.arack.io .arack.io;

# If missing, nginx needs reload
docker exec arack_nginx nginx -s reload
```

---

## 📊 Verification Checklist

### Server-Side Checks

- [ ] Rust project builds successfully
- [ ] Binary deployed to VPS
- [ ] Service restarted
- [ ] Service logs show no errors
- [ ] curl test shows `.arack.io` domain

### Browser Checks

- [ ] All cookies cleared
- [ ] Browser completely restarted
- [ ] Login successful
- [ ] Cookie shows `.arack.io` domain in DevTools
- [ ] Cookie visible on mail.arack.io
- [ ] API calls from mail.arack.io work

### Production Validation

- [ ] Users can login on arack.io
- [ ] Users can access mail.arack.io without re-login
- [ ] Users can access admin.arack.io without re-login
- [ ] No authentication errors in logs
- [ ] Session cookies persist across subdomains

---

## 🔄 Rollback Plan

If the deployment causes issues:

### Quick Rollback

```bash
ssh -i ~/.ssh/id_rsa_arack root@213.199.59.206

# Stop current service
docker stop search_engine_search_service

# Restore backup binary (if you created one)
cd /opt/arack
cp search-service.backup search-service

# Start service
docker start search_engine_search_service

# Verify
docker logs search_engine_search_service --tail 20
```

### Git Rollback

```bash
cd "/Users/intelifoxdz/RS Projects/Engine_search"

# Revert the changes
git checkout search/api/mod.rs

# Rebuild
cargo build --release --bin search-service

# Redeploy old version
# ... follow deployment steps ...
```

### Keep nginx Rewrite as Safety Net

**Even if Rust fix fails**, the nginx `proxy_cookie_domain` should still work for `/self-service/` endpoints.

---

## 📝 Post-Deployment Monitoring

### Day 1: Monitor Closely

```bash
# Watch logs for errors
ssh -i ~/.ssh/id_rsa_arack root@213.199.59.206
docker logs -f search_engine_search_service

# Check cookie domain daily
curl -c /tmp/daily_check.txt -s "https://api.arack.io/api/auth/flows/registration" > /dev/null && \
grep "\.arack\.io" /tmp/daily_check.txt && \
echo "✅ $(date): Cookies OK" || \
echo "❌ $(date): Cookie issue detected"
```

### Week 1: Collect User Feedback

**Ask users:**
- Can you login successfully?
- Can you access mail.arack.io after logging in?
- Do you see any cookie-related errors?

**Monitor analytics:**
- Login success rate
- Session duration
- Cross-subdomain navigation errors

### Month 1: Plan Phase 3

If everything works for 2-4 weeks:
- ✅ Consider Phase 3 (migrate to auth.arack.io)
- ✅ Schedule 3-hour maintenance window
- ✅ Implement proper architecture
- ✅ Remove Rust cookie manipulation code
- ✅ Clean up nginx rewrite rules

---

## 🎯 Success Criteria

### Deployment is Successful When:

✅ **Server-side:**
- Build completes without errors
- Service starts successfully
- curl test shows `Domain=.arack.io`
- No errors in service logs

✅ **Client-side:**
- Browser shows `Domain=.arack.io` in DevTools
- Cookie appears on mail.arack.io
- API calls work from all subdomains
- Users can navigate between subdomains without re-login

✅ **Production:**
- No increase in authentication errors
- No user complaints
- Session cookies persist correctly
- All subdomains accessible

---

## 📞 Support

**If deployment fails:**

1. **Check this guide's Troubleshooting section**
2. **Review server logs:** `docker logs search_engine_search_service`
3. **Verify nginx config:** `docker exec arack_nginx nginx -t`
4. **Test with curl:** `curl -c /tmp/test.txt https://api.arack.io/...`
5. **Rollback if needed** (see Rollback Plan above)

**If browser test fails:**

1. **Clear ALL cookies completely**
2. **Close and reopen browser**
3. **Try incognito/private mode**
4. **Try different browser**
5. **Check Network tab for cookie headers**

---

## 📄 Files Reference

**Modified Files:**
- `search/api/mod.rs` - Cookie domain fix

**Deployment Artifacts:**
- `cookie_domain_fix.patch` - Unified diff patch
- `COOKIE_FIX_DEPLOYMENT.md` - This deployment guide
- `PRODUCTION_APPROACH_FINAL.md` - Strategy documentation

**Related Documentation:**
- `COOKIE_DOMAIN_FIX_PLAN_V2.md` - Original fix plan
- `PRODUCTION_BEST_PRACTICES_RECOMMENDATION.md` - Ory recommendations
- `COOKIE_DOMAIN_DIAGNOSTIC_PLAN.md` - Root cause analysis

---

## ⏱️ Estimated Timeline

| Step | Time | Description |
|------|------|-------------|
| **Build** | 2-5 min | Cargo build --release |
| **Local Test** | 2 min | Optional verification |
| **Deploy to VPS** | 3-5 min | Upload, stop, start service |
| **Server Verify** | 1 min | curl test |
| **Browser Test** | 5 min | Clear cookies, login, verify |
| **Total** | **10-15 min** | End-to-end deployment |

---

## ✅ Summary

**What We Fixed:**
- Added `Domain=.arack.io` to registration flow cookies (line 1373)
- Added `Domain=.arack.io` to login flow cookies (line 1429)

**Result:**
- ✅ Cookies now shared across all `*.arack.io` subdomains
- ✅ Users can navigate between arack.io, mail.arack.io, admin.arack.io
- ✅ No more "No session cookie found" errors

**Next Steps:**
1. Build and deploy (10 min)
2. Test in browser (5 min)
3. Monitor for 1 week
4. Plan Phase 3 migration (auth.arack.io) for later

**Ready to Deploy!** 🚀
