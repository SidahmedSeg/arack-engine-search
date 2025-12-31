use anyhow::Result;
use engine_search::config::Config;
use engine_search::qdrant::QdrantService;
use engine_search::search::SearchClient;
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!("Starting image re-indexing to Qdrant...");

    let config = Config::load()?;

    // Initialize Qdrant service
    let qdrant_config = config.qdrant();
    let qdrant = Arc::new(
        QdrantService::new(&qdrant_config.url, qdrant_config.collection_name).await?
    );

    // Initialize Meilisearch client to fetch existing images
    let search_client = SearchClient::new(&config.meilisearch_url, &config.meilisearch_key)?;

    // Fetch all images from Meilisearch in batches
    let batch_size = 100;
    let mut offset = 0;
    let mut total_indexed = 0;

    loop {
        info!("Fetching batch at offset {}", offset);

        let results = search_client
            .search_images("", batch_size, offset, None, None, None)
            .await?;

        let hits: Vec<serde_json::Value> = results
            .get("hits")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        if hits.is_empty() {
            break;
        }

        // Index each image to Qdrant
        for hit in &hits {
            let id = hit.get("id").and_then(|v| v.as_str()).unwrap_or("");
            let image_url = hit.get("image_url").and_then(|v| v.as_str()).unwrap_or("");
            let source_url = hit.get("source_url").and_then(|v| v.as_str()).unwrap_or("");
            let figcaption = hit.get("figcaption").and_then(|v| v.as_str());
            let alt_text = hit.get("alt_text").and_then(|v| v.as_str());
            let title = hit.get("title").and_then(|v| v.as_str());
            let page_title = hit.get("page_title").and_then(|v| v.as_str()).unwrap_or("");
            let domain = hit.get("domain").and_then(|v| v.as_str()).unwrap_or("");

            match qdrant
                .index_image(id, image_url, source_url, figcaption, alt_text, title, page_title, domain)
                .await
            {
                Ok(_) => {
                    total_indexed += 1;
                    if total_indexed % 50 == 0 {
                        info!("Indexed {} images so far...", total_indexed);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to index image {}: {}", image_url, e);
                }
            }
        }

        offset += batch_size;

        // Check if we've processed all images
        let total_hits = results
            .get("total_hits")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        if offset >= total_hits {
            break;
        }
    }

    info!("Re-indexing complete! Indexed {} images to Qdrant", total_indexed);

    Ok(())
}
