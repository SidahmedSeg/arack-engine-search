# Arack.io - Staging Deployment Plan

## 📋 Overview

**Domain**: arack.io
**Server IP**: 213.199.59.206
**OS**: Ubuntu 22.04/24.04
**Environment**: Staging/Production

---

## 🌐 Service Architecture & Subdomains

### Frontend Applications
| Service | URL | Port | Description |
|---------|-----|------|-------------|
| Search Frontend | https://arack.io | 5001 | Main search engine interface |
| Email Frontend | https://mail.arack.io | 5006 | Email application (webmail) |
| Admin Dashboard | https://admin.arack.io | 5000 | Crawler management & analytics |

### Backend APIs
| Service | URL | Port | Description |
|---------|-----|------|-------------|
| Search API | https://api.arack.io | 3000 | Search & crawler backend (Rust) |
| Email API | https://api-mail.arack.io | 3001 | Email backend (Rust) |

### Authentication & SSO
| Service | URL | Port | Description |
|---------|-----|------|-------------|
| Kratos (Public) | https://auth.arack.io | 4433 | Identity management (login/register) |
| Kratos (Admin) | Internal | 4434 | Admin API (internal only) |
| Hydra (Public) | https://oauth.arack.io | 4444 | OAuth2/OIDC provider |
| Hydra (Admin) | Internal | 4445 | Admin API (internal only) |

### Real-time & WebSocket
| Service | URL | Port | Description |
|---------|-----|------|-------------|
| Centrifugo | wss://ws.arack.io | 8001 | WebSocket for email real-time updates |

### Email Server (Stalwart)
| Service | URL | Port | Description |
|---------|-----|------|-------------|
| SMTP | smtp.arack.io | 25, 587 | Email sending (SMTP/Submission) |
| IMAP | imap.arack.io | 143, 993 | Email retrieval (IMAP/IMAPS) |
| JMAP HTTP API | Internal | 8080 | JMAP protocol (internal only) |

### Infrastructure (Internal)
| Service | Port | Description |
|---------|------|-------------|
| PostgreSQL | 5432 | Database (internal only) |
| Redis | 6379 | Cache & job queue (internal only) |
| Meilisearch | 7700 | Search indexing (internal only) |
| Qdrant | 6333 | Vector database (internal only) |

---

## 📦 Deployment Phases

### Phase 1: Server Preparation (30-45 min)
**Tasks:**
1. ✅ Connect to VPS via SSH
2. ✅ Update system packages (`apt update && apt upgrade`)
3. ✅ Install Docker & Docker Compose
4. ✅ Install Git
5. ✅ Configure firewall (UFW)
6. ✅ Create deployment user (optional, can use root)
7. ✅ Clone repository from GitHub

**Success Criteria:**
- Docker version 24+ installed
- Docker Compose version 2.20+ installed
- Repository cloned to `/opt/arack` or similar

---

### Phase 2: SSL/TLS Setup (15-20 min)
**Tasks:**
1. ✅ Install Certbot
2. ✅ Obtain wildcard SSL certificate for `*.arack.io`
3. ✅ Configure auto-renewal

**Commands:**
```bash
certbot certonly --standalone -d arack.io -d *.arack.io \
  --email admin@arack.io --agree-tos --non-interactive
```

**Success Criteria:**
- SSL certificate at `/etc/letsencrypt/live/arack.io/`
- Auto-renewal cron job configured

---

### Phase 3: Environment Configuration (20 min)
**Tasks:**
1. ✅ Copy `.env.production.example` to `.env.production`
2. ✅ Generate secure secrets for:
   - PostgreSQL password
   - Redis password
   - Meilisearch master key
   - Qdrant API key
   - Kratos cookie/cipher secrets
   - Hydra system secret
   - Stalwart admin password
   - Centrifugo HMAC secret & API key
3. ✅ Configure OpenAI API key
4. ✅ Update all URLs to production domains

**Example Secrets Generation:**
```bash
# PostgreSQL password
openssl rand -base64 32

# Kratos secrets (32 chars)
openssl rand -base64 32 | head -c 32

# All other secrets
openssl rand -hex 32
```

**Success Criteria:**
- `.env.production` file created with all secrets
- No placeholder values remaining

---

### Phase 4: Frontend Environment Setup (10 min)
**Tasks:**
1. ✅ Update `frontend-search/.env`
   ```env
   VITE_API_URL=https://api.arack.io
   VITE_AUTH_URL=https://auth.arack.io
   ```

2. ✅ Update `frontend-email/.env`
   ```env
   VITE_EMAIL_API_URL=https://api-mail.arack.io
   VITE_CENTRIFUGO_URL=wss://ws.arack.io/connection/websocket
   ```

3. ✅ Update `frontend-admin/.env`
   ```env
   VITE_API_URL=https://api.arack.io
   ```

**Success Criteria:**
- All frontend env files point to production URLs
- WebSocket uses `wss://` (secure WebSocket)

---

### Phase 5: Kratos Configuration Update (10 min)
**Tasks:**
1. ✅ Update `ory/kratos/kratos.yml`:
   ```yaml
   serve:
     public:
       base_url: https://auth.arack.io
       cors:
         enabled: true
         allowed_origins:
           - https://arack.io
           - https://mail.arack.io
           - https://admin.arack.io

   selfservice:
     default_browser_return_url: https://arack.io
     allowed_return_urls:
       - https://arack.io
       - https://mail.arack.io
       - https://admin.arack.io
   ```

**Success Criteria:**
- Kratos allows CORS from all frontend domains
- Redirect URLs point to production

---

### Phase 6: Docker Build & Deploy (30-45 min)
**Tasks:**
1. ✅ Build Docker images:
   ```bash
   docker compose -f docker-compose.production.yml build
   ```

2. ✅ Start infrastructure services:
   ```bash
   docker compose -f docker-compose.production.yml up -d postgres redis meilisearch qdrant
   ```

3. ✅ Wait for health checks (30 seconds)

4. ✅ Run database migrations:
   ```bash
   docker compose -f docker-compose.production.yml up -d kratos-migrate
   ```

5. ✅ Start application services:
   ```bash
   docker compose -f docker-compose.production.yml up -d
   ```

6. ✅ Verify all containers running:
   ```bash
   docker ps
   ```

**Success Criteria:**
- All containers in "healthy" or "running" state
- No containers in "restarting" loop
- Logs show no critical errors

---

### Phase 7: Frontend Deployment (15 min)
**Tasks:**
1. ✅ Build frontend applications:
   ```bash
   cd frontend-search && npm run build
   cd ../frontend-email && npm run build
   cd ../frontend-admin && npm run build
   ```

2. ✅ Run frontend servers (PM2 or systemd):
   ```bash
   # Option 1: PM2
   pm2 start ecosystem.config.js

   # Option 2: Systemd services
   systemctl enable arack-search-frontend
   systemctl enable arack-email-frontend
   systemctl enable arack-admin-frontend
   ```

**Success Criteria:**
- All frontends accessible on their ports
- Frontend processes managed by process manager

---

### Phase 8: Nginx Configuration (10 min)
**Tasks:**
1. ✅ Copy nginx configs to server
2. ✅ Test nginx configuration:
   ```bash
   docker exec arack_nginx nginx -t
   ```

3. ✅ Reload nginx:
   ```bash
   docker restart arack_nginx
   ```

**Success Criteria:**
- Nginx configuration valid
- All domains return 200 OK
- SSL certificates loaded

---

### Phase 9: DNS Configuration (15-20 min)
**Tasks:**

#### A Records (Point to 213.199.59.206)
```
arack.io                → 213.199.59.206
www.arack.io            → 213.199.59.206
mail.arack.io           → 213.199.59.206
admin.arack.io          → 213.199.59.206
api.arack.io            → 213.199.59.206
api-mail.arack.io       → 213.199.59.206
auth.arack.io           → 213.199.59.206
oauth.arack.io          → 213.199.59.206
ws.arack.io             → 213.199.59.206
smtp.arack.io           → 213.199.59.206
imap.arack.io           → 213.199.59.206
```

#### MX Record (Email routing)
```
arack.io → smtp.arack.io (priority 10)
```

#### TXT Records (Email security)
```
# SPF (Sender Policy Framework)
arack.io → v=spf1 mx -all

# DMARC (Domain-based Message Authentication)
_dmarc.arack.io → v=DMARC1; p=quarantine; rua=mailto:dmarc-reports@arack.io

# DKIM (DomainKeys Identified Mail) - Generated by Stalwart
default._domainkey.arack.io → [Generated key from Stalwart]
```

**DNS Propagation**: Wait 5-30 minutes for DNS to propagate globally.

**Success Criteria:**
- All A records resolve to 213.199.59.206
- MX record points to smtp.arack.io
- TXT records configured correctly

---

### Phase 10: Email DKIM Setup (10 min)
**Tasks:**
1. ✅ Generate DKIM keys in Stalwart:
   ```bash
   docker exec arack_stalwart stalwart-cli dkim generate arack.io
   ```

2. ✅ Get public key:
   ```bash
   docker exec arack_stalwart stalwart-cli dkim show arack.io
   ```

3. ✅ Add DKIM TXT record to DNS

**Success Criteria:**
- DKIM public key added to DNS
- Email authentication passes (SPF, DKIM, DMARC)

---

### Phase 11: Testing & Verification (30 min)

#### Frontend Tests
- [ ] https://arack.io loads and search works
- [ ] https://mail.arack.io loads and displays mailboxes
- [ ] https://admin.arack.io loads and shows crawler metrics

#### Authentication Tests
- [ ] User registration works at https://auth.arack.io
- [ ] Login redirects back to frontend
- [ ] Session cookie persists across domains

#### Email Tests
- [ ] Send email from mail.arack.io
- [ ] Receive email at user@arack.io
- [ ] External email delivery works (Gmail, Outlook)
- [ ] SPF/DKIM/DMARC passes (use mail-tester.com)

#### WebSocket Tests
- [ ] Real-time email notifications work
- [ ] Connection status shows "Live" in mail.arack.io

#### SSL Tests
- [ ] All domains serve HTTPS
- [ ] SSL certificate valid (use ssllabs.com)
- [ ] No mixed content warnings

**Success Criteria:**
- All tests passing
- No console errors in browser
- Email deliverability score > 8/10

---

## 🔐 Security Checklist

- [ ] Firewall configured (allow only 22, 80, 443, 25, 587, 143, 993)
- [ ] SSH key authentication enabled (disable password auth)
- [ ] Root login disabled (use sudo user)
- [ ] Fail2ban installed and configured
- [ ] Docker socket not exposed
- [ ] All secrets in `.env.production` are strong (32+ chars)
- [ ] Database backups automated (daily)
- [ ] SSL certificate auto-renewal tested

---

## 📊 Monitoring Setup (Optional)

### Log Aggregation
```bash
# View all service logs
docker compose -f docker-compose.production.yml logs -f

# View specific service
docker compose logs -f email-service
```

### Metrics (Optional - Future)
- Prometheus + Grafana for system metrics
- Uptime monitoring (UptimeRobot, Pingdom)
- Error tracking (Sentry)

---

## 🔄 Backup Strategy

### Daily Automated Backups
```bash
# PostgreSQL backup
docker exec arack_postgres pg_dump -U postgres engine_search > backup_$(date +%Y%m%d).sql

# Volume backup
docker run --rm -v arack_postgres_data:/data -v $(pwd):/backup ubuntu tar czf /backup/postgres_$(date +%Y%m%d).tar.gz /data
```

### Retention Policy
- Daily backups: Keep 7 days
- Weekly backups: Keep 4 weeks
- Monthly backups: Keep 12 months

---

## 🚨 Rollback Plan

If deployment fails:

1. **Stop new containers**:
   ```bash
   docker compose -f docker-compose.production.yml down
   ```

2. **Restore database from backup**:
   ```bash
   docker exec -i arack_postgres psql -U postgres engine_search < backup_YYYYMMDD.sql
   ```

3. **Revert to previous Docker images**:
   ```bash
   docker compose -f docker-compose.production.yml pull
   ```

4. **Restart services**:
   ```bash
   docker compose -f docker-compose.production.yml up -d
   ```

---

## 📝 Post-Deployment Tasks

- [ ] Create initial admin user via Kratos
- [ ] Test user registration flow
- [ ] Create test email account (test@arack.io)
- [ ] Send test emails to Gmail, Outlook
- [ ] Monitor logs for errors (first 24 hours)
- [ ] Setup monitoring alerts
- [ ] Document any issues encountered
- [ ] Update DNS TTL to 1 hour (from default)

---

## 🎯 Success Metrics

**Deployment is successful when:**
- ✅ All services running and healthy
- ✅ All domains accessible via HTTPS
- ✅ User can register and login
- ✅ Search functionality works
- ✅ Email can be sent and received
- ✅ WebSocket real-time updates work
- ✅ Admin dashboard shows metrics
- ✅ SSL certificates valid for 90 days
- ✅ Email deliverability > 8/10

---

## 📞 Support & Troubleshooting

### Common Issues

**Issue**: Container keeps restarting
**Solution**: Check logs with `docker logs <container_name>`

**Issue**: SSL certificate not loading
**Solution**: Verify certbot ran successfully, check nginx config

**Issue**: Emails going to spam
**Solution**: Verify SPF, DKIM, DMARC records, check mail-tester.com score

**Issue**: WebSocket connection fails
**Solution**: Ensure wss:// URL is used, check Centrifugo logs

**Issue**: CORS errors in browser
**Solution**: Update Kratos CORS config, restart Kratos container

---

## 📅 Timeline Estimate

| Phase | Duration | Dependencies |
|-------|----------|--------------|
| 1. Server Preparation | 30-45 min | None |
| 2. SSL Setup | 15-20 min | Phase 1 |
| 3. Environment Config | 20 min | Phase 1 |
| 4. Frontend Env | 10 min | Phase 3 |
| 5. Kratos Config | 10 min | Phase 3 |
| 6. Docker Deploy | 30-45 min | Phase 2, 3 |
| 7. Frontend Deploy | 15 min | Phase 4 |
| 8. Nginx Config | 10 min | Phase 6, 7 |
| 9. DNS Config | 15-20 min | Phase 8 |
| 10. DKIM Setup | 10 min | Phase 9 |
| 11. Testing | 30 min | All phases |

**Total Estimated Time**: 3-4 hours (single developer)

---

## 🤖 What Claude Needs for VPS Access

To work on the VPS, I can execute commands via SSH. Here's what I can do:

### ✅ I Can Do:
- Execute bash commands via SSH
- Copy files to/from the server
- Edit configuration files
- Run Docker commands
- Monitor logs and services
- Install packages
- Configure services

### ❌ I Cannot Do:
- Open interactive shells (like `vim`, `nano` interactively)
- Use GUI tools
- Handle prompts that require real-time interaction
- Execute commands that require TTY (unless using workarounds)

### 🔧 What You Need to Provide:
**Already Provided ✅:**
- Server IP: 213.199.59.206
- User: root
- Password: c1lFXao11rF4vAa1310N
- OS: Ubuntu

**Additional Info Helpful:**
- Domain registrar access (for DNS configuration) - I'll provide exact DNS records to add
- OpenAI API key (for AI features) - You can provide this when we configure `.env.production`

### 🎯 How I'll Work:
1. Execute commands via SSH using the Bash tool
2. Show you the output of each command
3. Ask for confirmation before critical operations (like database migrations)
4. Provide clear explanations of what each step does

**Example SSH command I'll run:**
```bash
ssh root@213.199.59.206 "cd /opt/arack && docker compose -f docker-compose.production.yml up -d"
```

---

## ✅ Ready to Deploy?

**Before we start, please confirm:**
1. You have access to arack.io DNS settings (to add A, MX, TXT records)
2. You have an OpenAI API key for AI features (or we can skip AI initially)
3. You're ready for me to start executing commands on the VPS

**Once confirmed, I'll proceed with Phase 1: Server Preparation** 🚀
