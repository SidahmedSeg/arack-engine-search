pub mod types;

use anyhow::{Context, Result};
use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config as BertConfig, DTYPE};
use qdrant_client::{
    Qdrant,
    qdrant::{
        CreateCollectionBuilder, Distance, PointStruct,
        SearchPointsBuilder, UpsertPointsBuilder, Value, VectorParamsBuilder,
    },
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokenizers::{PaddingParams, Tokenizer};
use tokio::sync::Mutex;

const EMBEDDING_DIM: u64 = 384; // all-MiniLM-L6-v2 dimension
const MODEL_ID: &str = "sentence-transformers/all-MiniLM-L6-v2";
const IMAGE_COLLECTION_NAME: &str = "image_embeddings"; // Phase 10.5: Image semantic search collection

pub struct QdrantService {
    client: Qdrant,
    collection_name: String,
    embedder: Arc<Mutex<BertEmbedder>>,
}

struct BertEmbedder {
    model: BertModel,
    tokenizer: Tokenizer,
    device: Device,
}

impl BertEmbedder {
    fn new(_model_path: PathBuf) -> Result<Self> {
        let device = Device::Cpu;

        // Use cached model files directly (bypass hf-hub HTTP/2 issue)
        tracing::info!("Loading BERT model from local cache...");
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        let cache_dir = PathBuf::from(format!(
            "{}/.cache/huggingface/hub/models--sentence-transformers--all-MiniLM-L6-v2/snapshots/c9745ed1d9f207416be6d2e6f8de32d1f16199bf",
            home
        ));

        tracing::info!("Cache directory: {:?}", cache_dir);
        if !cache_dir.exists() {
            anyhow::bail!("Model cache directory not found at {:?}. Please run: mkdir -p {:?}", cache_dir, cache_dir);
        }

        let config_path = cache_dir.join("config.json");
        let tokenizer_path = cache_dir.join("tokenizer.json");
        let weights_path = cache_dir.join("model.safetensors");

        // Verify all files exist
        if !config_path.exists() {
            anyhow::bail!("config.json not found in cache at {:?}", config_path);
        }
        if !tokenizer_path.exists() {
            anyhow::bail!("tokenizer.json not found in cache at {:?}", tokenizer_path);
        }
        if !weights_path.exists() {
            anyhow::bail!("model.safetensors not found in cache at {:?}", weights_path);
        }
        tracing::info!("All model files found in cache");

        // Load config
        tracing::info!("Loading config from {:?}", config_path);
        let config_str = std::fs::read_to_string(&config_path)?;
        let config: BertConfig = serde_json::from_str(&config_str)?;
        tracing::info!("Config loaded successfully");

        // Load tokenizer
        tracing::info!("Loading tokenizer from {:?}", tokenizer_path);
        let mut tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| anyhow::anyhow!("Failed to load tokenizer: {}", e))?;
        tracing::info!("Tokenizer loaded successfully");

        // Configure padding
        tracing::info!("Configuring tokenizer padding and truncation...");
        let pp = PaddingParams {
            strategy: tokenizers::PaddingStrategy::BatchLongest,
            ..Default::default()
        };
        tokenizer
            .with_padding(Some(pp))
            .with_truncation(Some(tokenizers::TruncationParams {
                max_length: 512,
                ..Default::default()
            }))
            .map_err(|_| anyhow::anyhow!("Failed to configure tokenizer"))?;
        tracing::info!("Tokenizer configuration complete");

        // Load model weights
        tracing::info!("Loading model weights from {:?}", weights_path);
        let vb = unsafe {
            VarBuilder::from_mmaped_safetensors(&[weights_path], DTYPE, &device)?
        };
        tracing::info!("VarBuilder created, loading BERT model...");
        let model = BertModel::load(vb, &config)?;
        tracing::info!("BERT model loaded successfully!");

        Ok(Self {
            model,
            tokenizer,
            device,
        })
    }

    fn embed(&self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>> {
        // Tokenize
        let encodings = self
            .tokenizer
            .encode_batch(texts.clone(), true)
            .map_err(|e| anyhow::anyhow!("Tokenization failed: {}", e))?;

        let token_ids = encodings
            .iter()
            .map(|e| {
                Tensor::new(
                    e.get_ids()
                        .iter()
                        .map(|&id| id as u32)
                        .collect::<Vec<_>>()
                        .as_slice(),
                    &self.device,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;

        let token_type_ids = encodings
            .iter()
            .map(|e| {
                Tensor::new(
                    e.get_type_ids()
                        .iter()
                        .map(|&id| id as u32)
                        .collect::<Vec<_>>()
                        .as_slice(),
                    &self.device,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;

        let attention_mask = encodings
            .iter()
            .map(|e| {
                Tensor::new(
                    e.get_attention_mask()
                        .iter()
                        .map(|&m| m as u32)
                        .collect::<Vec<_>>()
                        .as_slice(),
                    &self.device,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;

        // Stack tensors
        let token_ids = Tensor::stack(&token_ids, 0)?;
        let token_type_ids = Tensor::stack(&token_type_ids, 0)?;
        let attention_mask = Tensor::stack(&attention_mask, 0)?;

        // Forward pass
        let embeddings = self
            .model
            .forward(&token_ids, &token_type_ids, Some(&attention_mask))?;

        // Mean pooling with attention mask
        let pooled = self.mean_pool(&embeddings, &attention_mask)?;

        // Normalize embeddings (L2 normalization)
        let normalized = self.normalize(&pooled)?;

        // Convert to Vec<Vec<f32>>
        // Use to_vec2() since normalized is [batch_size, embedding_dim]
        let result: Vec<Vec<f32>> = normalized.to_vec2()?;

        Ok(result)
    }

    fn mean_pool(&self, embeddings: &Tensor, attention_mask: &Tensor) -> Result<Tensor> {
        // Expand attention mask for broadcasting
        let attention_mask = attention_mask
            .to_dtype(DTYPE)?
            .unsqueeze(2)?;

        // Multiply embeddings by attention mask and sum
        let masked = embeddings.broadcast_mul(&attention_mask)?;
        let sum_embeddings = masked.sum(1)?;

        // Calculate sum of attention masks
        let sum_mask = attention_mask.sum(1)?;

        // Avoid division by zero
        let sum_mask = sum_mask.clamp(1e-9, f64::MAX)?;

        // Divide to get mean
        Ok(sum_embeddings.broadcast_div(&sum_mask)?)
    }

    fn normalize(&self, tensor: &Tensor) -> Result<Tensor> {
        let norm = tensor.sqr()?.sum_keepdim(1)?.sqrt()?;
        Ok(tensor.broadcast_div(&norm)?)
    }
}

impl QdrantService {
    /// Temporary placeholder to allow service to start without BERT model
    pub fn placeholder() -> Self {
        use qdrant_client::Qdrant;
        use std::path::PathBuf;
        use candle_core::{Device, DType};
        use candle_nn::VarBuilder;

        tracing::warn!("Using Qdrant placeholder - semantic search disabled");

        // Create a minimal dummy client (won't be used)
        let client = Qdrant::from_url("http://localhost:6333").build().unwrap();

        // Create a dummy embedder with minimal valid components
        let config = candle_transformers::models::bert::Config::default();
        let vb = VarBuilder::zeros(DType::F32, &Device::Cpu);
        let dummy_model = BertModel::load(vb, &config).unwrap();
        let dummy_tokenizer = tokenizers::Tokenizer::new(tokenizers::models::wordpiece::WordPiece::default());

        let embedder = BertEmbedder {
            model: dummy_model,
            tokenizer: dummy_tokenizer,
            device: Device::Cpu,
        };

        Self {
            client,
            collection_name: "pages".to_string(),
            embedder: Arc::new(Mutex::new(embedder)),
        }
    }

    pub async fn new(url: &str, collection_name: String) -> Result<Self> {
        // Initialize Qdrant client
        let client = Qdrant::from_url(url)
            .build()
            .context("Failed to create Qdrant client")?;

        // Initialize BERT embedder
        tracing::info!("Initializing BERT embedder (downloading model from HuggingFace...)");
        let model_path = PathBuf::from("."); // Not used, handled by hf-hub
        let embedder = tokio::task::spawn_blocking(move || {
            BertEmbedder::new(model_path)
        })
        .await
        .context("Failed to spawn blocking task")?
        .context("Failed to initialize BERT embedder")?;

        let service = Self {
            client,
            collection_name: collection_name.clone(),
            embedder: Arc::new(Mutex::new(embedder)),
        };

        // Create collections if not exist
        service.ensure_collection().await?;
        service.ensure_image_collection().await?; // Phase 10.5: Image semantic search

        tracing::info!(
            "Qdrant service initialized with collections: {} (pages), {} (images)",
            collection_name,
            IMAGE_COLLECTION_NAME
        );

        Ok(service)
    }

    async fn ensure_collection(&self) -> Result<()> {
        // Check if collection exists
        let collections = self.client.list_collections().await?;
        let exists = collections
            .collections
            .iter()
            .any(|c| c.name == self.collection_name);

        if !exists {
            tracing::info!("Creating Qdrant collection: {}", self.collection_name);

            self.client
                .create_collection(
                    CreateCollectionBuilder::new(&self.collection_name)
                        .vectors_config(VectorParamsBuilder::new(EMBEDDING_DIM, Distance::Cosine)),
                )
                .await
                .context("Failed to create collection")?;
        }

        Ok(())
    }

    /// Phase 10.5: Ensure image collection exists
    async fn ensure_image_collection(&self) -> Result<()> {
        // Check if collection exists
        let collections = self.client.list_collections().await?;
        let exists = collections
            .collections
            .iter()
            .any(|c| c.name == IMAGE_COLLECTION_NAME);

        if !exists {
            tracing::info!("Creating Qdrant image collection: {}", IMAGE_COLLECTION_NAME);

            self.client
                .create_collection(
                    CreateCollectionBuilder::new(IMAGE_COLLECTION_NAME)
                        .vectors_config(VectorParamsBuilder::new(EMBEDDING_DIM, Distance::Cosine)),
                )
                .await
                .context("Failed to create image collection")?;
        }

        Ok(())
    }

    /// Generate embedding for a single text
    pub async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let embedder = self.embedder.lock().await;
        let embeddings = embedder.embed(vec![text])?;

        embeddings
            .into_iter()
            .next()
            .context("No embedding generated")
    }

    /// Index a page with its embedding
    pub async fn index_page(
        &self,
        page_id: &str,
        url: &str,
        title: &str,
        content: &str,
    ) -> Result<()> {
        // Combine title + content for richer embeddings (limit to first 512 tokens ~2048 chars)
        let text = format!("{}\n\n{}", title, content);
        let truncated_text = if text.len() > 2048 {
            &text[..2048]
        } else {
            &text
        };

        // Generate embedding
        let embedding = self.generate_embedding(truncated_text).await?;

        // Create payload
        let mut payload = HashMap::new();
        payload.insert("url".to_string(), Value::from(url));
        payload.insert("title".to_string(), Value::from(title));

        // Create point with metadata
        let point = PointStruct::new(
            page_id.to_string(),
            embedding,
            payload,
        );

        // Upsert to Qdrant
        self.client
            .upsert_points(
                UpsertPointsBuilder::new(&self.collection_name, vec![point])
            )
            .await
            .context("Failed to upsert point to Qdrant")?;

        Ok(())
    }

    /// Search for similar pages
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<ScoredPage>> {
        // Generate query embedding
        let query_embedding = self.generate_embedding(query).await?;

        // Search Qdrant
        let search_result = self.client
            .search_points(
                SearchPointsBuilder::new(&self.collection_name, query_embedding, limit as u64)
                    .with_payload(true)
            )
            .await
            .context("Failed to search Qdrant")?;

        // Extract results
        let results = search_result
            .result
            .into_iter()
            .filter_map(|scored_point| {
                use qdrant_client::qdrant::value::Kind;

                let payload = scored_point.payload;

                // Extract URL from payload
                let url = match payload.get("url")?.kind.as_ref()? {
                    Kind::StringValue(s) => s.clone(),
                    _ => return None,
                };

                // Extract title from payload
                let title = match payload.get("title")?.kind.as_ref()? {
                    Kind::StringValue(s) => s.clone(),
                    _ => return None,
                };

                // Extract ID
                let id = match scored_point.id? {
                    qdrant_client::qdrant::PointId { point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Uuid(uuid)) } => uuid,
                    qdrant_client::qdrant::PointId { point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Num(num)) } => num.to_string(),
                    _ => return None,
                };

                Some(ScoredPage {
                    id,
                    url,
                    title,
                    score: scored_point.score,
                })
            })
            .collect();

        Ok(results)
    }

    /// Delete a page by ID
    pub async fn delete_page(&self, page_id: &str) -> Result<()> {
        use qdrant_client::qdrant::{DeletePointsBuilder, PointsIdsList};

        self.client
            .delete_points(
                DeletePointsBuilder::new(&self.collection_name)
                    .points(PointsIdsList {
                        ids: vec![qdrant_client::qdrant::PointId::from(page_id)],
                    })
            )
            .await
            .context("Failed to delete point from Qdrant")?;

        Ok(())
    }

    /// Phase 10.5: Index an image with its embedding
    /// Combines: figcaption + alt_text + title + page_title for rich semantic context
    pub async fn index_image(
        &self,
        image_id: &str,
        image_url: &str,
        source_url: &str,
        figcaption: Option<&str>,
        alt_text: Option<&str>,
        title: Option<&str>,
        page_title: &str,
        domain: &str,
    ) -> Result<()> {
        // Build combined text for embedding (prioritize semantic fields)
        let mut text_parts = Vec::new();

        if let Some(fig) = figcaption {
            if !fig.trim().is_empty() {
                text_parts.push(fig);
            }
        }
        if let Some(alt) = alt_text {
            if !alt.trim().is_empty() {
                text_parts.push(alt);
            }
        }
        if let Some(t) = title {
            if !t.trim().is_empty() {
                text_parts.push(t);
            }
        }
        if !page_title.trim().is_empty() {
            text_parts.push(page_title);
        }

        // Fallback: if no text available, use domain + URL
        if text_parts.is_empty() {
            text_parts.push(domain);
            text_parts.push(image_url);
        }

        let combined_text = text_parts.join(" ");

        // Generate embedding
        let embedding = self.generate_embedding(&combined_text).await?;

        // Create payload
        let mut payload = HashMap::new();
        payload.insert("image_url".to_string(), Value::from(image_url));
        payload.insert("source_url".to_string(), Value::from(source_url));
        payload.insert("domain".to_string(), Value::from(domain));

        // Create point with metadata
        let point = PointStruct::new(
            image_id.to_string(),
            embedding,
            payload,
        );

        // Upsert to Qdrant
        self.client
            .upsert_points(
                UpsertPointsBuilder::new(IMAGE_COLLECTION_NAME, vec![point])
            )
            .await
            .context("Failed to upsert image to Qdrant")?;

        Ok(())
    }

    /// Phase 10.5: Search for similar images by semantic query
    pub async fn search_images(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<ScoredImage>> {
        // Generate query embedding
        let query_embedding = self.generate_embedding(query).await?;

        // Search Qdrant image collection
        let search_result = self.client
            .search_points(
                SearchPointsBuilder::new(IMAGE_COLLECTION_NAME, query_embedding, limit as u64)
                    .with_payload(true)
            )
            .await
            .context("Failed to search Qdrant for images")?;

        // Extract results
        let results = search_result
            .result
            .into_iter()
            .filter_map(|scored_point| {
                use qdrant_client::qdrant::value::Kind;

                let payload = scored_point.payload;

                // Extract image_url from payload
                let image_url = match payload.get("image_url")?.kind.as_ref()? {
                    Kind::StringValue(s) => s.clone(),
                    _ => return None,
                };

                // Extract source_url from payload
                let source_url = match payload.get("source_url")?.kind.as_ref()? {
                    Kind::StringValue(s) => s.clone(),
                    _ => return None,
                };

                // Extract domain from payload
                let domain = match payload.get("domain")?.kind.as_ref()? {
                    Kind::StringValue(s) => s.clone(),
                    _ => return None,
                };

                // Extract ID
                let id = match scored_point.id? {
                    qdrant_client::qdrant::PointId { point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Uuid(uuid)) } => uuid,
                    qdrant_client::qdrant::PointId { point_id_options: Some(qdrant_client::qdrant::point_id::PointIdOptions::Num(num)) } => num.to_string(),
                    _ => return None,
                };

                Some(ScoredImage {
                    id,
                    image_url,
                    source_url,
                    domain,
                    score: scored_point.score,
                })
            })
            .collect();

        Ok(results)
    }

    /// Phase 10.5: Delete an image by ID
    pub async fn delete_image(&self, image_id: &str) -> Result<()> {
        use qdrant_client::qdrant::{DeletePointsBuilder, PointsIdsList};

        self.client
            .delete_points(
                DeletePointsBuilder::new(IMAGE_COLLECTION_NAME)
                    .points(PointsIdsList {
                        ids: vec![qdrant_client::qdrant::PointId::from(image_id)],
                    })
            )
            .await
            .context("Failed to delete image from Qdrant")?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ScoredPage {
    pub id: String,
    pub url: String,
    pub title: String,
    pub score: f32,
}

/// Phase 10.5: Scored image result from semantic search
#[derive(Debug, Clone)]
pub struct ScoredImage {
    pub id: String,
    pub image_url: String,
    pub source_url: String,
    pub domain: String,
    pub score: f32,
}
