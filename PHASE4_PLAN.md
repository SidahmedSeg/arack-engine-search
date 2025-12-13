# Phase 4: Frontend UI Development - Detailed Plan

**Status**: 📋 Planning
**Goal**: Build two separate web applications - Admin Dashboard and End User Search Interface

---

## Architecture Overview

### Two-App Approach

```
Engine_search/
├── frontend-admin/          # Admin Dashboard (SvelteKit)
│   ├── src/
│   │   ├── routes/
│   │   ├── lib/
│   │   └── app.html
│   └── package.json
│
├── frontend-search/         # End User Search App (SvelteKit)
│   ├── src/
│   │   ├── routes/
│   │   ├── lib/
│   │   └── app.html
│   └── package.json
│
└── shared/                  # Shared utilities and types
    ├── types/
    ├── api-client/
    └── utils/
```

---

## App 1: Admin Dashboard

### Purpose
Internal tool for administrators to manage the search engine operations.

### Target Users
- System administrators
- Content managers
- DevOps team

### Core Features

#### 1. Dashboard Overview (Home Page)
**Route**: `/`

**Components**:
- System health status card
- Quick stats overview:
  - Total indexed documents
  - Active crawl jobs
  - Index size
  - Search queries today
- Recent activity feed
- Quick action buttons (Start Crawl, Clear Index)

**API Calls**:
- `GET /api/health`
- `GET /api/stats`

---

#### 2. Crawl Management
**Route**: `/crawl`

**Features**:
- **Start New Crawl**
  - URL input (single or multiple)
  - Max depth selector
  - Advanced options (respect robots.txt, rate limit)
  - Submit button → triggers `POST /api/crawl`

- **Active Crawls** (Future: requires job tracking API)
  - List of running crawl jobs
  - Progress indicators
  - Pause/Cancel buttons
  - Real-time status updates

- **Crawl History**
  - Table of past crawl jobs
  - Columns: URL, Date, Documents Indexed, Duration, Status
  - Filter by date, URL, status
  - Sortable columns
  - Details view (click to expand)

**Components**:
- `CrawlForm.svelte` - Form to start new crawl
- `ActiveCrawls.svelte` - List of running jobs
- `CrawlHistory.svelte` - Historical crawl data
- `CrawlJobCard.svelte` - Individual job display

**API Calls**:
- `POST /api/crawl` - Start crawl
- `GET /api/crawl/jobs` (Future)
- `GET /api/crawl/:jobId` (Future)
- `DELETE /api/crawl/:jobId` (Future - cancel job)

---

#### 3. Index Management
**Route**: `/index`

**Features**:
- **Index Statistics**
  - Total documents count
  - Field distribution
  - Index size
  - Indexing status
  - Last updated timestamp

- **Index Operations**
  - Clear entire index (with confirmation)
  - Re-index documents (Future)
  - Optimize index (Future)

- **Document Browser**
  - Paginated table of all indexed documents
  - Columns: Title, URL, Word Count, Crawled Date
  - Search within documents
  - View document details
  - Delete individual document (Future)

**Components**:
- `IndexStats.svelte` - Statistics cards
- `IndexOperations.svelte` - Action buttons
- `DocumentBrowser.svelte` - Document list/table
- `DocumentDetails.svelte` - Modal/drawer for details

**API Calls**:
- `GET /api/stats`
- `DELETE /api/index`
- `GET /api/search` (for browsing documents)
- `DELETE /api/documents/:id` (Future)

---

#### 4. Search Testing
**Route**: `/test-search`

**Features**:
- **Search Query Tester**
  - Search input field
  - Advanced filters UI:
    - Limit (slider/input)
    - Offset (pagination)
    - Min/Max word count (range slider)
    - Date range picker
    - Sort field dropdown
    - Sort order toggle
  - Submit button

- **Results Display**
  - Result cards with full details
  - Processing time display
  - Total hits count
  - JSON response viewer (toggleable)
  - Export results (JSON/CSV)

- **Search Analytics** (Future)
  - Popular queries
  - Failed searches
  - Average response time
  - Query volume over time (chart)

**Components**:
- `SearchTester.svelte` - Main search testing interface
- `SearchFilters.svelte` - Advanced filter controls
- `SearchResults.svelte` - Results display
- `ResultCard.svelte` - Individual result
- `JsonViewer.svelte` - Raw JSON display

**API Calls**:
- `GET /api/search` with all parameters

---

#### 5. System Settings (Future)
**Route**: `/settings`

**Features**:
- Crawler configuration
- Meilisearch settings
- API key management
- User management (if auth is added)
- Notification settings

---

### Admin Dashboard UI/UX

#### Design System
- **Color Scheme**: Professional, dashboard-focused
  - Primary: Blue (#3B82F6)
  - Success: Green (#10B981)
  - Warning: Yellow (#F59E0B)
  - Error: Red (#EF4444)
  - Background: Light gray (#F9FAFB)

- **Layout**:
  - Sidebar navigation (collapsible)
  - Top header with user info and notifications
  - Main content area
  - Responsive design (mobile-friendly)

#### Components Library
- Data tables with sorting/filtering
- Statistics cards
- Progress bars and spinners
- Toast notifications
- Confirmation modals
- Forms with validation
- Charts (Chart.js or similar)

#### State Management
- Svelte stores for:
  - User session
  - Active crawl jobs
  - System stats
  - Notifications
  - UI state (sidebar open/closed)

---

## App 2: End User Search Interface

### Purpose
Public-facing search interface for end users to search indexed content.

### Target Users
- General public
- Website visitors
- Anyone searching for information

### Core Features

#### 1. Search Home Page
**Route**: `/`

**Components**:
- **Hero Section**
  - Large search bar (prominent)
  - Placeholder text: "Search across indexed websites..."
  - Search button / Enter to search
  - Optional: Logo and tagline

- **Search Stats**
  - "Searching X documents" (from API stats)
  - Optional: Sample search queries

- **Features Section** (Marketing)
  - Fast search (1-3ms)
  - Typo-tolerant
  - Advanced filters

**API Calls**:
- `GET /api/stats` (for document count)

---

#### 2. Search Results Page
**Route**: `/search?q={query}`

**Layout**:
```
┌─────────────────────────────────────┐
│ [Search Bar with current query]    │
├─────────────┬───────────────────────┤
│             │                       │
│  Filters    │   Results             │
│  Sidebar    │   - Result 1          │
│             │   - Result 2          │
│  - Sorting  │   - Result 3          │
│  - Dates    │   ...                 │
│  - Word cnt │   [Pagination]        │
│             │                       │
└─────────────┴───────────────────────┘
```

**Components**:

##### Search Bar (Top)
- Current query displayed
- Instant search / Search on enter
- Clear button
- Search suggestions (Future)
- Autocomplete (Future)

##### Filters Sidebar (Left)
- **Sort Options**
  - Relevance (default)
  - Date (newest/oldest)
  - Word count (longest/shortest)

- **Date Range Filter**
  - Today
  - Past week
  - Past month
  - Custom range (date picker)

- **Content Length Filter**
  - Short (< 100 words)
  - Medium (100-500 words)
  - Long (> 500 words)
  - Custom range (slider)

- **Clear All Filters** button

##### Results Area (Center/Right)
- **Results Metadata**
  - "About X results (Y ms)"
  - Query display with highlighting

- **Result Cards** (each result):
  - Title (linked to URL)
  - URL (breadcrumb style)
  - Content snippet (with query highlighting)
  - Word count badge
  - Date badge
  - Favicon (if available)

- **Empty State**
  - No results message
  - Suggestions (try different keywords)
  - Reset filters button

- **Pagination**
  - Previous/Next buttons
  - Page numbers (1, 2, 3, ..., N)
  - Results per page selector (10, 20, 50)

**Components**:
- `SearchBar.svelte` - Main search input
- `SearchFilters.svelte` - Filter sidebar
- `SearchResults.svelte` - Results container
- `ResultCard.svelte` - Individual search result
- `Pagination.svelte` - Pagination controls
- `EmptyState.svelte` - No results message
- `SearchStats.svelte` - Results count and time

**API Calls**:
- `GET /api/search?q={query}&limit={limit}&offset={offset}&...`

---

#### 3. About Page (Optional)
**Route**: `/about`

**Content**:
- What is this search engine?
- How does it work?
- Technology stack
- Privacy policy
- Contact information

---

### End User App UI/UX

#### Design System
- **Color Scheme**: Clean, Google-like aesthetic
  - Primary: Blue (#4285F4)
  - Accent: Orange (#FBBC04)
  - Background: Pure white (#FFFFFF)
  - Text: Dark gray (#202124)

- **Typography**:
  - Clean sans-serif (Inter, Roboto)
  - Large, readable font sizes
  - Proper spacing and line height

- **Layout**:
  - Minimalist, focused on search
  - No clutter
  - Mobile-first responsive design

#### Animations
- Smooth transitions
- Loading skeletons for results
- Fade-in animations for results
- Search bar focus effects

#### State Management
- URL query parameters for search state
- Svelte stores for:
  - Search query
  - Active filters
  - Search results
  - Loading state

---

## Shared Utilities

### Location: `shared/`

#### 1. API Client
**File**: `shared/api-client/index.ts`

```typescript
// Centralized API client
export class SearchEngineAPI {
  private baseUrl: string;

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl;
  }

  // Search endpoints
  async search(params: SearchParams): Promise<SearchResponse> { }

  // Crawl endpoints
  async startCrawl(urls: string[], maxDepth: number): Promise<CrawlResponse> { }

  // Stats endpoints
  async getStats(): Promise<StatsResponse> { }

  // Health check
  async healthCheck(): Promise<HealthResponse> { }

  // Index management
  async clearIndex(): Promise<void> { }
}
```

#### 2. TypeScript Types
**File**: `shared/types/index.ts`

```typescript
export interface SearchResult {
  id: string;
  url: string;
  title: string;
  content: string;
  description?: string;
  keywords?: string[];
  word_count: number;
  crawled_at: string;
}

export interface SearchResponse {
  hits: SearchResult[];
  query: string;
  processing_time_ms: number;
  total_hits: number;
}

export interface SearchParams {
  q: string;
  limit?: number;
  offset?: number;
  min_word_count?: number;
  max_word_count?: number;
  from_date?: string;
  to_date?: string;
  sort_by?: 'crawled_at' | 'word_count';
  sort_order?: 'asc' | 'desc';
}

// ... more types
```

#### 3. Utility Functions
**File**: `shared/utils/index.ts`

```typescript
// Date formatting
export function formatDate(date: string): string { }

// URL helpers
export function extractDomain(url: string): string { }

// Text helpers
export function highlightText(text: string, query: string): string { }
export function truncateText(text: string, maxLength: number): string { }

// Validation
export function isValidUrl(url: string): boolean { }
```

---

## Implementation Phases

### Phase 4.1: Project Setup (1-2 hours)
- [ ] Create `frontend-admin/` directory
- [ ] Initialize SvelteKit project for admin
- [ ] Create `frontend-search/` directory
- [ ] Initialize SvelteKit project for search
- [ ] Create `shared/` directory with utilities
- [ ] Set up TailwindCSS for both projects
- [ ] Install dependencies (axios, date-fns, etc.)
- [ ] Configure TypeScript
- [ ] Set up environment variables

### Phase 4.2: Shared Utilities (2-3 hours)
- [ ] Create API client in `shared/`
- [ ] Define TypeScript types
- [ ] Implement utility functions
- [ ] Write tests for utilities

### Phase 4.3: Admin Dashboard - Core (4-6 hours)
- [ ] Set up layout with sidebar navigation
- [ ] Create Dashboard home page with stats
- [ ] Implement Crawl Management page
  - Start crawl form
  - Crawl history table
- [ ] Implement Index Management page
  - Stats display
  - Clear index functionality
  - Document browser

### Phase 4.4: Admin Dashboard - Advanced (3-4 hours)
- [ ] Create Search Testing page
  - Advanced filters UI
  - Results display with JSON viewer
- [ ] Add charts and visualizations
- [ ] Implement toast notifications
- [ ] Add loading states and error handling
- [ ] Polish UI/UX

### Phase 4.5: End User Search App - Core (4-5 hours)
- [ ] Create search home page with hero section
- [ ] Implement search results page
  - Search bar component
  - Results display
  - Basic filtering
- [ ] Add pagination
- [ ] Implement result highlighting

### Phase 4.6: End User Search App - Advanced (3-4 hours)
- [ ] Create filters sidebar with all options
- [ ] Add sorting functionality
- [ ] Implement date range picker
- [ ] Add loading skeletons
- [ ] Create empty states
- [ ] Add animations and transitions
- [ ] Polish mobile responsiveness

### Phase 4.7: Testing & Polish (2-3 hours)
- [ ] Test all features in both apps
- [ ] Cross-browser testing
- [ ] Mobile device testing
- [ ] Performance optimization
- [ ] Accessibility improvements
- [ ] Documentation updates

---

## Technology Stack

### Both Applications
- **Framework**: SvelteKit
- **Language**: TypeScript
- **Styling**: TailwindCSS
- **HTTP Client**: axios
- **Date Handling**: date-fns
- **Icons**: lucide-svelte or heroicons

### Admin Dashboard Specific
- **Charts**: Chart.js with svelte-chartjs
- **Tables**: TanStack Table (Svelte adapter)
- **Date Picker**: svelte-flatpickr

### End User Search Specific
- **Animations**: svelte/transition
- **URL State**: SvelteKit's $page store

---

## Development Environment

### Running Both Apps Simultaneously

**Terminal 1**: Backend
```bash
cargo run --release
```

**Terminal 2**: Admin Dashboard
```bash
cd frontend-admin
npm run dev -- --port 5000
```

**Terminal 3**: Search App
```bash
cd frontend-search
npm run dev -- --port 5001
```

**Terminal 4**: Meilisearch
```bash
docker-compose up
```

### URLs
- Backend API: `http://127.0.0.1:3000`
- Admin Dashboard: `http://localhost:5000`
- Search App: `http://localhost:5001`
- Meilisearch: `http://127.0.0.1:7700`

---

## Design Mockups (Text-based)

### Admin Dashboard - Main Layout
```
┌─────────────────────────────────────────────────────┐
│ [Logo] Search Engine Admin          [@username ▼]  │
├──────────┬──────────────────────────────────────────┤
│          │                                          │
│ 🏠 Home  │  Dashboard Overview                      │
│ 🔄 Crawl │  ┌──────────┐ ┌──────────┐ ┌──────────┐│
│ 📑 Index │  │  Docs    │ │  Active  │ │  Queries ││
│ 🔍 Search│  │  1,234   │ │  Crawls  │ │  Today   ││
│ ⚙️ Config│  │          │ │    2     │ │   456    ││
│          │  └──────────┘ └──────────┘ └──────────┘│
│          │                                          │
│          │  Recent Activity:                        │
│          │  • Crawled example.com (10 docs)        │
│          │  • Index cleared by admin                │
│          │  • New search query: "rust"              │
│          │                                          │
└──────────┴──────────────────────────────────────────┘
```

### Search App - Home Page
```
┌─────────────────────────────────────────────────────┐
│                                                     │
│                                                     │
│              [Search Engine Logo]                   │
│                                                     │
│         ┌──────────────────────────────────┐       │
│         │ Search across indexed websites   │ [🔍] │
│         └──────────────────────────────────┘       │
│                                                     │
│           Searching 1,234 documents                 │
│                                                     │
└─────────────────────────────────────────────────────┘
```

### Search App - Results Page
```
┌─────────────────────────────────────────────────────┐
│ [Logo] ┌────────────────────────────┐ [🔍]         │
│        │ rust programming           │              │
│        └────────────────────────────┘              │
├──────────┬──────────────────────────────────────────┤
│ Filters  │ About 42 results (2 ms)                 │
│          │                                          │
│ Sort by  │ ┌────────────────────────────────────┐  │
│ • Rel.   │ │ Introduction to Rust Programming   │  │
│ ○ Date   │ │ example.com/rust                   │  │
│ ○ Length │ │ Learn Rust programming language... │  │
│          │ │ 📄 523 words  📅 2025-12-09         │  │
│ Date     │ └────────────────────────────────────┘  │
│ ○ Any    │                                          │
│ ○ Today  │ ┌────────────────────────────────────┐  │
│ ○ Week   │ │ Rust Tutorial for Beginners        │  │
│          │ │ tutorial.org/rust                  │  │
│ Length   │ │ Complete guide to learning Rust... │  │
│ [====]   │ │ 📄 1,024 words  📅  2025-12-08      │  │
│ 10-1000  │ └────────────────────────────────────┘  │
│          │                                          │
│          │ [1] 2 3 4 ... 10 [Next]                 │
└──────────┴──────────────────────────────────────────┘
```

---

## API Integration Points

### Admin Dashboard → Backend API
- Dashboard: `GET /api/health`, `GET /api/stats`
- Crawl page: `POST /api/crawl`, (Future: GET crawl jobs)
- Index page: `GET /api/stats`, `DELETE /api/index`, `GET /api/search`
- Search test: `GET /api/search` with all parameters

### End User Search → Backend API
- Home page: `GET /api/stats` (for document count)
- Search page: `GET /api/search` with query parameters
- All interactions are read-only (GET requests)

---

## Security Considerations

### Admin Dashboard
- Should be protected with authentication (Phase 8)
- For now: Deploy on internal network only
- Add BASIC_AUTH as temporary measure
- CORS: Only allow admin domain

### End User Search
- Public-facing, no authentication needed
- Rate limiting on API (Phase 3.5 or later)
- CORS: Allow all origins for search endpoint
- XSS protection for displaying user queries

---

## Performance Optimization

### Admin Dashboard
- Lazy load charts and heavy components
- Debounce API calls for search testing
- Cache stats data (refresh every 30s)
- Virtual scrolling for large document lists

### End User Search
- Instant search with debounce (300ms)
- Prefetch search suggestions
- Optimize bundle size (code splitting)
- Image lazy loading (if displaying favicons)
- Cache search results in memory
- Service worker for offline support (Future)

---

## Accessibility

### Both Applications
- Semantic HTML
- ARIA labels for interactive elements
- Keyboard navigation support
- Focus management
- Color contrast compliance (WCAG AA)
- Screen reader friendly

---

## Mobile Responsiveness

### Admin Dashboard
- Collapsible sidebar on mobile
- Responsive tables (horizontal scroll or card view)
- Touch-friendly buttons and inputs
- Minimum breakpoints: 320px, 768px, 1024px

### End User Search
- Mobile-first design
- Filters in drawer/modal on mobile
- Stack layout on small screens
- Touch-optimized search interface

---

## Testing Strategy

### Unit Tests
- Test utility functions
- Test API client methods
- Test component logic

### Integration Tests
- Test API → Component data flow
- Test search workflow end-to-end
- Test crawl initiation workflow

### E2E Tests (Playwright)
- Test complete user journeys
- Test admin workflows
- Test search workflows

---

## Documentation

### For Developers
- Setup instructions for both apps
- Component documentation
- API client usage guide
- Contribution guidelines

### For Users
- Admin dashboard user guide
- Search tips and tricks
- FAQ

---

## Success Metrics

### Admin Dashboard
- All CRUD operations working
- Real-time updates for crawl jobs
- Intuitive navigation
- < 2 seconds load time

### End User Search
- Search results in < 1 second
- Mobile-friendly (Lighthouse score > 90)
- Accessibility score > 90
- User-friendly interface

---

## Deliverables

1. **Admin Dashboard App**
   - Fully functional SvelteKit application
   - All management features implemented
   - Responsive and polished UI

2. **End User Search App**
   - Clean, fast search interface
   - Advanced filtering and sorting
   - Mobile-responsive design

3. **Shared Utilities**
   - Reusable API client
   - TypeScript types
   - Helper functions

4. **Documentation**
   - Setup guides
   - User manuals
   - Developer documentation

---

## Estimated Timeline

- **Phase 4.1**: 1-2 hours
- **Phase 4.2**: 2-3 hours
- **Phase 4.3**: 4-6 hours
- **Phase 4.4**: 3-4 hours
- **Phase 4.5**: 4-5 hours
- **Phase 4.6**: 3-4 hours
- **Phase 4.7**: 2-3 hours

**Total**: 19-27 hours (2-4 days of focused work)

---

## Next Steps

1. Review and approve this plan
2. Create project structure
3. Set up both SvelteKit applications
4. Start with Phase 4.1 (Project Setup)

---

## Notes

- Both apps will initially run locally
- Production deployment planning in Phase 10
- Authentication will be added in Phase 8
- Consider using a monorepo tool (Turborepo/Nx) if projects share a lot of code
- Admin dashboard can be extended with more features over time
- End user app should prioritize speed and simplicity

