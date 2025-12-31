# Search Engine

A high-performance search engine built with Rust, Spider-rs web crawler, Meilisearch, and SvelteKit.

## Features

- **Fast Web Crawling**: Powered by Spider-rs for parallel, efficient crawling
- **Typo-Tolerant Search**: Meilisearch provides instant, relevant results (1-3ms)
- **Advanced Search**: Pagination, filtering, and sorting capabilities
- **Modern UI**: Built with SvelteKit and TailwindCSS (Phase 4)
- **REST API**: Production-ready API for crawling and searching
- **Real-time Indexing**: Automatic indexing of crawled content
- **Comprehensive Documentation**: Complete API reference and guides

## Tech Stack

### Backend
- **Rust**: High-performance backend
- **Spider-rs**: Web crawler
- **Meilisearch**: Search engine
- **Axum**: Web framework
- **Tokio**: Async runtime

### Frontend
- **SvelteKit**: Full-stack framework
- **TailwindCSS**: Styling
- **TypeScript**: Type safety

## Prerequisites

- Rust 1.70+ ([Install Rust](https://rustup.rs/))
- Node.js 18+ ([Install Node.js](https://nodejs.org/))
- Docker & Docker Compose ([Install Docker](https://docs.docker.com/get-docker/))

## Quick Start

### 1. Clone the Repository

```bash
git clone <repository-url>
cd Engine_search
```

### 2. Set Up Environment Variables

```bash
cp .env.example .env
```

Edit `.env` if you need to change default values:
```env
SERVER_HOST=127.0.0.1
SERVER_PORT=3000
MEILISEARCH_URL=http://127.0.0.1:7700
MEILISEARCH_KEY=masterKey
CRAWLER_MAX_DEPTH=3
CRAWLER_MAX_CONCURRENT=10
RUST_LOG=info
```

### 3. Start Meilisearch

```bash
docker-compose up -d
```

This will start Meilisearch on `http://127.0.0.1:7700`

### 4. Build and Run the Backend

```bash
cargo build --release
cargo run --release
```

The API server will start on `http://127.0.0.1:3000`

### 5. Start the Admin Dashboard

```bash
cd frontend-admin
npm install
npm run dev
```

The admin dashboard will be available at `http://localhost:5004`

### 6. Start the Search App

```bash
cd frontend-search
npm install
npm run dev
```

The search app will be available at `http://localhost:5005`

## API Endpoints

**For complete API documentation, see [API_DOCUMENTATION.md](API_DOCUMENTATION.md)**

### Health Check
```bash
GET /health
```

### Start Crawling
```bash
POST /api/crawl
Content-Type: application/json

{
  "urls": ["https://example.com"],
  "max_depth": 2
}
```

### Search with Advanced Features
```bash
# Basic search
GET /api/search?q=your+query&limit=20

# With pagination
GET /api/search?q=query&limit=10&offset=20

# With filtering
GET /api/search?q=query&min_word_count=10&max_word_count=100

# With sorting
GET /api/search?q=query&sort_by=word_count&sort_order=desc

# Combined features
GET /api/search?q=query&limit=10&min_word_count=50&sort_by=crawled_at&sort_order=desc
```

### Get Statistics
```bash
GET /api/stats
```

### Clear Index
```bash
DELETE /api/index
```

## Usage Examples

### 1. Crawl a Website

```bash
curl -X POST http://127.0.0.1:3000/api/crawl \
  -H "Content-Type: application/json" \
  -d '{
    "urls": ["https://example.com"],
    "max_depth": 2
  }'
```

### 2. Search Indexed Content

```bash
curl "http://127.0.0.1:3000/api/search?q=example&limit=10"
```

### 3. Get Index Statistics

```bash
curl http://127.0.0.1:3000/api/stats
```

## Project Structure

```
Engine_search/
├── src/                 # Rust Backend
│   ├── main.rs          # Application entry point
│   ├── api/             # REST API handlers
│   ├── config/          # Configuration management
│   ├── crawler/         # Web crawler with sitemap support
│   └── search/          # Meilisearch integration
├── frontend-admin/      # Admin Dashboard (SvelteKit)
│   ├── src/
│   │   ├── routes/      # Pages (dashboard, collections)
│   │   ├── lib/         # Shared components and utilities
│   │   └── app.css      # Global styles
│   └── package.json
├── frontend-search/     # End-User Search App (SvelteKit)
│   ├── src/
│   │   ├── routes/      # Pages (home, search results)
│   │   ├── lib/         # Components (FiltersPanel, MobileFilters)
│   │   └── app.css      # Global styles
│   └── package.json
├── shared/              # Shared utilities
├── Cargo.toml           # Rust dependencies
├── docker-compose.yml   # Meilisearch setup
├── .env.example         # Environment variables template
└── README.md
```

## Development

### Run in Development Mode

```bash
# Terminal 1: Start Meilisearch
docker-compose up

# Terminal 2: Run backend with auto-reload (install cargo-watch first)
cargo install cargo-watch
cargo watch -x run

# Terminal 3: Run frontend
cd frontend
npm run dev
```

### Run Tests

```bash
cargo test
```

### Format Code

```bash
cargo fmt
```

### Lint Code

```bash
cargo clippy
```

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `SERVER_HOST` | `127.0.0.1` | API server host |
| `SERVER_PORT` | `3000` | API server port |
| `MEILISEARCH_URL` | `http://127.0.0.1:7700` | Meilisearch URL |
| `MEILISEARCH_KEY` | `masterKey` | Meilisearch API key |
| `CRAWLER_MAX_DEPTH` | `3` | Maximum crawl depth |
| `CRAWLER_MAX_CONCURRENT` | `10` | Max concurrent requests |
| `RUST_LOG` | `info` | Log level (trace, debug, info, warn, error) |

### Crawler Settings

The crawler respects:
- `robots.txt` files
- Rate limiting per domain
- Maximum depth configuration
- Concurrent request limits

## Troubleshooting

### Meilisearch Connection Failed

Ensure Meilisearch is running:
```bash
docker-compose ps
```

If not running, start it:
```bash
docker-compose up -d
```

### Port Already in Use

Change the `SERVER_PORT` in your `.env` file or stop the conflicting process.

### Crawl Not Working

Check that:
1. The URL is accessible
2. The website allows crawling (check `robots.txt`)
3. You have internet connectivity
4. Check logs with `RUST_LOG=debug cargo run`

## Documentation

- **[API_DOCUMENTATION.md](API_DOCUMENTATION.md)** - Complete API reference with examples
- **[TEST_REPORT.md](TEST_REPORT.md)** - Comprehensive test results (20/20 tests passed)
- **[PHASE3_COMPLETION_SUMMARY.md](PHASE3_COMPLETION_SUMMARY.md)** - Phase 3 completion details
- **[plan.md](plan.md)** - Complete implementation roadmap

## Project Status

### ✅ Completed Phases
- **Phase 1: Foundation Setup** - Project infrastructure and dependencies
- **Phase 2: Core Crawler Implementation** - Web crawler with Meilisearch integration
- **Phase 3: REST API Development** - Production-ready REST API with advanced features
- **Phase 4: Frontend UI Development** - Admin Dashboard and End-User Search App ✅

### 🚀 Current Status
All MVP phases (1-4) completed! The system is fully functional with:
- Backend API running on http://127.0.0.1:3000
- Admin Dashboard on http://localhost:5004
- Search App on http://localhost:5005

### 📋 Roadmap

See [plan.md](plan.md) for the complete implementation roadmap.

#### MVP Progress (Phases 1-4)
- [x] Phase 1: Foundation Setup ✅
- [x] Phase 2: Core Crawler Implementation ✅
- [x] Phase 3: REST API Development ✅
- [x] Phase 4: Frontend UI Development ✅
  - [x] Phase 4.5: End-User Search App ✅
  - [x] Phase 4.6: Advanced Features (Filters, Sorting, Mobile) ✅

#### Future Phases
- Phase 5-10: Production-ready features
- Phase 11: Advanced features (AI/ML, semantic search)
- Phase 12: Testing & QA

## Performance Metrics

- **Search Speed**: 1-3ms average response time
- **Crawl Speed**: 2-5 seconds per site (varies by site)
- **Test Coverage**: 20/20 tests passed (100% success rate)
- **API Uptime**: 100% during testing phase

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests and linting
5. Submit a pull request

## License

[Add your license here]

## Support

For issues and questions:
- Open an issue on GitHub
- Check the [plan.md](plan.md) for implementation details

## Acknowledgments

- [Spider-rs](https://github.com/spider-rs/spider) - Fast web crawler
- [Meilisearch](https://www.meilisearch.com/) - Search engine
- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [SvelteKit](https://kit.svelte.dev/) - Frontend framework
