use axum::{
    extract::{Extension, Path, Query, State},
    http::{StatusCode, Method, header, HeaderMap, HeaderValue, Response},
    middleware,
    response::{IntoResponse, Redirect},
    routing::{delete, get, post},
    Json, Router,
};
use axum_login::{tower_sessions::{ExpiredDeletion, Expiry, SessionManagerLayer}, AuthManagerLayerBuilder};
use serde::{Deserialize, Serialize};
use tower_sessions_sqlx_store::PostgresStore;
use uuid::Uuid;
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::cors::{CorsLayer, AllowOrigin};
use tower_http::trace::TraceLayer;
use tracing::{error, info};
use validator::Validate;

use crate::{
    search::analytics::{AnalyticsManager, ClickEvent, SearchEvent}, // Phase 7.6-7.7
    auth::{
        self, Backend, Credentials, RegisterRequest, AuthResponse, UserRole, UserResponse,
        CreateInvitationRequest, InvitationRepository, InvitationStatus, AuthSession, // Phase 8.3
    }, // Phase 8
    search::crawler::{Crawler, ImageData}, // Phase 10.5: ImageData for hybrid image search
    ory, // Phase 8.6: Ory Kratos integration
    search::qdrant::{QdrantService, ScoredImage}, // Phase 10: Semantic search, Phase 10.5: Image search
    search::redis::{CacheManager, JobQueue},
    search::search::SearchClient,
    types::{ApiResponse, CrawlRequest, SearchQuery},
};

// Webhook handlers for Kratos events (Phase 2)
mod webhooks;

// Username availability and suggestions (Phase 8: Simplified Registration)
mod username;

#[derive(Clone)]
pub struct AppState {
    pub search_client: SearchClient,
    pub qdrant_service: Arc<QdrantService>, // Phase 10: Semantic search
    pub crawler: Crawler,
    pub db_pool: PgPool,
    pub cache: CacheManager,
    pub job_queue: JobQueue,
    pub analytics: AnalyticsManager, // Phase 7.6-7.7: Analytics tracking
    pub kratos: Arc<ory::KratosClient>, // Phase 8.6: Ory Kratos client
    pub ory_repo: ory::OryUserRepository, // Phase 8.6: Ory user features repository
}

pub async fn serve(
    addr: &str,
    search_client: SearchClient,
    qdrant_service: Arc<QdrantService>,
    db_pool: PgPool,
    mut cache: CacheManager,
    job_queue: JobQueue,
    kratos_public_url: String,
    kratos_admin_url: String,
) -> anyhow::Result<()> {
    // Initialize search index
    search_client.initialize_index().await?;

    // Note: Crawler in API state is not actively used for crawling
    // (workers handle actual crawling). This is kept for compatibility.
    let crawler = Crawler::new(3, 10);

    // Phase 7.6-7.7: Initialize analytics manager
    let analytics_redis = cache.get_connection().await?;
    let analytics = AnalyticsManager::new(analytics_redis);

    // Phase 8: Setup session store and auth backend
    let session_store = PostgresStore::new(db_pool.clone());
    session_store.migrate().await?;
    info!("Session store initialized");

    let deletion_task = tokio::task::spawn(
        session_store
            .clone()
            .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
    );

    let session_layer = SessionManagerLayer::new(session_store.clone())
        .with_expiry(Expiry::OnInactivity(time::Duration::days(7)));

    let backend = Backend::new(db_pool.clone());
    let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();
    info!("Authentication layer initialized");

    // Phase 8.6: Initialize Ory clients
    let kratos = Arc::new(ory::KratosClient::new(
        kratos_public_url,
        kratos_admin_url,
    ));
    let ory_repo = ory::OryUserRepository::new(db_pool.clone());
    info!("Ory integration initialized");

    // Phase 8: Manual CORS middleware for debugging
    info!("Using manual CORS middleware for debugging");
    let cors_middleware = middleware::from_fn(|req: axum::extract::Request, next: middleware::Next| async move {
        let origin = req.headers().get("origin").cloned();
        let method = req.method().clone();

        // Log incoming request
        if let Some(ref origin_value) = origin {
            info!("CORS: Incoming request with Origin: {:?}, Method: {}", origin_value, method);
        } else {
            info!("CORS: Incoming request without Origin header, Method: {}", method);
        }

        // Handle OPTIONS preflight requests immediately
        if method == Method::OPTIONS {
            info!("CORS: Handling OPTIONS preflight request");
            let mut response = Response::builder()
                .status(200)
                .body(axum::body::Body::empty())
                .unwrap();

            // Add CORS headers for preflight
            if let Some(origin_value) = origin {
                if let Ok(origin_str) = origin_value.to_str() {
                    let allowed_origins = vec![
                        "http://localhost:5173",
                        "http://localhost:5000",
                        "http://localhost:5001",
                        "http://localhost:5002",
                        "http://127.0.0.1:5173",
                        "http://127.0.0.1:5000",
                        "http://127.0.0.1:5001",
                        "http://127.0.0.1:5002",
                        "https://arack.io",
                        "https://www.arack.io",
                        "https://mail.arack.io",
                        "https://admin.arack.io",
                    ];

                    if allowed_origins.contains(&origin_str) {
                        response.headers_mut().insert(
                            header::ACCESS_CONTROL_ALLOW_ORIGIN,
                            origin_value
                        );
                        response.headers_mut().insert(
                            header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
                            HeaderValue::from_static("true")
                        );
                    }
                }
            }

            response.headers_mut().insert(
                header::ACCESS_CONTROL_ALLOW_METHODS,
                HeaderValue::from_static("GET,POST,PUT,DELETE,OPTIONS")
            );
            response.headers_mut().insert(
                header::ACCESS_CONTROL_ALLOW_HEADERS,
                HeaderValue::from_static("content-type,authorization,accept")
            );
            response.headers_mut().insert(
                header::ACCESS_CONTROL_MAX_AGE,
                HeaderValue::from_static("3600")
            );

            return response;
        }

        let mut response = next.run(req).await;

        // Allowed origins
        let allowed_origins = vec![
            "http://localhost:5173",
            "http://localhost:5000",
            "http://localhost:5001",
            "http://localhost:5002",
            "http://127.0.0.1:5173",
            "http://127.0.0.1:5000",
            "http://127.0.0.1:5001",
            "http://127.0.0.1:5002",
            "https://arack.io",
            "https://www.arack.io",
            "https://mail.arack.io",
            "https://admin.arack.io",
        ];

        // Check if origin is allowed
        if let Some(origin_value) = origin {
            if let Ok(origin_str) = origin_value.to_str() {
                if allowed_origins.contains(&origin_str) {
                    info!("CORS: Origin {:?} is allowed - setting header", origin_str);
                    response.headers_mut().insert(
                        header::ACCESS_CONTROL_ALLOW_ORIGIN,
                        origin_value.clone()
                    );
                    response.headers_mut().insert(
                        header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
                        HeaderValue::from_static("true")
                    );
                } else {
                    info!("CORS: Origin {:?} is NOT in allowed list", origin_str);
                }
            }
        }

        response.headers_mut().insert(
            header::ACCESS_CONTROL_ALLOW_METHODS,
            HeaderValue::from_static("GET,POST,PUT,DELETE,OPTIONS")
        );
        response.headers_mut().insert(
            header::ACCESS_CONTROL_ALLOW_HEADERS,
            HeaderValue::from_static("content-type,authorization,accept")
        );
        response.headers_mut().insert(
            header::ACCESS_CONTROL_MAX_AGE,
            HeaderValue::from_static("3600")
        );

        info!("CORS: Response headers set");
        response
    });

    // Create application state
    let state = Arc::new(AppState {
        search_client,
        qdrant_service,
        crawler,
        db_pool,
        cache,
        job_queue,
        analytics,
        kratos: kratos.clone(),
        ory_repo,
    });

    // Admin routes with require_admin middleware (Phase 8.3-8.4)
    let admin_routes = Router::new()
        .route("/api/admin/invitations", post(create_invitation))
        .route("/api/admin/invitations", get(list_invitations))
        .route("/api/admin/invitations/:id", delete(delete_invitation))
        .route("/api/admin/users", get(list_users))
        .route("/api/admin/users/:id", get(get_user))
        .route("/api/admin/users/:id", post(update_user))
        .route("/api/admin/users/:id", delete(delete_user))
        .route_layer(middleware::from_fn(auth::middleware::require_admin));

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/api/crawl", post(crawl))
        .route("/api/crawl/history", get(crawl_history))
        .route("/api/jobs/:job_id", get(get_job_status))
        .route("/api/search", get(search))
        .route("/api/search/hybrid", get(hybrid_search)) // Phase 10: Hybrid semantic search
        .route("/api/search/autocomplete", get(autocomplete)) // Phase 7.1
        .route("/api/search/images", get(search_images)) // Phase 9: Image search
        .route("/api/search/images/hybrid", get(search_images_hybrid)) // Phase 10.5: Hybrid image search
        .route("/api/stats", get(stats))
        .route("/api/stats/images", get(image_stats)) // Phase 9: Image index stats
        .route("/api/index", delete(clear_index))
        // Crawler metrics endpoints (Phase 6.10)
        .route("/api/crawler/metrics", get(crawler_metrics))
        .route("/api/crawler/domains", get(crawler_domains))
        .route("/api/crawler/scheduler", get(crawler_scheduler))
        // Analytics endpoints (Phase 7.6-7.8)
        .route("/api/analytics/summary", get(analytics_summary))
        .route("/api/analytics/click", post(track_click))
        // Username availability and suggestions (Phase 8: Simplified Registration)
        .route("/api/auth/check-username", get(username::check_username_availability))
        .route("/api/auth/suggest-usernames", post(username::suggest_usernames))
        // Simple authentication endpoints (Phase 8 - for admin dashboard)
        .route("/api/auth/login", post(login))
        .route("/api/auth/logout", post(logout))
        .route("/api/auth/me", get(current_user))
        // Kratos authentication flow endpoints (Phase 8 - Kratos Migration)
        .route("/api/auth/flows/registration", get(init_registration_flow).post(submit_registration_flow))
        .route("/api/auth/flows/login", get(init_login_flow).post(submit_login_flow))
        .route("/api/auth/flows/logout", get(init_logout_flow))
        .route("/api/auth/whoami", get(kratos_whoami))
        // Hydra OAuth provider endpoints (Phase 6 - SSO)
        .route("/api/hydra/login", get(handle_hydra_login))
        .route("/api/hydra/consent", get(handle_hydra_consent))
        .route("/api/hydra/consent/accept", post(accept_consent))
        .route("/api/hydra/consent/reject", post(reject_consent))
        // Invitation endpoints (Phase 8.3)
        .route("/api/auth/invitations/:token", get(verify_invitation))
        .route("/api/auth/invitations/:token/accept", post(accept_invitation))
        // Internal webhook endpoints (Phase 2: Automatic Provisioning)
        .route("/internal/auth/user-created", post(webhooks::handle_user_created))
        // Merge Ory routes (protected - Phase 8.6 - DEPRECATED)
        .merge(ory_routes(state.clone()))
        // Merge user routes (protected - Custom OAuth)
        .merge(user_routes(state.clone()))
        // Merge admin routes (protected)
        .merge(admin_routes)
        // Apply auth layer to ALL routes (provides AuthSession extractor)
        .layer(auth_layer);

    let app = app
        .layer(cors_middleware)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("API server listening on {}", addr);

    let server = axum::serve(listener, app);

    // Gracefully shutdown cleanup task on server shutdown
    tokio::select! {
        result = server => {
            deletion_task.abort();
            result?
        }
    }

    Ok(())
}

async fn root() -> &'static str {
    "Search Engine API - Use /api/search?q=query to search"
}

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

async fn crawl(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CrawlRequest>,
) -> impl IntoResponse {
    info!("Received crawl request for {} URLs", payload.urls.len());

    // Create a new crawl job
    let job = crate::search::redis::CrawlJob::new(
        payload.urls.clone(),
        payload.max_depth,
        None, // collection_id can be added later
    );

    // Enqueue the job for background processing
    let mut queue = state.job_queue.clone();
    match queue.enqueue(&job).await {
        Ok(_) => {
            info!("Crawl job {} enqueued successfully", job.id);
            let response = ApiResponse::success(serde_json::json!({
                "message": "Crawl job enqueued successfully",
                "job_id": job.id,
                "urls": payload.urls,
                "status": "pending"
            }));
            (StatusCode::ACCEPTED, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to enqueue crawl job: {}", e);
            let response = ApiResponse::error(format!("Failed to enqueue job: {}", e));
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

#[derive(Deserialize)]
struct CrawlHistoryQuery {
    #[serde(default = "default_crawl_history_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}

fn default_crawl_history_limit() -> i64 {
    20
}

async fn crawl_history(
    State(state): State<Arc<AppState>>,
    Query(params): Query<CrawlHistoryQuery>,
) -> impl IntoResponse {
    info!("Fetching crawl history: limit={}, offset={}", params.limit, params.offset);

    // Query crawl history from database
    let query = sqlx::query_as::<_, crate::db::models::CrawlHistory>(
        "SELECT * FROM crawl_history ORDER BY started_at DESC LIMIT $1 OFFSET $2"
    )
    .bind(params.limit)
    .bind(params.offset);

    match query.fetch_all(&state.db_pool).await {
        Ok(history) => {
            // Also get total count
            let count_result = sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM crawl_history"
            )
            .fetch_one(&state.db_pool)
            .await;

            let total = count_result.unwrap_or(0);

            let response = ApiResponse::success(serde_json::json!({
                "history": history,
                "total": total,
                "limit": params.limit,
                "offset": params.offset
            }));
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to fetch crawl history: {}", e);
            let response = ApiResponse::error(format!("Failed to fetch crawl history: {}", e));
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

async fn search(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchQuery>,
) -> impl IntoResponse {
    info!(
        "Search query: '{}', limit: {}, offset: {}",
        params.q, params.limit, params.offset
    );

    // Generate cache key
    let cache_key = crate::search::redis::CacheManager::search_cache_key(&params.q, params.limit, params.offset);

    // Try to get from cache first
    let mut cache = state.cache.clone();
    if let Ok(Some(cached_results)) = cache.get::<serde_json::Value>(&cache_key).await {
        info!("Returning cached search results for query: {}", params.q);
        let response = ApiResponse::success(cached_results);
        return (StatusCode::OK, Json(response)).into_response();
    }

    // Cache miss - fetch from Meilisearch
    let start_time = std::time::Instant::now();
    match state.search_client.search_with_params(params.clone()).await {
        Ok(mut results) => {
            // Phase 9: Fetch image counts for each result
            let urls: Vec<String> = results.hits.iter().map(|h| h.url.clone()).collect();
            if let Ok(image_counts) = state.search_client.get_image_counts_by_url(urls).await {
                for hit in &mut results.hits {
                    hit.image_count = image_counts.get(&hit.url).copied();
                }
            }

            let processing_time_ms = start_time.elapsed().as_millis() as u64;
            let result_count = results.hits.len();
            let results_json = serde_json::json!(results);

            // Store in cache (fire and forget)
            let mut cache_clone = state.cache.clone();
            let cache_key_clone = cache_key.clone();
            let results_clone = results_json.clone();
            tokio::spawn(async move {
                if let Err(e) = cache_clone.set(&cache_key_clone, &results_clone).await {
                    tracing::warn!("Failed to cache search results: {}", e);
                }
            });

            // Phase 7.6: Track search analytics (fire and forget)
            let mut analytics_clone = state.analytics.clone();
            let query_clone = params.q.clone();
            tokio::spawn(async move {
                let event = SearchEvent {
                    query: query_clone,
                    result_count,
                    processing_time_ms,
                    timestamp: chrono::Utc::now(),
                };
                if let Err(e) = analytics_clone.track_search(event).await {
                    tracing::warn!("Failed to track search analytics: {}", e);
                }
            });

            let response = ApiResponse::success(results_json);
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            let response = ApiResponse::error(format!("Search failed: {}", e));
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

/// Phase 10: Hybrid search endpoint (keyword + semantic)
#[derive(serde::Serialize)]
struct HybridSearchResponse {
    hits: Vec<HybridResult>,
    query: String,
    processing_time_ms: u64,
    keyword_count: usize,
    semantic_count: usize,
}

#[derive(serde::Serialize, Clone)]
struct HybridResult {
    id: String,
    url: String,
    title: String,
    description: Option<String>,
    content: Option<String>,
    keyword_score: Option<f32>,
    semantic_score: Option<f32>,
    combined_score: f32,
}

async fn hybrid_search(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchQuery>,
) -> impl IntoResponse {
    info!("Hybrid search query: '{}'", params.q);

    let start_time = std::time::Instant::now();

    // 1. Perform keyword search (Meilisearch)
    let keyword_results = match state.search_client.search_with_params(params.clone()).await {
        Ok(results) => results,
        Err(e) => {
            let response = ApiResponse::error(format!("Keyword search failed: {}", e));
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response();
        }
    };

    // 2. Perform semantic search (Qdrant)
    let semantic_results = match state
        .qdrant_service
        .search(&params.q, params.limit as usize)
        .await
    {
        Ok(results) => results,
        Err(e) => {
            tracing::warn!("Semantic search failed: {}", e);
            Vec::new() // Continue with keyword-only results if semantic search fails
        }
    };

    // 3. Merge results (deduplicate by URL, combine scores)
    let semantic_count = semantic_results.len();
    let merged = merge_search_results(keyword_results.hits, semantic_results);

    let processing_time_ms = start_time.elapsed().as_millis() as u64;

    let response_data = HybridSearchResponse {
        hits: merged.clone(),
        query: params.q.clone(),
        processing_time_ms,
        keyword_count: keyword_results.total_hits,
        semantic_count,
    };

    // Track analytics
    let mut analytics_clone = state.analytics.clone();
    let query_clone = params.q.clone();
    tokio::spawn(async move {
        let event = SearchEvent {
            query: query_clone,
            result_count: merged.len(),
            processing_time_ms,
            timestamp: chrono::Utc::now(),
        };
        if let Err(e) = analytics_clone.track_search(event).await {
            tracing::warn!("Failed to track search analytics: {}", e);
        }
    });

    let response = ApiResponse::success(response_data);
    (StatusCode::OK, Json(response)).into_response()
}

fn merge_search_results(
    keyword: Vec<crate::search::search::SearchResult>,
    semantic: Vec<crate::search::qdrant::ScoredPage>,
) -> Vec<HybridResult> {
    use std::collections::HashMap;

    let mut results: HashMap<String, HybridResult> = HashMap::new();

    // Add keyword results (position-based scoring)
    for (idx, result) in keyword.into_iter().enumerate() {
        let keyword_score = 1.0 - (idx as f32 / 100.0); // Decay by position
        results.insert(
            result.url.clone(),
            HybridResult {
                id: result.id,
                url: result.url,
                title: result.title,
                description: result.description,
                content: Some(result.content),
                keyword_score: Some(keyword_score),
                semantic_score: None,
                combined_score: keyword_score * 0.5, // 50% weight for keyword
            },
        );
    }

    // Add semantic results
    for result in semantic {
        results
            .entry(result.url.clone())
            .and_modify(|e| {
                e.semantic_score = Some(result.score);
                e.combined_score += result.score * 0.5; // 50% weight for semantic
            })
            .or_insert_with(|| HybridResult {
                id: result.id.clone(),
                url: result.url.clone(),
                title: result.title.clone(),
                description: None,
                content: None,
                keyword_score: None,
                semantic_score: Some(result.score),
                combined_score: result.score * 0.5,
            });
    }

    // Sort by combined score
    let mut merged: Vec<_> = results.into_values().collect();
    merged.sort_by(|a, b| {
        b.combined_score
            .partial_cmp(&a.combined_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    merged
}

/// Phase 7.1: Autocomplete endpoint
#[derive(Deserialize)]
struct AutocompleteQuery {
    q: String,
    #[serde(default = "default_autocomplete_limit")]
    limit: usize,
}

fn default_autocomplete_limit() -> usize {
    5
}

async fn autocomplete(
    State(state): State<Arc<AppState>>,
    Query(params): Query<AutocompleteQuery>,
) -> impl IntoResponse {
    info!("Autocomplete query: '{}', limit: {}", params.q, params.limit);

    match state.search_client.autocomplete(&params.q, params.limit).await {
        Ok(results) => {
            let response = ApiResponse::success(results);
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Autocomplete failed: {}", e);
            let response = ApiResponse::error(format!("Autocomplete failed: {}", e));
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

/// Phase 9: Image search endpoint
#[derive(Deserialize)]
struct ImageSearchQuery {
    q: String,
    #[serde(default = "default_image_search_limit")]
    limit: usize,
    #[serde(default)]
    offset: usize,
    min_width: Option<u32>,
    min_height: Option<u32>,
    domain: Option<String>,
}

fn default_image_search_limit() -> usize {
    20
}

// Phase 10.5: Hybrid image search result
#[derive(Debug, Serialize)]
struct HybridImageResult {
    pub id: String,
    pub image_url: String,
    pub source_url: String,
    pub alt_text: Option<String>,
    pub title: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub page_title: String,
    pub domain: String,
    pub crawled_at: chrono::DateTime<chrono::Utc>,
    pub is_og_image: bool,
    pub figcaption: Option<String>,
    pub srcset_url: Option<String>,
    pub keyword_score: Option<f32>,
    pub semantic_score: Option<f32>,
    pub combined_score: f32,
}

async fn search_images(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ImageSearchQuery>,
) -> impl IntoResponse {
    info!(
        "Image search query: '{}', limit: {}, offset: {}",
        params.q, params.limit, params.offset
    );

    match state.search_client.search_images(
        &params.q,
        params.limit,
        params.offset,
        params.min_width,
        params.min_height,
        params.domain,
    ).await {
        Ok(results) => {
            let response = ApiResponse::success(results);
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Image search failed: {}", e);
            let response = ApiResponse::error(format!("Image search failed: {}", e));
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

// Phase 10.5: Hybrid image search (keyword + semantic)
async fn search_images_hybrid(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ImageSearchQuery>,
) -> impl IntoResponse {
    info!(
        "Hybrid image search query: '{}', limit: {}, offset: {}",
        params.q, params.limit, params.offset
    );

    let start = std::time::Instant::now();

    // 1. Keyword search (Meilisearch)
    let keyword_results = match state.search_client.search_images(
        &params.q,
        params.limit * 2,  // Get more for better merging
        0,  // Start from 0 for merging
        params.min_width,
        params.min_height,
        params.domain.clone(),
    ).await {
        Ok(results) => results,
        Err(e) => {
            error!("Keyword image search failed: {}", e);
            let response = ApiResponse::error(format!("Keyword search failed: {}", e));
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response();
        }
    };

    // Extract hits from Meilisearch response
    let keyword_hits: Vec<ImageData> = keyword_results
        .get("hits")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    // 2. Semantic search (Qdrant)
    let semantic_results = match state.qdrant_service.search_images(
        &params.q,
        params.limit * 2,
    ).await {
        Ok(results) => results,
        Err(e) => {
            tracing::warn!("Semantic image search failed, using keyword-only: {}", e);
            Vec::new()  // Fallback to keyword-only if Qdrant fails
        }
    };

    // Capture semantic count before moving
    let semantic_count = semantic_results.len();

    // 3. Merge results (deduplicate by image_url, combine scores)
    let merged = merge_image_results(keyword_hits, semantic_results);

    // Apply pagination to merged results
    let total_merged = merged.len();
    let paginated: Vec<_> = merged
        .into_iter()
        .skip(params.offset)
        .take(params.limit)
        .collect();

    let processing_time_ms = start.elapsed().as_millis() as u64;

    let response_data = serde_json::json!({
        "hits": paginated,
        "query": params.q,
        "processing_time_ms": processing_time_ms,
        "total_hits": total_merged,
        "keyword_count": keyword_results.get("total_hits").and_then(|v| v.as_u64()).unwrap_or(0),
        "semantic_count": semantic_count,
    });

    let response = ApiResponse::success(response_data);
    (StatusCode::OK, Json(response)).into_response()
}

// Phase 10.5: Merge keyword and semantic image search results
fn merge_image_results(
    keyword: Vec<ImageData>,
    semantic: Vec<ScoredImage>,
) -> Vec<HybridImageResult> {
    use std::collections::HashMap;

    let mut results: HashMap<String, HybridImageResult> = HashMap::new();

    // Add keyword results
    for (idx, image) in keyword.into_iter().enumerate() {
        let keyword_score = 1.0 - (idx as f32 / 100.0);  // Decay by position
        results.insert(
            image.image_url.clone(),
            HybridImageResult {
                id: image.id,
                image_url: image.image_url,
                source_url: image.source_url,
                alt_text: image.alt_text,
                title: image.title,
                width: image.width,
                height: image.height,
                page_title: image.page_title,
                domain: image.domain,
                crawled_at: image.crawled_at,
                is_og_image: image.is_og_image,
                figcaption: image.figcaption,
                srcset_url: image.srcset_url,
                keyword_score: Some(keyword_score),
                semantic_score: None,
                combined_score: keyword_score * 0.5,  // 50% weight
            },
        );
    }

    // Add semantic results
    for result in semantic {
        results
            .entry(result.image_url.clone())
            .and_modify(|e| {
                e.semantic_score = Some(result.score);
                e.combined_score += result.score * 0.5;  // 50% weight
            })
            .or_insert_with(|| {
                // Image only found in semantic search (not in keyword)
                // Create partial result with semantic score only
                HybridImageResult {
                    id: result.id.clone(),
                    image_url: result.image_url.clone(),
                    source_url: result.source_url.clone(),
                    alt_text: None,
                    title: None,
                    width: None,
                    height: None,
                    page_title: String::new(),
                    domain: result.domain.clone(),
                    crawled_at: chrono::Utc::now(),
                    is_og_image: false,
                    figcaption: None,
                    srcset_url: None,
                    keyword_score: None,
                    semantic_score: Some(result.score),
                    combined_score: result.score * 0.5,
                }
            });
    }

    // Sort by combined score
    let mut merged: Vec<_> = results.into_values().collect();
    merged.sort_by(|a, b| {
        b.combined_score
            .partial_cmp(&a.combined_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    merged
}

async fn stats(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.search_client.get_stats().await {
        Ok(stats) => {
            let response = ApiResponse::success(stats);
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            let response = ApiResponse::error(format!("Failed to get stats: {}", e));
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

async fn image_stats(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.search_client.get_image_stats().await {
        Ok(stats) => {
            let response = ApiResponse::success(stats);
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            let response = ApiResponse::error(format!("Failed to get image stats: {}", e));
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

async fn get_job_status(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> impl IntoResponse {
    let job_uuid = match Uuid::parse_str(&job_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            let response = ApiResponse::error("Invalid job ID format".to_string());
            return (StatusCode::BAD_REQUEST, Json(response)).into_response();
        }
    };

    let mut queue = state.job_queue.clone();
    match queue.get_job(job_uuid).await {
        Ok(Some(job)) => {
            let response = ApiResponse::success(serde_json::json!({
                "job_id": job.id,
                "status": format!("{:?}", job.status),
                "urls": job.urls,
                "pages_crawled": job.pages_crawled,
                "pages_indexed": job.pages_indexed,
                "created_at": job.created_at,
                "started_at": job.started_at,
                "completed_at": job.completed_at,
                "error": job.error
            }));
            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(None) => {
            let response = ApiResponse::error("Job not found".to_string());
            (StatusCode::NOT_FOUND, Json(response)).into_response()
        }
        Err(e) => {
            let response = ApiResponse::error(format!("Failed to get job status: {}", e));
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

async fn clear_index(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    info!("Clearing search index");

    match state.search_client.clear_index().await {
        Ok(_) => {
            let response = ApiResponse::success(serde_json::json!({
                "message": "Index cleared successfully"
            }));
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            let response = ApiResponse::error(format!("Failed to clear index: {}", e));
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

// Crawler metrics endpoints (Phase 6.10)

async fn crawler_metrics(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    info!("Getting crawler metrics");

    let metrics = serde_json::json!({
        "rate_limiter": state.crawler.rate_limiter_stats(),
        "politeness": state.crawler.politeness_stats(),
        "circuit_breaker": state.crawler.circuit_breaker_stats(),
        "scheduler": state.crawler.scheduler_stats(),
        "filters": state.crawler.filter_stats(),
        "robots": state.crawler.robots_stats(),
    });

    let response = ApiResponse::success(metrics);
    (StatusCode::OK, Json(response)).into_response()
}

async fn crawler_domains(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    info!("Getting per-domain crawler stats");

    let domains = state.crawler.circuit_breaker().get_all_domains();

    let response = ApiResponse::success(serde_json::json!({
        "domains": domains
    }));
    (StatusCode::OK, Json(response)).into_response()
}

async fn crawler_scheduler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    info!("Getting crawler scheduler info");

    let stats = state.crawler.scheduler_stats();
    let tasks = state.crawler.scheduler().get_all();

    let response = ApiResponse::success(serde_json::json!({
        "stats": stats,
        "tasks": tasks.into_iter().take(100).collect::<Vec<_>>(), // Limit to 100 tasks
    }));
    (StatusCode::OK, Json(response)).into_response()
}

// Phase 7.6-7.8: Analytics endpoint handlers

#[derive(Debug, Deserialize)]
struct DaysQuery {
    #[serde(default = "default_days")]
    days: usize,
}

fn default_days() -> usize {
    7
}

async fn analytics_summary(
    State(state): State<Arc<AppState>>,
    Query(params): Query<DaysQuery>,
) -> impl IntoResponse {
    info!("Getting analytics summary for {} days", params.days);

    let mut analytics = state.analytics.clone();
    match analytics.get_summary(params.days).await {
        Ok(summary) => {
            let response = ApiResponse::success(summary);
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Analytics summary failed: {}", e);
            let response = ApiResponse::error(format!("Analytics summary failed: {}", e));
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

async fn track_click(
    State(state): State<Arc<AppState>>,
    Json(event): Json<ClickEvent>,
) -> impl IntoResponse {
    info!("Tracking click: {} at position {}", event.clicked_url, event.position);

    let mut analytics = state.analytics.clone();
    match analytics.track_click(event).await {
        Ok(_) => {
            let response = ApiResponse::success(serde_json::json!({
                "message": "Click tracked successfully"
            }));
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Click tracking failed: {}", e);
            let response = ApiResponse::error(format!("Click tracking failed: {}", e));
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

// Phase 8: Authentication endpoint handlers

async fn register(
    State(state): State<Arc<AppState>>,
    mut auth_session: AuthSession,
    Json(payload): Json<RegisterRequest>,
) -> impl IntoResponse {
    // Validate input
    if let Err(errors) = payload.validate() {
        let response = ApiResponse::error(format!("Validation failed: {}", errors));
        return (StatusCode::BAD_REQUEST, Json(response)).into_response();
    }

    info!("Registration attempt for email: {}", payload.email);

    let repo = auth::UserRepository::new(state.db_pool.clone());

    // Check if email already exists
    match repo.email_exists(&payload.email).await {
        Ok(true) => {
            let response = ApiResponse::error("Email already exists".to_string());
            return (StatusCode::CONFLICT, Json(response)).into_response();
        }
        Ok(false) => {}
        Err(e) => {
            error!("Database error checking email: {}", e);
            let response = ApiResponse::error("Internal server error".to_string());
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response();
        }
    }

    // Hash password
    let password_hash = match auth::hash_password(&payload.password) {
        Ok(hash) => hash,
        Err(e) => {
            error!("Password hashing failed: {}", e);
            let response = ApiResponse::error("Internal server error".to_string());
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response();
        }
    };

    // Create user
    let user = match repo
        .create_user(
            &payload.email,
            &password_hash,
            &payload.first_name,
            &payload.last_name,
            UserRole::User,
            None,
        )
        .await
    {
        Ok(user) => user,
        Err(e) => {
            error!("Failed to create user: {}", e);
            let response = ApiResponse::error("Failed to create user".to_string());
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response();
        }
    };

    // Login the user automatically
    if let Err(e) = auth_session.login(&user).await {
        error!("Failed to login after registration: {}", e);
    }

    info!("User registered successfully: {}", user.email);

    let response = ApiResponse::success(AuthResponse {
        user: user.into(),
    });
    (StatusCode::CREATED, Json(response)).into_response()
}

async fn login(
    mut auth_session: AuthSession,
    Json(creds): Json<Credentials>,
) -> impl IntoResponse {
    // Validate input
    if let Err(errors) = creds.validate() {
        let response = ApiResponse::error(format!("Validation failed: {}", errors));
        return (StatusCode::BAD_REQUEST, Json(response)).into_response();
    }

    info!("Login attempt for email: {}", creds.email);

    let user = match auth_session.authenticate(creds.clone()).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            info!("Invalid credentials for email: {}", creds.email);
            let response = ApiResponse::error("Invalid credentials".to_string());
            return (StatusCode::UNAUTHORIZED, Json(response)).into_response();
        }
        Err(e) => {
            error!("Authentication error: {}", e);
            let response = ApiResponse::error("Internal server error".to_string());
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response();
        }
    };

    if let Err(e) = auth_session.login(&user).await {
        error!("Failed to create session: {}", e);
        let response = ApiResponse::error("Failed to create session".to_string());
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response();
    }

    info!("User logged in successfully: {}", user.email);

    let response = ApiResponse::success(AuthResponse {
        user: user.into(),
    });
    (StatusCode::OK, Json(response)).into_response()
}

async fn logout(mut auth_session: AuthSession) -> impl IntoResponse {
    match auth_session.logout().await {
        Ok(_) => {
            info!("User logged out successfully");
            let response = ApiResponse::success(serde_json::json!({
                "message": "Logged out successfully"
            }));
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Logout failed: {}", e);
            let response = ApiResponse::error("Logout failed".to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

async fn current_user(auth_session: AuthSession) -> impl IntoResponse {
    match auth_session.user {
        Some(user) => {
            let response = ApiResponse::success(auth::UserResponse::from(user));
            (StatusCode::OK, Json(response)).into_response()
        }
        None => {
            let response = ApiResponse::error("Not authenticated".to_string());
            (StatusCode::UNAUTHORIZED, Json(response)).into_response()
        }
    }
}

// Phase 8 - Kratos Migration: Flow handler endpoints

/// Initialize registration flow
async fn init_registration_flow(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match state.kratos.init_registration_flow().await {
        Ok(flow) => {
            let response = ApiResponse::success(flow);
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to init registration flow: {}", e);
            let response = ApiResponse::error("Failed to initialize registration".to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

/// Initialize login flow
async fn init_login_flow(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    match state.kratos.init_login_flow().await {
        Ok(flow) => {
            let response = ApiResponse::success(flow);
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to init login flow: {}", e);
            let response = ApiResponse::error("Failed to initialize login".to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

/// Initialize logout flow
async fn init_logout_flow(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let cookie_header = headers.get("cookie").and_then(|h| h.to_str().ok());

    match state.kratos.init_logout_flow(cookie_header).await {
        Ok(logout_url) => {
            let response = ApiResponse::success(serde_json::json!({
                "logout_url": logout_url
            }));
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to init logout flow: {}", e);
            let response = ApiResponse::error("Failed to initialize logout".to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

/// Get current user from Kratos session
async fn kratos_whoami(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let cookie_header = match headers.get("cookie").and_then(|h| h.to_str().ok()) {
        Some(c) => c,
        None => {
            let response = ApiResponse::error("Not authenticated".to_string());
            return (StatusCode::UNAUTHORIZED, Json(response)).into_response();
        }
    };

    match state.kratos.whoami(cookie_header).await {
        Ok(session) => {
            if session.active {
                let response = ApiResponse::success(serde_json::json!({
                    "id": session.identity.id,
                    "email": session.identity.traits.email,
                    "first_name": session.identity.traits.first_name,
                    "last_name": session.identity.traits.last_name,
                    "authenticated": true
                }));
                (StatusCode::OK, Json(response)).into_response()
            } else {
                let response = ApiResponse::error("Session expired".to_string());
                (StatusCode::UNAUTHORIZED, Json(response)).into_response()
            }
        }
        Err(e) => {
            error!("Whoami failed: {}", e);
            let response = ApiResponse::error("Not authenticated".to_string());
            (StatusCode::UNAUTHORIZED, Json(response)).into_response()
        }
    }
}

/// Submit registration flow (proxy to Kratos)
async fn submit_registration_flow(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let flow_id = match payload["flow_id"].as_str() {
        Some(id) => id,
        None => {
            let response = ApiResponse::error("Missing flow_id".to_string());
            return (StatusCode::BAD_REQUEST, Json(response)).into_response();
        }
    };

    let email = match payload["email"].as_str() {
        Some(e) => e,
        None => {
            let response = ApiResponse::error("Missing email".to_string());
            return (StatusCode::BAD_REQUEST, Json(response)).into_response();
        }
    };

    let password = match payload["password"].as_str() {
        Some(p) => p,
        None => {
            let response = ApiResponse::error("Missing password".to_string());
            return (StatusCode::BAD_REQUEST, Json(response)).into_response();
        }
    };

    let first_name = match payload["first_name"].as_str() {
        Some(f) => f,
        None => {
            let response = ApiResponse::error("Missing first_name".to_string());
            return (StatusCode::BAD_REQUEST, Json(response)).into_response();
        }
    };

    let last_name = match payload["last_name"].as_str() {
        Some(l) => l,
        None => {
            let response = ApiResponse::error("Missing last_name".to_string());
            return (StatusCode::BAD_REQUEST, Json(response)).into_response();
        }
    };

    match state.kratos.submit_registration(flow_id, email, password, first_name, last_name).await {
        Ok((body, _cookies)) => {
            // For API flows, extract session_token and set it as a cookie
            let mut response = Json(ApiResponse::success(body.clone())).into_response();

            // Extract session_token from the response body and set it as ory_kratos_session cookie
            if let Some(session_token) = body.get("session_token").and_then(|v| v.as_str()) {
                let cookie_value = format!(
                    "ory_kratos_session={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=604800",
                    session_token
                );

                if let Ok(cookie_header) = header::HeaderValue::from_str(&cookie_value) {
                    response.headers_mut().append(header::SET_COOKIE, cookie_header);
                }
            }

            (StatusCode::OK, response).into_response()
        }
        Err(e) => {
            error!("Registration submission failed: {}", e);
            let response = ApiResponse::error(format!("Registration failed: {}", e));
            (StatusCode::BAD_REQUEST, Json(response)).into_response()
        }
    }
}

/// Submit login flow (proxy to Kratos)
async fn submit_login_flow(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let flow_id = match payload["flow_id"].as_str() {
        Some(id) => id,
        None => {
            let response = ApiResponse::error("Missing flow_id".to_string());
            return (StatusCode::BAD_REQUEST, Json(response)).into_response();
        }
    };

    let identifier = match payload["identifier"].as_str() {
        Some(i) => i,
        None => {
            let response = ApiResponse::error("Missing identifier".to_string());
            return (StatusCode::BAD_REQUEST, Json(response)).into_response();
        }
    };

    let password = match payload["password"].as_str() {
        Some(p) => p,
        None => {
            let response = ApiResponse::error("Missing password".to_string());
            return (StatusCode::BAD_REQUEST, Json(response)).into_response();
        }
    };

    match state.kratos.submit_login(flow_id, identifier, password).await {
        Ok((body, _cookies)) => {
            // For API flows, extract session_token and set it as a cookie
            let mut response = Json(ApiResponse::success(body.clone())).into_response();

            // Extract session_token and set it as ory_kratos_session cookie
            if let Some(session_token) = body.get("session_token").and_then(|v| v.as_str()) {
                let cookie_value = format!(
                    "ory_kratos_session={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=604800",
                    session_token
                );

                if let Ok(cookie_header) = header::HeaderValue::from_str(&cookie_value) {
                    response.headers_mut().append(header::SET_COOKIE, cookie_header);
                }
            }

            (StatusCode::OK, response).into_response()
        }
        Err(e) => {
            error!("Login submission failed: {}", e);
            let response = ApiResponse::error(format!("Login failed: {}", e));
            (StatusCode::UNAUTHORIZED, Json(response)).into_response()
        }
    }
}

// Phase 6: Hydra OAuth provider handlers (SSO for Stalwart)

#[derive(Deserialize)]
struct HydraChallenge {
    login_challenge: Option<String>,
    consent_challenge: Option<String>,
}

/// Handle Hydra login flow - bridge to Kratos
async fn handle_hydra_login(
    State(state): State<Arc<AppState>>,
    Query(challenge): Query<HydraChallenge>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let login_challenge = match challenge.login_challenge {
        Some(c) => c,
        None => {
            let response = ApiResponse::error("Missing login_challenge".to_string());
            return (StatusCode::BAD_REQUEST, Json(response)).into_response();
        }
    };

    // Check if user is already authenticated with Kratos
    let cookie_header = headers.get("cookie").and_then(|h| h.to_str().ok());

    if let Some(cookie) = cookie_header {
        if let Ok(session) = state.kratos.whoami(cookie).await {
            if session.active {
                // User is authenticated - accept login challenge
                return accept_hydra_login(state, login_challenge, session).await.into_response();
            }
        }
    }

    // User not authenticated - redirect to Kratos login
    let redirect_url = format!(
        "http://127.0.0.1:5001/auth/login?login_challenge={}",
        login_challenge
    );

    info!("User not authenticated, redirecting to login: {}", redirect_url);
    Redirect::to(&redirect_url).into_response()
}

/// Accept Hydra login challenge with Kratos session
async fn accept_hydra_login(
    state: Arc<AppState>,
    login_challenge: String,
    session: crate::ory::KratosSession,
) -> impl IntoResponse {
    let hydra_admin_url = "http://127.0.0.1:4445";
    let url = format!("{}/admin/oauth2/auth/requests/login/accept?login_challenge={}",
        hydra_admin_url, login_challenge);

    let client = reqwest::Client::new();
    let response = client
        .put(&url)
        .json(&serde_json::json!({
            "subject": session.identity.id,
            "remember": true,
            "remember_for": 604800,
            "context": {
                "email": session.identity.traits.email,
                "first_name": session.identity.traits.first_name,
                "last_name": session.identity.traits.last_name
            }
        }))
        .send()
        .await;

    match response {
        Ok(resp) => {
            let status = resp.status();
            if status.is_success() {
                if let Ok(body) = resp.json::<serde_json::Value>().await {
                    let redirect_to = body["redirect_to"].as_str().unwrap_or("");
                    info!("Login accepted, redirecting to: {}", redirect_to);
                    // Return HTTP redirect instead of JSON
                    return Redirect::to(redirect_to).into_response();
                }
            }

            error!("Hydra login accept failed with status: {:?}", status);
            let response = ApiResponse::error("Failed to accept login".to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
        Err(e) => {
            error!("Hydra login accept failed: {}", e);
            let response = ApiResponse::error("Internal error".to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

/// Handle Hydra consent flow
async fn handle_hydra_consent(
    State(state): State<Arc<AppState>>,
    Query(challenge): Query<HydraChallenge>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let consent_challenge = match challenge.consent_challenge {
        Some(c) => c,
        None => {
            let response = ApiResponse::error("Missing consent_challenge".to_string());
            return (StatusCode::BAD_REQUEST, Json(response)).into_response();
        }
    };

    // Get consent request details from Hydra
    let hydra_admin_url = "http://127.0.0.1:4445";
    let url = format!("{}/admin/oauth2/auth/requests/consent?consent_challenge={}",
        hydra_admin_url, consent_challenge);

    let client = reqwest::Client::new();
    let consent_request = match client.get(&url).send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                resp.json::<serde_json::Value>().await.ok()
            } else {
                error!("Failed to get consent request: status {:?}", resp.status());
                None
            }
        }
        Err(e) => {
            error!("Failed to get consent request: {}", e);
            None
        }
    };

    let consent_req = match consent_request {
        Some(req) => req,
        None => {
            let response = ApiResponse::error("Invalid consent request".to_string());
            return (StatusCode::BAD_REQUEST, Json(response)).into_response();
        }
    };

    // Check if user already granted consent
    let skip = consent_req["skip"].as_bool().unwrap_or(false);

    if skip {
        // User already consented - auto-accept (browser redirect from Hydra)
        return accept_hydra_consent_internal(consent_challenge.to_string(), consent_req, false).await.into_response();
    }

    // Check if this is an AJAX request (from consent UI) or browser redirect (from Hydra)
    let is_ajax = headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .map(|accept| accept.contains("application/json"))
        .unwrap_or(false);

    if is_ajax {
        // AJAX request from consent UI - return JSON with consent details
        info!("Consent UI requesting details for challenge: {}", consent_challenge);
        let response = ApiResponse::success(serde_json::json!({
            "client_name": consent_req["client"]["client_name"],
            "requested_scope": consent_req["requested_scope"],
            "consent_challenge": consent_challenge
        }));
        return (StatusCode::OK, Json(response)).into_response();
    }

    // Browser redirect from Hydra - redirect to consent UI
    let redirect_url = format!(
        "http://127.0.0.1:5001/auth/consent?consent_challenge={}",
        consent_challenge
    );

    info!("First time consent, redirecting to UI: {}", redirect_url);
    Redirect::to(&redirect_url).into_response()
}

/// Accept consent from UI
async fn accept_consent(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let consent_challenge = match payload["consent_challenge"].as_str() {
        Some(c) => c,
        None => {
            let response = ApiResponse::error("Missing consent_challenge".to_string());
            return (StatusCode::BAD_REQUEST, Json(response)).into_response();
        }
    };

    // Get consent request details
    let hydra_admin_url = "http://127.0.0.1:4445";
    let url = format!("{}/admin/oauth2/auth/requests/consent?consent_challenge={}",
        hydra_admin_url, consent_challenge);

    let client = reqwest::Client::new();
    let consent_req = match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => {
            resp.json::<serde_json::Value>().await.ok()
        }
        _ => None,
    };

    let consent_req = match consent_req {
        Some(req) => req,
        None => {
            let response = ApiResponse::error("Failed to get consent request".to_string());
            return (StatusCode::BAD_REQUEST, Json(response)).into_response();
        }
    };

    accept_hydra_consent_internal(consent_challenge.to_string(), consent_req, true).await.into_response()
}

/// Reject consent from UI
async fn reject_consent(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let consent_challenge = match payload["consent_challenge"].as_str() {
        Some(c) => c,
        None => {
            let response = ApiResponse::error("Missing consent_challenge".to_string());
            return (StatusCode::BAD_REQUEST, Json(response)).into_response();
        }
    };

    let hydra_admin_url = "http://127.0.0.1:4445";
    let url = format!("{}/admin/oauth2/auth/requests/consent/reject?consent_challenge={}",
        hydra_admin_url, consent_challenge);

    let client = reqwest::Client::new();
    let response = client
        .put(&url)
        .json(&serde_json::json!({
            "error": "access_denied",
            "error_description": "The resource owner denied the request"
        }))
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(body) = resp.json::<serde_json::Value>().await {
                let redirect_to = body["redirect_to"].as_str().unwrap_or("");
                let response = ApiResponse::success(serde_json::json!({
                    "redirect_to": redirect_to
                }));
                (StatusCode::OK, Json(response)).into_response()
            } else {
                let response = ApiResponse::error("Failed to parse response".to_string());
                (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
            }
        }
        _ => {
            let response = ApiResponse::error("Failed to reject consent".to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

/// Internal function to accept Hydra consent
/// - return_json: if true, returns JSON (for AJAX), if false, returns HTTP redirect (for browser)
async fn accept_hydra_consent_internal(
    consent_challenge: String,
    consent_req: serde_json::Value,
    return_json: bool,
) -> impl IntoResponse {
    let hydra_admin_url = "http://127.0.0.1:4445";
    let url = format!("{}/admin/oauth2/auth/requests/consent/accept?consent_challenge={}",
        hydra_admin_url, consent_challenge);

    let requested_scope = consent_req["requested_scope"]
        .as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<_>>())
        .unwrap_or_default();

    let client = reqwest::Client::new();
    let response = client
        .put(&url)
        .json(&serde_json::json!({
            "grant_scope": requested_scope,
            "grant_access_token_audience": consent_req["requested_access_token_audience"],
            "remember": true,
            "remember_for": 604800,
            "session": {
                "id_token": {
                    "email": consent_req["subject"],
                    "first_name": consent_req.get("context").and_then(|c| c.get("first_name")).and_then(|v| v.as_str()).unwrap_or(""),
                    "last_name": consent_req.get("context").and_then(|c| c.get("last_name")).and_then(|v| v.as_str()).unwrap_or("")
                }
            }
        }))
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(body) = resp.json::<serde_json::Value>().await {
                let redirect_to = body["redirect_to"].as_str().unwrap_or("");
                info!("Consent accepted, redirect URL: {}", redirect_to);

                if return_json {
                    // Return JSON (for AJAX requests from consent UI)
                    let response = ApiResponse::success(serde_json::json!({
                        "redirect_to": redirect_to
                    }));
                    (StatusCode::OK, Json(response)).into_response()
                } else {
                    // Return HTTP redirect (for browser redirects from Hydra)
                    Redirect::to(redirect_to).into_response()
                }
            } else {
                let response = ApiResponse::error("Failed to parse response".to_string());
                (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
            }
        }
        Ok(resp) => {
            error!("Hydra consent accept failed with status: {:?}", resp.status());
            let response = ApiResponse::error("Failed to accept consent".to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
        Err(e) => {
            error!("Hydra consent accept failed: {}", e);
            let response = ApiResponse::error("Internal error".to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

// Phase 8.3: Admin invitation endpoint handlers

async fn create_invitation(
    State(state): State<Arc<AppState>>,
    auth_session: AuthSession,
    Json(payload): Json<CreateInvitationRequest>,
) -> impl IntoResponse {
    let admin_user = auth_session.user.expect("Admin middleware should have verified authentication");

    info!("Admin {} creating invitation for {}", admin_user.email, payload.email);

    let repo = InvitationRepository::new(state.db_pool.clone());
    let user_repo = auth::UserRepository::new(state.db_pool.clone());

    // Check if email already exists as a user
    match user_repo.email_exists(&payload.email).await {
        Ok(true) => {
            let response = ApiResponse::error("User with this email already exists".to_string());
            return (StatusCode::CONFLICT, Json(response)).into_response();
        }
        Ok(false) => {}
        Err(e) => {
            error!("Database error checking email: {}", e);
            let response = ApiResponse::error("Internal server error".to_string());
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response();
        }
    }

    // Create invitation
    let role = payload.role.unwrap_or(UserRole::User);
    let expires_in_days = payload.expires_in_days.unwrap_or(7);

    match repo
        .create_invitation(&payload.email, admin_user.id, role, expires_in_days)
        .await
    {
        Ok(invitation) => {
            info!("Invitation created successfully for {}", payload.email);
            let response = ApiResponse::success(invitation);
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to create invitation: {}", e);
            let response = ApiResponse::error("Failed to create invitation".to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

#[derive(Deserialize)]
struct ListInvitationsQuery {
    status: Option<InvitationStatus>,
    #[serde(default = "default_page")]
    page: i64,
    #[serde(default = "default_limit")]
    limit: i64,
}

fn default_page() -> i64 {
    1
}

fn default_limit() -> i64 {
    20
}

async fn list_invitations(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListInvitationsQuery>,
) -> impl IntoResponse {
    info!("Admin listing invitations, page: {}, limit: {}", params.page, params.limit);

    let repo = InvitationRepository::new(state.db_pool.clone());

    match repo
        .list_invitations(params.status, params.page, params.limit)
        .await
    {
        Ok((invitations, total)) => {
            let response = ApiResponse::success(serde_json::json!({
                "invitations": invitations,
                "total": total,
                "page": params.page,
                "limit": params.limit,
                "total_pages": (total as f64 / params.limit as f64).ceil() as i64,
            }));
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to list invitations: {}", e);
            let response = ApiResponse::error("Failed to list invitations".to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

async fn delete_invitation(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    info!("Admin deleting invitation {}", id);

    let repo = InvitationRepository::new(state.db_pool.clone());

    match repo.delete_invitation(id).await {
        Ok(_) => {
            info!("Invitation {} deleted successfully", id);
            let response = ApiResponse::success(serde_json::json!({
                "message": "Invitation deleted successfully"
            }));
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to delete invitation: {}", e);
            let response = ApiResponse::error("Failed to delete invitation".to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

async fn verify_invitation(
    State(state): State<Arc<AppState>>,
    Path(token): Path<String>,
) -> impl IntoResponse {
    info!("Verifying invitation token");

    let repo = InvitationRepository::new(state.db_pool.clone());

    match repo.find_by_token(&token).await {
        Ok(Some(invitation)) => {
            // Check if invitation is still valid
            if invitation.status != InvitationStatus::Pending {
                let response = ApiResponse::error("Invitation is no longer valid".to_string());
                return (StatusCode::BAD_REQUEST, Json(response)).into_response();
            }

            if invitation.expires_at < chrono::Utc::now() {
                let response = ApiResponse::error("Invitation has expired".to_string());
                return (StatusCode::BAD_REQUEST, Json(response)).into_response();
            }

            let response = ApiResponse::success(serde_json::json!({
                "email": invitation.email,
                "role": invitation.role,
                "expires_at": invitation.expires_at,
            }));
            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(None) => {
            let response = ApiResponse::error("Invalid invitation token".to_string());
            (StatusCode::NOT_FOUND, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to verify invitation: {}", e);
            let response = ApiResponse::error("Internal server error".to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

#[derive(Deserialize, Validate)]
struct AcceptInvitationRequest {
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    password: String,
    #[validate(length(min = 1, message = "First name is required"))]
    first_name: String,
    #[validate(length(min = 1, message = "Last name is required"))]
    last_name: String,
}

async fn accept_invitation(
    State(state): State<Arc<AppState>>,
    mut auth_session: AuthSession,
    Path(token): Path<String>,
    Json(payload): Json<AcceptInvitationRequest>,
) -> impl IntoResponse {
    // Validate input
    if let Err(errors) = payload.validate() {
        let response = ApiResponse::error(format!("Validation failed: {}", errors));
        return (StatusCode::BAD_REQUEST, Json(response)).into_response();
    }

    info!("Accepting invitation");

    let invitation_repo = InvitationRepository::new(state.db_pool.clone());
    let user_repo = auth::UserRepository::new(state.db_pool.clone());

    // Find and validate invitation
    let invitation = match invitation_repo.find_by_token(&token).await {
        Ok(Some(inv)) => inv,
        Ok(None) => {
            let response = ApiResponse::error("Invalid invitation token".to_string());
            return (StatusCode::NOT_FOUND, Json(response)).into_response();
        }
        Err(e) => {
            error!("Failed to find invitation: {}", e);
            let response = ApiResponse::error("Internal server error".to_string());
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response();
        }
    };

    // Check invitation status
    if invitation.status != InvitationStatus::Pending {
        let response = ApiResponse::error("Invitation is no longer valid".to_string());
        return (StatusCode::BAD_REQUEST, Json(response)).into_response();
    }

    // Check expiration
    if invitation.expires_at < chrono::Utc::now() {
        let response = ApiResponse::error("Invitation has expired".to_string());
        return (StatusCode::BAD_REQUEST, Json(response)).into_response();
    }

    // Check if user already exists (shouldn't happen, but double check)
    match user_repo.email_exists(&invitation.email).await {
        Ok(true) => {
            let response = ApiResponse::error("User with this email already exists".to_string());
            return (StatusCode::CONFLICT, Json(response)).into_response();
        }
        Ok(false) => {}
        Err(e) => {
            error!("Database error checking email: {}", e);
            let response = ApiResponse::error("Internal server error".to_string());
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response();
        }
    }

    // Hash password
    let password_hash = match auth::hash_password(&payload.password) {
        Ok(hash) => hash,
        Err(e) => {
            error!("Password hashing failed: {}", e);
            let response = ApiResponse::error("Internal server error".to_string());
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response();
        }
    };

    // Create user
    let user = match user_repo
        .create_user(
            &invitation.email,
            &password_hash,
            &payload.first_name,
            &payload.last_name,
            invitation.role,
            Some(invitation.invited_by),
        )
        .await
    {
        Ok(user) => user,
        Err(e) => {
            error!("Failed to create user: {}", e);
            let response = ApiResponse::error("Failed to create user".to_string());
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response();
        }
    };

    // Mark invitation as accepted
    if let Err(e) = invitation_repo.mark_as_accepted(invitation.id).await {
        error!("Failed to mark invitation as accepted: {}", e);
        // Don't fail the request, user is created
    }

    // Login the user automatically
    if let Err(e) = auth_session.login(&user).await {
        error!("Failed to login after accepting invitation: {}", e);
    }

    info!("User created and logged in from invitation: {}", user.email);

    let response = ApiResponse::success(AuthResponse {
        user: user.into(),
    });
    (StatusCode::CREATED, Json(response)).into_response()
}

// Phase 8.4: Admin user management endpoint handlers

#[derive(Deserialize)]
struct ListUsersQuery {
    role: Option<UserRole>,
    is_active: Option<bool>,
    #[serde(default = "default_page")]
    page: i64,
    #[serde(default = "default_limit")]
    limit: i64,
}

async fn list_users(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListUsersQuery>,
) -> impl IntoResponse {
    info!("Admin listing users, page: {}, limit: {}", params.page, params.limit);

    let repo = auth::UserRepository::new(state.db_pool.clone());

    match repo
        .list_users(params.role, params.is_active, params.page, params.limit)
        .await
    {
        Ok((users, total)) => {
            let user_responses: Vec<UserResponse> = users.into_iter().map(|u| u.into()).collect();
            let response = ApiResponse::success(serde_json::json!({
                "users": user_responses,
                "total": total,
                "page": params.page,
                "limit": params.limit,
                "total_pages": (total as f64 / params.limit as f64).ceil() as i64,
            }));
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to list users: {}", e);
            let response = ApiResponse::error("Failed to list users".to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    info!("Admin getting user {}", id);

    let repo = auth::UserRepository::new(state.db_pool.clone());

    match repo.find_by_id(id).await {
        Ok(Some(user)) => {
            let response = ApiResponse::success(UserResponse::from(user));
            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(None) => {
            let response = ApiResponse::error("User not found".to_string());
            (StatusCode::NOT_FOUND, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to get user: {}", e);
            let response = ApiResponse::error("Internal server error".to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

#[derive(Deserialize)]
struct UpdateUserRequest {
    first_name: Option<String>,
    last_name: Option<String>,
    role: Option<UserRole>,
    is_active: Option<bool>,
}

async fn update_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateUserRequest>,
) -> impl IntoResponse {
    info!("Admin updating user {}", id);

    let repo = auth::UserRepository::new(state.db_pool.clone());

    // Check if user exists
    let mut user = match repo.find_by_id(id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            let response = ApiResponse::error("User not found".to_string());
            return (StatusCode::NOT_FOUND, Json(response)).into_response();
        }
        Err(e) => {
            error!("Failed to find user: {}", e);
            let response = ApiResponse::error("Internal server error".to_string());
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response();
        }
    };

    // Update fields
    if let Some(first_name) = payload.first_name {
        user.first_name = Some(first_name);
    }
    if let Some(last_name) = payload.last_name {
        user.last_name = Some(last_name);
    }
    if let Some(role) = payload.role {
        user.role = role;
    }
    if let Some(is_active) = payload.is_active {
        user.is_active = is_active;
    }

    // Update user in database
    match repo.update_user(&user).await {
        Ok(_) => {
            info!("User {} updated successfully", id);
            let response = ApiResponse::success(UserResponse::from(user));
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to update user: {}", e);
            let response = ApiResponse::error("Failed to update user".to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    info!("Admin deleting user {}", id);

    let repo = auth::UserRepository::new(state.db_pool.clone());

    match repo.delete_user(id).await {
        Ok(_) => {
            info!("User {} deleted successfully", id);
            let response = ApiResponse::success(serde_json::json!({
                "message": "User deleted successfully"
            }));
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to delete user: {}", e);
            let response = ApiResponse::error("Failed to delete user".to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

// Phase 8.6: Ory-authenticated routes and handlers

fn ory_routes(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/ory/me", get(get_ory_user))
        .route("/api/ory/preferences", get(get_preferences))
        .route("/api/ory/preferences", post(update_preferences))
        .route("/api/ory/saved-searches", get(list_saved_searches))
        .route("/api/ory/saved-searches", post(create_saved_search))
        .route("/api/ory/saved-searches/:id", delete(delete_saved_search))
        .route("/api/ory/search-history", get(get_search_history))
        .route("/api/ory/search-history", post(track_search_history))
        .route("/api/ory/search-history/click", post(track_click_history))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            ory::middleware::require_ory_auth
        ))
        .with_state(state)
}

// Custom OAuth authenticated routes (replaces Ory routes)
fn user_routes(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/user/search-history", get(get_user_search_history))
        .route("/api/user/search-history", post(track_user_search_history))
        .route("/api/user/search-history/click", post(track_user_click_history))
        .route_layer(middleware::from_fn(auth::middleware::require_auth))
        .with_state(state)
}

/// Get user search history (Custom OAuth)
async fn get_user_search_history(
    State(state): State<Arc<AppState>>,
    auth_session: AuthSession,
    Query(params): Query<HistoryQuery>,
) -> impl IntoResponse {
    let user = match auth_session.user {
        Some(user) => user,
        None => {
            let response = ApiResponse::error("Authentication required".to_string());
            return (StatusCode::UNAUTHORIZED, Json(response)).into_response();
        }
    };

    match state.ory_repo.get_search_history(user.id.to_string(), params.limit).await {
        Ok(history) => {
            let response = ApiResponse::success(serde_json::json!({
                "history": history,
                "total": history.len(),
            }));
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to get search history: {}", e);
            let response = ApiResponse::error("Failed to get search history".to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

/// Track a search query in history (Custom OAuth)
async fn track_user_search_history(
    State(state): State<Arc<AppState>>,
    auth_session: AuthSession,
    Json(payload): Json<ory::TrackSearchRequest>,
) -> impl IntoResponse {
    let user = match auth_session.user {
        Some(user) => user,
        None => {
            let response = ApiResponse::error("Authentication required".to_string());
            return (StatusCode::UNAUTHORIZED, Json(response)).into_response();
        }
    };

    // Check if user has opted out of analytics
    match state.ory_repo.get_or_create_preferences(user.id.to_string()).await {
        Ok(prefs) => {
            if prefs.analytics_opt_out {
                info!("User {} has opted out of search tracking", user.id);
                let response = ApiResponse::success(serde_json::json!({
                    "message": "Search tracking skipped (opted out)"
                }));
                return (StatusCode::OK, Json(response)).into_response();
            }
        }
        Err(e) => {
            error!("Failed to check analytics opt-out: {}", e);
            // Continue anyway
        }
    }

    match state.ory_repo.track_search(user.id.to_string(), payload).await {
        Ok(history) => {
            let response = ApiResponse::success(history);
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to track search: {}", e);
            let response = ApiResponse::error("Failed to track search".to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

/// Track a click on a search result (Custom OAuth)
async fn track_user_click_history(
    State(state): State<Arc<AppState>>,
    auth_session: AuthSession,
    Json(payload): Json<ory::TrackClickRequest>,
) -> impl IntoResponse {
    let user = match auth_session.user {
        Some(user) => user,
        None => {
            let response = ApiResponse::error("Authentication required".to_string());
            return (StatusCode::UNAUTHORIZED, Json(response)).into_response();
        }
    };

    match state.ory_repo.track_click(
        user.id.to_string(),
        payload.search_history_id,
        payload.clicked_url,
        payload.clicked_position,
    ).await {
        Ok(true) => {
            let response = ApiResponse::success(serde_json::json!({
                "message": "Click tracked successfully"
            }));
            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(false) => {
            let response = ApiResponse::error("Search history not found".to_string());
            (StatusCode::NOT_FOUND, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to track click: {}", e);
            let response = ApiResponse::error("Failed to track click".to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

/// Get current Ory user information
async fn get_ory_user(
    Extension(session): Extension<ory::OrySession>,
) -> impl IntoResponse {
    let identity = &session.0.identity;
    let traits = &identity.traits;

    let response = ApiResponse::success(serde_json::json!({
        "id": identity.id,
        "email": traits.email,
        "first_name": traits.first_name,
        "last_name": traits.last_name,
        "session_id": session.0.id,
        "authenticated_at": session.0.authenticated_at,
    }));
    (StatusCode::OK, Json(response)).into_response()
}

/// Get or create user preferences
async fn get_preferences(
    State(state): State<Arc<AppState>>,
    Extension(session): Extension<ory::OrySession>,
) -> impl IntoResponse {
    let kratos_id = session.0.identity.id;

    match state.ory_repo.get_or_create_preferences(kratos_id).await {
        Ok(prefs) => {
            let response = ApiResponse::success(prefs);
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to get preferences: {}", e);
            let response = ApiResponse::error("Failed to get preferences".to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

/// Update user preferences
async fn update_preferences(
    State(state): State<Arc<AppState>>,
    Extension(session): Extension<ory::OrySession>,
    Json(payload): Json<ory::UpdatePreferencesRequest>,
) -> impl IntoResponse {
    let kratos_id = session.0.identity.id;

    match state.ory_repo.update_preferences(kratos_id, payload).await {
        Ok(prefs) => {
            info!("User {} updated preferences", kratos_id);
            let response = ApiResponse::success(prefs);
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to update preferences: {}", e);
            let response = ApiResponse::error("Failed to update preferences".to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

/// List all saved searches for the user
async fn list_saved_searches(
    State(state): State<Arc<AppState>>,
    Extension(session): Extension<ory::OrySession>,
) -> impl IntoResponse {
    let kratos_id = session.0.identity.id;

    match state.ory_repo.list_saved_searches(kratos_id).await {
        Ok(searches) => {
            let response = ApiResponse::success(serde_json::json!({
                "searches": searches,
                "total": searches.len(),
            }));
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to list saved searches: {}", e);
            let response = ApiResponse::error("Failed to list saved searches".to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

/// Create a new saved search
async fn create_saved_search(
    State(state): State<Arc<AppState>>,
    Extension(session): Extension<ory::OrySession>,
    Json(payload): Json<ory::CreateSavedSearchRequest>,
) -> impl IntoResponse {
    // Validate input
    if let Err(errors) = payload.validate() {
        let response = ApiResponse::error(format!("Validation failed: {}", errors));
        return (StatusCode::BAD_REQUEST, Json(response)).into_response();
    }

    let kratos_id = session.0.identity.id;

    match state.ory_repo.create_saved_search(kratos_id, payload).await {
        Ok(search) => {
            info!("User {} created saved search: {}", kratos_id, search.name);
            let response = ApiResponse::success(search);
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to create saved search: {}", e);
            let response = ApiResponse::error("Failed to create saved search".to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

/// Delete a saved search
async fn delete_saved_search(
    State(state): State<Arc<AppState>>,
    Extension(session): Extension<ory::OrySession>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let kratos_id = session.0.identity.id;

    match state.ory_repo.delete_saved_search(kratos_id, id).await {
        Ok(true) => {
            info!("User {} deleted saved search {}", kratos_id, id);
            let response = ApiResponse::success(serde_json::json!({
                "message": "Saved search deleted successfully"
            }));
            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(false) => {
            let response = ApiResponse::error("Saved search not found".to_string());
            (StatusCode::NOT_FOUND, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to delete saved search: {}", e);
            let response = ApiResponse::error("Failed to delete saved search".to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

/// Get search history for the user
#[derive(Deserialize)]
struct HistoryQuery {
    #[serde(default = "default_history_limit")]
    limit: i64,
}

fn default_history_limit() -> i64 {
    50
}

async fn get_search_history(
    State(state): State<Arc<AppState>>,
    Extension(session): Extension<ory::OrySession>,
    Query(params): Query<HistoryQuery>,
) -> impl IntoResponse {
    let kratos_id = session.0.identity.id;

    match state.ory_repo.get_search_history(kratos_id, params.limit).await {
        Ok(history) => {
            let response = ApiResponse::success(serde_json::json!({
                "history": history,
                "total": history.len(),
            }));
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to get search history: {}", e);
            let response = ApiResponse::error("Failed to get search history".to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

/// Track a search query in history
async fn track_search_history(
    State(state): State<Arc<AppState>>,
    Extension(session): Extension<ory::OrySession>,
    Json(payload): Json<ory::TrackSearchRequest>,
) -> impl IntoResponse {
    let kratos_id = session.0.identity.id;

    // Check if user has opted out of analytics
    match state.ory_repo.get_or_create_preferences(kratos_id).await {
        Ok(prefs) => {
            if prefs.analytics_opt_out {
                info!("User {} has opted out of search tracking", kratos_id);
                let response = ApiResponse::success(serde_json::json!({
                    "message": "Search tracking skipped (opted out)"
                }));
                return (StatusCode::OK, Json(response)).into_response();
            }
        }
        Err(e) => {
            error!("Failed to check analytics opt-out: {}", e);
            // Continue anyway
        }
    }

    match state.ory_repo.track_search(kratos_id, payload).await {
        Ok(history) => {
            let response = ApiResponse::success(history);
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to track search: {}", e);
            let response = ApiResponse::error("Failed to track search".to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

/// Track a click on a search result
async fn track_click_history(
    State(state): State<Arc<AppState>>,
    Extension(session): Extension<ory::OrySession>,
    Json(payload): Json<ory::TrackClickRequest>,
) -> impl IntoResponse {
    let kratos_id = session.0.identity.id;

    match state.ory_repo.track_click(
        kratos_id,
        payload.search_history_id,
        payload.clicked_url,
        payload.clicked_position,
    ).await {
        Ok(true) => {
            let response = ApiResponse::success(serde_json::json!({
                "message": "Click tracked successfully"
            }));
            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(false) => {
            let response = ApiResponse::error("Search history not found".to_string());
            (StatusCode::NOT_FOUND, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to track click: {}", e);
            let response = ApiResponse::error("Failed to track click".to_string());
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}
