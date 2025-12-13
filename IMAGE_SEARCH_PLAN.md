# Image Search Feature Implementation Plan

## Overview

Add image crawling and search functionality with a dedicated "Images" tab in search results, similar to Google Images.

---

## Feature Scope

### What We'll Build:

1. **Image Extraction During Crawl:**
   - Extract all `<img>` tags from crawled pages
   - Capture image metadata (URL, alt text, title, dimensions)
   - Store source page context

2. **Separate Image Index:**
   - Dedicated Meilisearch index for images
   - Search by alt text, image title, source page content
   - Filter by domain, dimensions, etc.

3. **Image Search Tab (Frontend):**
   - Second tab in search results: "All" | "Images"
   - Grid layout display (like Google Images)
   - Image preview with metadata
   - Link to source page

---

## Architecture Design

### Backend Components

#### 1. Image Extraction (Crawler)

**Location:** `src/crawler/mod.rs` or new `src/crawler/image_extractor.rs`

**What to Extract:**
```rust
pub struct ImageData {
    pub id: Uuid,
    pub image_url: String,        // Full URL of image
    pub source_url: String,        // Page where image was found
    pub alt_text: Option<String>,  // Alt attribute
    pub title: Option<String>,     // Title attribute
    pub width: Option<u32>,        // Width attribute
    pub height: Option<u32>,       // Height attribute
    pub page_title: String,        // Title of source page
    pub page_content: String,      // Context from source page
    pub domain: String,            // Domain of source page
    pub crawled_at: DateTime<Utc>,
}
```

**Implementation:**
```rust
// During page crawling, extract images
pub fn extract_images(html: &str, source_url: &str) -> Vec<ImageData> {
    // Use scraper crate to find all <img> tags
    // Parse attributes: src, alt, title, width, height
    // Convert relative URLs to absolute
    // Filter out small images (tracking pixels, icons)
    // Return image metadata
}
```

**Filters to Apply:**
- Skip images < 100x100 pixels (likely icons/tracking pixels)
- Skip data URIs (base64 encoded images)
- Skip SVGs (optional - can be included)
- Validate image URLs

---

#### 2. Image Index (Meilisearch)

**Index Name:** `images`

**Schema:**
```json
{
  "id": "uuid",
  "image_url": "string",
  "source_url": "string",
  "alt_text": "string",
  "title": "string",
  "width": "number",
  "height": "number",
  "page_title": "string",
  "page_content": "string (truncated to 500 chars)",
  "domain": "string",
  "crawled_at": "timestamp"
}
```

**Searchable Attributes:**
1. `alt_text` (highest priority)
2. `title`
3. `page_title`
4. `page_content`

**Filterable Attributes:**
- `domain`
- `width`
- `height`
- `crawled_at`

**Sortable Attributes:**
- `crawled_at`
- `width`
- `height`

---

#### 3. Database Schema (Optional - PostgreSQL)

**Table:** `indexed_images`

```sql
CREATE TABLE IF NOT EXISTS indexed_images (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    image_url TEXT NOT NULL,
    source_url TEXT NOT NULL,
    alt_text TEXT,
    title TEXT,
    width INTEGER,
    height INTEGER,
    page_title TEXT,
    domain VARCHAR(255),
    crawled_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

    -- Indexes
    CONSTRAINT unique_image_source UNIQUE(image_url, source_url)
);

CREATE INDEX idx_images_domain ON indexed_images(domain);
CREATE INDEX idx_images_crawled_at ON indexed_images(crawled_at DESC);
CREATE INDEX idx_images_dimensions ON indexed_images(width, height);
```

**Purpose:**
- Backup/persistence beyond Meilisearch
- Analytics (most crawled images, image stats)
- Deduplication

---

#### 4. API Endpoints

**New Endpoints:**

```
GET /api/search/images?q=query&limit=20&offset=0
```

**Query Parameters:**
- `q` - Search query (required)
- `limit` - Results per page (default: 20, max: 100)
- `offset` - Pagination offset
- `min_width` - Minimum image width
- `min_height` - Minimum image height
- `domain` - Filter by domain

**Response:**
```json
{
  "success": true,
  "data": {
    "hits": [
      {
        "id": "uuid",
        "image_url": "https://example.com/image.jpg",
        "source_url": "https://example.com/page",
        "alt_text": "Beautiful sunset",
        "title": "Sunset photo",
        "width": 1920,
        "height": 1080,
        "page_title": "Photography Blog",
        "domain": "example.com",
        "crawled_at": "2025-12-12T15:30:00Z"
      }
    ],
    "total_hits": 145,
    "query": "sunset",
    "processing_time_ms": 12
  }
}
```

---

### Frontend Components

#### 1. Image Search Tab

**Location:** `frontend-search/src/routes/search/+page.svelte`

**UI Design:**
```
┌─────────────────────────────────────┐
│  [All]  [Images]                    │ ← Tabs
├─────────────────────────────────────┤
│                                     │
│  ┌────┐ ┌────┐ ┌────┐ ┌────┐      │
│  │img │ │img │ │img │ │img │      │ ← Grid layout
│  └────┘ └────┘ └────┘ └────┘      │
│  ┌────┐ ┌────┐ ┌────┐ ┌────┐      │
│  │img │ │img │ │img │ │img │      │
│  └────┘ └────┘ └────┘ └────┘      │
│                                     │
└─────────────────────────────────────┘
```

**Image Card (Hover):**
```
┌────────────────────────┐
│                        │
│      [Image Preview]   │
│                        │
├────────────────────────┤
│ Alt: "Beautiful photo" │
│ 1920x1080             │
│ example.com           │
└────────────────────────┘
```

---

#### 2. Image Grid Component

**File:** `frontend-search/src/lib/components/ImageGrid.svelte`

```svelte
<script lang="ts">
  import { ImageOff } from 'lucide-svelte';

  interface ImageResult {
    id: string;
    image_url: string;
    source_url: string;
    alt_text?: string;
    width?: number;
    height?: number;
    domain: string;
  }

  let { results = $bindable([]) }: { results: ImageResult[] } = $props();

  function handleImageError(e: Event) {
    // Handle broken images
    const img = e.target as HTMLImageElement;
    img.src = '/placeholder-image.svg';
  }

  function openSource(url: string) {
    window.open(url, '_blank');
  }
</script>

<div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4">
  {#each results as image}
    <div class="group relative aspect-square overflow-hidden rounded-lg bg-gray-100 cursor-pointer">
      <img
        src={image.image_url}
        alt={image.alt_text || 'Image'}
        class="w-full h-full object-cover group-hover:scale-105 transition-transform"
        onerror={handleImageError}
        onclick={() => openSource(image.source_url)}
      />

      <!-- Hover overlay -->
      <div class="absolute inset-0 bg-black bg-opacity-0 group-hover:bg-opacity-60 transition-opacity flex items-end">
        <div class="p-3 text-white opacity-0 group-hover:opacity-100 transition-opacity">
          <p class="text-sm font-medium truncate">{image.alt_text || 'No description'}</p>
          <p class="text-xs text-gray-300">{image.width}x{image.height} • {image.domain}</p>
        </div>
      </div>
    </div>
  {/each}
</div>
```

---

#### 3. Tab Switcher

**Add to search page:**

```svelte
<script>
  let activeTab = $state<'all' | 'images'>('all');

  async function switchTab(tab: 'all' | 'images') {
    activeTab = tab;
    if (tab === 'images') {
      await performImageSearch();
    } else {
      await performSearch();
    }
  }
</script>

<!-- Tab Navigation -->
<div class="flex gap-6 border-b border-gray-200 mb-6">
  <button
    onclick={() => switchTab('all')}
    class="pb-3 px-1 border-b-2 {activeTab === 'all' ? 'border-primary text-primary' : 'border-transparent text-gray-600'}"
  >
    All
  </button>
  <button
    onclick={() => switchTab('images')}
    class="pb-3 px-1 border-b-2 {activeTab === 'images' ? 'border-primary text-primary' : 'border-transparent text-gray-600'}"
  >
    Images
  </button>
</div>

<!-- Results -->
{#if activeTab === 'all'}
  <!-- Regular search results -->
{:else}
  <ImageGrid results={imageResults} />
{/if}
```

---

## Implementation Phases

### Phase 1: Backend - Image Extraction (2-3 hours)

**Tasks:**
1. Create `ImageExtractor` module
2. Extract images during crawling
3. Filter out small/invalid images
4. Store in database (optional)

**Files to Modify:**
- `src/crawler/mod.rs` - Add image extraction call
- Create `src/crawler/image_extractor.rs` - New module
- Create `migrations/003_create_images_table.sql` - Optional

---

### Phase 2: Backend - Image Indexing (1-2 hours)

**Tasks:**
1. Create Meilisearch `images` index
2. Configure search settings
3. Index extracted images

**Files to Modify:**
- `src/search/mod.rs` - Add image index initialization
- `src/worker/mod.rs` - Add image indexing after page crawl

---

### Phase 3: Backend - Image Search API (1 hour)

**Tasks:**
1. Create image search endpoint
2. Add query parameters and filters
3. Test endpoint

**Files to Modify:**
- `src/api/mod.rs` - Add `/api/search/images` route
- Add handler function

---

### Phase 4: Frontend - API Client (30 minutes)

**Tasks:**
1. Add image search method to API client
2. Add TypeScript types

**Files to Modify:**
- `shared/api-client/index.ts`
- `shared/types/index.ts`

---

### Phase 5: Frontend - Image Tab UI (2-3 hours)

**Tasks:**
1. Create ImageGrid component
2. Add tab switcher to search page
3. Implement image search logic
4. Style image grid (responsive)

**Files to Create:**
- `frontend-search/src/lib/components/ImageGrid.svelte`

**Files to Modify:**
- `frontend-search/src/routes/search/+page.svelte`

---

### Phase 6: Testing & Polish (1 hour)

**Tasks:**
1. Test image extraction on various sites
2. Test broken image handling
3. Test pagination
4. Mobile responsive testing
5. Performance optimization (lazy loading)

---

## Technical Considerations

### 1. Image Loading Performance

**Challenges:**
- Loading many images can be slow
- Large images consume bandwidth

**Solutions:**
- Implement lazy loading (only load visible images)
- Use image thumbnails/proxies (optional)
- Add loading skeletons
- Limit results per page (20-30 images)

### 2. Broken/Invalid Images

**Challenges:**
- Some image URLs may be broken
- Images may require authentication
- CORS issues in browser

**Solutions:**
- Show placeholder for broken images
- Handle `onerror` event
- Test image accessibility during crawling (optional)

### 3. Image Deduplication

**Challenge:**
- Same image may appear on multiple pages

**Solution:**
- Use image URL as unique identifier
- Store all source pages for each image
- Show "Found on X pages" in UI

### 4. Storage Considerations

**Images in Database:**
- Store metadata only (URLs, not actual image files)
- ~1KB per image record
- 10,000 images = ~10MB database

**Images in Meilisearch:**
- Similar size to database
- Faster search
- Auto-expiring (can rebuild from database)

### 5. Legal/Copyright Considerations

**Important:**
- Only index publicly accessible images
- Respect robots.txt for images
- Add image attribution (source URL)
- Don't download/store actual image files (just link to them)
- Include "Report Image" feature (optional)

---

## Estimated Time

**Total Implementation Time:** 8-12 hours

**Breakdown:**
- Backend (Image extraction + indexing): 4-5 hours
- Backend (API endpoints): 1 hour
- Frontend (UI components): 3-4 hours
- Testing & polish: 2 hours

---

## Expected Results

### Search Query: "nature"

**All Tab:**
- Shows text results about nature

**Images Tab:**
```
┌────────────────────────────────────────────────┐
│  Showing 1-20 of 145 images for "nature"       │
├────────────────────────────────────────────────┤
│                                                │
│   🖼️    🖼️    🖼️    🖼️    🖼️               │
│   🖼️    🖼️    🖼️    🖼️    🖼️               │
│   🖼️    🖼️    🖼️    🖼️    🖼️               │
│   🖼️    🖼️    🖼️    🖼️    🖼️               │
│                                                │
│              [Previous] [Next]                 │
└────────────────────────────────────────────────┘
```

### Success Metrics

**Per Crawl:**
- Extract 20-50 images per page (average)
- 100 pages crawled = 2,000-5,000 images indexed

**Search Performance:**
- Image search < 50ms (similar to text search)
- Grid loads in < 2 seconds (with lazy loading)

**User Experience:**
- Google Images-like interface
- Smooth scrolling and hover effects
- Mobile-friendly grid layout

---

## Alternative Approaches

### Option 1: External Image Search API (Easier)

Instead of crawling images yourself, integrate with:
- Google Custom Search API (Images)
- Bing Image Search API
- Unsplash API (for free stock photos)

**Pros:**
- No crawling needed
- Better image quality
- Legal compliance handled

**Cons:**
- Requires API keys
- May have usage limits
- Less control over results

### Option 2: Delayed Implementation

**Phase 1:** Basic image extraction (store metadata only)
**Phase 2:** Add search later when you have more images

### Option 3: Simple Image Viewer

Instead of separate tab:
- Show images inline with text results
- Add "View Images" button per result
- Simpler implementation (1-2 hours)

---

## Recommendation

**Start with Full Implementation** because:

1. ✅ You already have the crawler infrastructure
2. ✅ Meilisearch can handle image metadata easily
3. ✅ Adds significant value to your search engine
4. ✅ Differentiates from basic text search
5. ✅ Not technically complex (8-12 hours work)

**Next Steps:**
1. Review this plan
2. Decide if you want to proceed
3. I can start with Phase 1 (Backend image extraction)
4. Iterate and test each phase

---

## Questions to Consider

1. **Scope:** Do you want to implement all phases or start with basics?
2. **Database:** Should we store images in PostgreSQL or just Meilisearch?
3. **Filtering:** Do you want size filters (e.g., "Large images only")?
4. **Mobile:** How should image grid look on mobile? (2 columns vs 3)
5. **Preview:** Should clicking an image show a modal preview or go to source?

Let me know if you want to proceed with implementation! 🚀
