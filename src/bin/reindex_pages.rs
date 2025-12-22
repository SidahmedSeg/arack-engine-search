use anyhow::Result;
use arack_shared::config::Config;
use arack_shared::search::qdrant::QdrantService;
use arack_shared::search::search::SearchClient;
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!("Starting page re-indexing to Qdrant for semantic search...");

    let config = Config::load()?;

    // Initialize Qdrant service
    let qdrant_config = config.qdrant();
    let qdrant = Arc::new(
        QdrantService::new(&qdrant_config.url, qdrant_config.collection_name).await?
    );

    // Initialize Meilisearch client to fetch existing pages
    let search_client = SearchClient::new(&config.meilisearch_url, &config.meilisearch_key)?;

    // Fetch all pages from Meilisearch in batches
    let batch_size = 50; // Smaller batch for BERT processing
    let mut offset = 0;
    let mut total_indexed = 0;

    loop {
        info!("Fetching batch at offset {}", offset);

        // Use search_with_params to get all documents with pagination
        let params = arack_shared::types::SearchQuery {
            q: String::new(),  // Empty query gets all
            limit: batch_size,
            offset,
            sort_by: None,
            sort_order: None,
            min_word_count: None,
            max_word_count: None,
            from_date: None,
            to_date: None,
            domain: None,
        };

        let results = search_client
            .search_with_params(params)
            .await?;

        if results.hits.is_empty() {
            break;
        }

        info!("Processing {} pages in this batch", results.hits.len());

        // Index each page to Qdrant
        for hit in &results.hits {
            // Skip if essential fields are missing
            if hit.url.is_empty() {
                eprintln!("Skipping page with missing url");
                continue;
            }

            match qdrant
                .index_page(&hit.id, &hit.url, &hit.title, &hit.content)
                .await
            {
                Ok(_) => {
                    total_indexed += 1;
                    if total_indexed % 10 == 0 {
                        info!("Indexed {} pages so far...", total_indexed);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to index page {}: {}", hit.url, e);
                }
            }
        }

        offset += batch_size;

        // Check if we've processed all pages
        if offset >= results.total_hits {
            break;
        }

        // Small delay to avoid overwhelming the system
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    info!("Re-indexing complete! Indexed {} pages to Qdrant", total_indexed);
    info!("Semantic search is now enabled for these pages!");

    Ok(())
}
