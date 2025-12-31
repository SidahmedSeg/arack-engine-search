# BERT Issue - ROOT CAUSE FOUND ✅

**Date:** December 23, 2025
**Time:** 17:30 UTC

---

## 🔍 ROOT CAUSE IDENTIFIED

### The Problem

When we recreated the search-service container with `docker compose up -d`, it used an **OLD Docker image** that doesn't have the BERT model.

### Evidence

**Dockerfile.search (VPS):**
```dockerfile
# Build Stage - Downloads BERT model
RUN apt-get update && apt-get install -y python3 python3-pip && \
    pip3 install huggingface-hub --break-system-packages && \
    python3 -c "from huggingface_hub import snapshot_download; \
    snapshot_download(repo_id='sentence-transformers/all-MiniLM-L6-v2', \
    cache_dir='/root/.cache/huggingface/hub')" && \
    rm -rf /var/lib/apt/lists/*

# Runtime Stage - Copies BERT model from builder
COPY --from=builder /root/.cache/huggingface /root/.cache/huggingface
```

**Container Status:**
- Dockerfile modified: Dec 23, 00:48 (today)
- Container: Using old image WITHOUT BERT model
- Error: `Model cache directory not found`

**What Happened:**
1. Dockerfile.search was updated to download BERT during build ✅
2. But Docker image was NEVER rebuilt ❌
3. `docker compose up -d` recreates container but uses existing image ❌
4. Old image doesn't have BERT → crash loop ❌

---

## ✅ THE FIX: Rebuild Docker Image

### Step-by-Step Fix

**1. Stop the crashing container**
```bash
docker compose -f /opt/arack/docker-compose.production.yml stop search-service
```

**2. Rebuild the Docker image (includes BERT download)**
```bash
docker compose -f /opt/arack/docker-compose.production.yml build --no-cache search-service
```

**Why `--no-cache`:**
- Forces fresh download of BERT model
- Ensures all steps run from scratch
- Takes ~5-10 minutes (downloads ~150MB model)

**3. Start the service with new image**
```bash
docker compose -f /opt/arack/docker-compose.production.yml up -d search-service
```

**4. Verify BERT model exists in new container**
```bash
docker exec arack_search_service ls -la /root/.cache/huggingface/hub/models--sentence-transformers--all-MiniLM-L6-v2/
```

**5. Check service started successfully**
```bash
docker logs arack_search_service --tail 30
# Should see: "BERT embedder initialized successfully"
```

**6. Test search functionality**
```bash
curl http://localhost:3000/health
curl "http://localhost:3000/api/search?q=test"
```

---

## 📊 EXPECTED BUILD TIME

| Step | Duration | Notes |
|------|----------|-------|
| Rust dependencies | 2-3 min | Cargo fetch + compile |
| BERT model download | 2-3 min | ~150MB from HuggingFace |
| Binary compilation | 3-4 min | Release mode build |
| **Total** | **7-10 min** | One-time rebuild |

---

## ✅ SUCCESS CRITERIA

After rebuild:
- ✅ Image contains BERT model at `/root/.cache/huggingface/`
- ✅ Search service starts without errors
- ✅ Logs show: "BERT embedder initialized successfully"
- ✅ Logs show: "Connected to Meilisearch"
- ✅ Health endpoint returns 200
- ✅ Search API returns results
- ✅ Admin dashboard loads

---

## 🔒 WHY THIS HAPPENED

**Timeline:**
1. **Dec 23, 00:48** - Dockerfile.search updated with BERT download
2. **Dec 23, 16:25** - Meilisearch container created (Meilisearch key issue)
3. **Dec 23, 17:20** - Fixed Meilisearch key, recreated search-service
4. **Issue:** `docker compose up -d` used old image (no BERT)

**Missing Step:** Never ran `docker compose build` after Dockerfile update

---

## 📝 COMMANDS TO RUN

```bash
# SSH to VPS
ssh -i ~/.ssh/id_rsa_arack root@213.199.59.206

# Navigate to project directory
cd /opt/arack

# Stop search service
docker compose -f docker-compose.production.yml stop search-service

# Rebuild image with BERT model (takes 7-10 minutes)
docker compose -f docker-compose.production.yml build --no-cache search-service

# Start service with new image
docker compose -f docker-compose.production.yml up -d search-service

# Wait for startup (30 seconds)
sleep 30

# Verify BERT model exists
docker exec arack_search_service ls -la /root/.cache/huggingface/hub/

# Check logs
docker logs arack_search_service --tail 50

# Test health
curl http://localhost:3000/health

# Test search
curl "http://localhost:3000/api/search?q=test&limit=5"
```

---

## ⚠️ NOTES

1. **Build time:** 7-10 minutes (one-time)
2. **Downtime:** Search service will be down during rebuild
3. **Meilisearch:** Still working (already fixed)
4. **Other services:** Unaffected
5. **BERT model:** Will be baked into image (persists across container recreations)

---

## 🎯 AFTER FIX

Once rebuilt:
- ✅ BERT model permanently in Docker image
- ✅ Container recreations won't lose model
- ✅ Keyword search working (Meilisearch)
- ✅ Semantic search working (BERT + Qdrant)
- ✅ Admin dashboard working
- ✅ All search features restored

---

## ✅ READY TO EXECUTE

This is the correct fix. The Dockerfile already has BERT download configured, we just need to rebuild the image.

**Shall I proceed with the rebuild?**
