use anyhow::Result;
use chrono::{DateTime, Utc};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

/// Represents an extracted image with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageData {
    pub id: String,
    pub image_url: String,
    pub source_url: String,
    pub alt_text: Option<String>,
    pub title: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub page_title: String,
    pub page_content: String,
    pub domain: String,
    pub crawled_at: DateTime<Utc>,
    // Rich image signals (Priority 1)
    #[serde(default)]  // Default to false if missing (backward compatibility)
    pub is_og_image: bool,           // True if from Open Graph metadata
    pub figcaption: Option<String>,  // Caption from <figcaption> if available
    pub srcset_url: Option<String>,  // Highest resolution URL from srcset
}

/// Extracts images from HTML content
pub struct ImageExtractor;

impl ImageExtractor {
    /// Extract all valid images from HTML
    pub fn extract_images(
        html: &str,
        source_url: &str,
        page_title: &str,
        page_content: &str,
    ) -> Result<Vec<ImageData>> {
        let document = Html::parse_document(html);

        let base_url = Url::parse(source_url)?;
        let domain = base_url.host_str().unwrap_or("unknown").to_string();

        let mut images = Vec::new();
        let crawled_at = Utc::now();

        // Truncate page content for storage (500 chars max)
        // Use char-based truncation to avoid UTF-8 boundary panics
        let truncated_content = if page_content.chars().count() > 500 {
            let truncated: String = page_content.chars().take(500).collect();
            format!("{}...", truncated)
        } else {
            page_content.to_string()
        };

        // Priority 1: Extract Open Graph images (highest quality)
        if let Some(og_image) = Self::extract_og_image(&document, &base_url, source_url, page_title, &truncated_content, &domain, crawled_at) {
            images.push(og_image);
        }

        // Extract regular <img> tags
        let img_selector = Selector::parse("img").unwrap();

        for element in document.select(&img_selector) {
            // Extract src attribute
            let src = match element.value().attr("src") {
                Some(s) => s,
                None => continue, // Skip images without src
            };

            // Skip data URIs (base64 encoded images)
            if src.starts_with("data:") {
                continue;
            }

            // Skip SVGs (optional - can be included if needed)
            if src.ends_with(".svg") {
                continue;
            }

            // Convert relative URL to absolute
            let image_url = match Self::make_absolute_url(&base_url, src) {
                Ok(url) => url,
                Err(_) => continue, // Skip invalid URLs
            };

            // Extract attributes
            let alt_text = element.value().attr("alt").map(|s| s.to_string());
            let title = element.value().attr("title").map(|s| s.to_string());

            let width = element
                .value()
                .attr("width")
                .and_then(|w| w.parse::<u32>().ok());

            let height = element
                .value()
                .attr("height")
                .and_then(|h| h.parse::<u32>().ok());

            // Filter out tiny images (likely icons, tracking pixels)
            // Skip if both dimensions are known and either is < 100px
            if let (Some(w), Some(h)) = (width, height) {
                if w < 100 || h < 100 {
                    continue;
                }
            }

            // Skip common tracking/analytics images
            if Self::is_tracking_image(&image_url) {
                continue;
            }

            // Priority 1: Extract figcaption if image is inside <figure>
            let figcaption = Self::extract_figcaption(&element, &document);

            // Priority 1: Parse srcset and get highest resolution URL
            let srcset_url = element
                .value()
                .attr("srcset")
                .and_then(|srcset| Self::parse_srcset(srcset, &base_url));

            let image_data = ImageData {
                id: Uuid::new_v4().to_string(),
                image_url: image_url.clone(),
                source_url: source_url.to_string(),
                alt_text,
                title,
                width,
                height,
                page_title: page_title.to_string(),
                page_content: truncated_content.clone(),
                domain: domain.clone(),
                crawled_at,
                is_og_image: false,
                figcaption,
                srcset_url,
            };

            images.push(image_data);
        }

        Ok(images)
    }

    /// Convert relative URL to absolute URL
    fn make_absolute_url(base: &Url, relative: &str) -> Result<String> {
        let absolute = base.join(relative)?;
        Ok(absolute.to_string())
    }

    /// Check if URL appears to be a tracking/analytics image
    fn is_tracking_image(url: &str) -> bool {
        let url_lower = url.to_lowercase();

        // Common tracking domains
        let tracking_patterns = [
            "google-analytics.com",
            "facebook.com/tr",
            "doubleclick.net",
            "pixel",
            "tracker",
            "analytics",
            "beacon",
            "1x1.gif",
            "blank.gif",
            "transparent.gif",
        ];

        tracking_patterns.iter().any(|pattern| url_lower.contains(pattern))
    }

    /// Extract Open Graph image metadata (Priority 1: High quality images)
    fn extract_og_image(
        document: &Html,
        base_url: &Url,
        source_url: &str,
        page_title: &str,
        page_content: &str,
        domain: &str,
        crawled_at: DateTime<Utc>,
    ) -> Option<ImageData> {
        // Look for og:image meta tag
        let meta_selector = Selector::parse("meta[property='og:image']").ok()?;
        let og_image_element = document.select(&meta_selector).next()?;
        let og_image_url = og_image_element.value().attr("content")?;

        // Convert to absolute URL
        let image_url = Self::make_absolute_url(base_url, og_image_url).ok()?;

        // Skip tracking images
        if Self::is_tracking_image(&image_url) {
            return None;
        }

        // Extract additional OG metadata
        let og_image_alt_selector = Selector::parse("meta[property='og:image:alt']").ok()?;
        let alt_text = document
            .select(&og_image_alt_selector)
            .next()
            .and_then(|el| el.value().attr("content"))
            .map(|s| s.to_string());

        let og_image_width_selector = Selector::parse("meta[property='og:image:width']").ok()?;
        let width = document
            .select(&og_image_width_selector)
            .next()
            .and_then(|el| el.value().attr("content"))
            .and_then(|w| w.parse::<u32>().ok());

        let og_image_height_selector = Selector::parse("meta[property='og:image:height']").ok()?;
        let height = document
            .select(&og_image_height_selector)
            .next()
            .and_then(|el| el.value().attr("content"))
            .and_then(|h| h.parse::<u32>().ok());

        Some(ImageData {
            id: Uuid::new_v4().to_string(),
            image_url,
            source_url: source_url.to_string(),
            alt_text,
            title: Some("Open Graph Image".to_string()),
            width,
            height,
            page_title: page_title.to_string(),
            page_content: page_content.to_string(),
            domain: domain.to_string(),
            crawled_at,
            is_og_image: true,
            figcaption: None,
            srcset_url: None,
        })
    }

    /// Extract figcaption text if image is inside a <figure> element (Priority 1)
    fn extract_figcaption(
        img_element: &scraper::ElementRef,
        document: &Html,
    ) -> Option<String> {
        // Try to find parent <figure> element
        // Note: scraper doesn't have direct parent traversal, so we'll search for figure elements
        let figure_selector = Selector::parse("figure").ok()?;
        let figcaption_selector = Selector::parse("figcaption").ok()?;

        for figure in document.select(&figure_selector) {
            // Check if this figure contains our img by comparing their HTML
            let figure_html = figure.html();
            let img_html = img_element.html();

            if figure_html.contains(&img_html) {
                // Found the parent figure, now extract figcaption
                for caption in figure.select(&figcaption_selector) {
                    let caption_text: String = caption.text().collect::<Vec<_>>().join(" ");
                    let trimmed = caption_text.trim();
                    if !trimmed.is_empty() {
                        return Some(trimmed.to_string());
                    }
                }
            }
        }

        None
    }

    /// Parse srcset attribute and return the highest resolution URL (Priority 1)
    fn parse_srcset(srcset: &str, base_url: &Url) -> Option<String> {
        // srcset format: "url1 descriptor1, url2 descriptor2, ..."
        // Descriptors can be: 1x, 2x, 3x (density) or 800w, 1200w (width)

        let mut max_descriptor = 0.0;
        let mut max_url = None;

        for source in srcset.split(',') {
            let parts: Vec<&str> = source.trim().split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            let url = parts[0];
            let descriptor = if parts.len() > 1 {
                parts[1]
            } else {
                "1x" // Default descriptor
            };

            // Parse descriptor value
            let value = if descriptor.ends_with('x') {
                // Density descriptor (1x, 2x, 3x)
                descriptor.trim_end_matches('x').parse::<f32>().unwrap_or(1.0)
            } else if descriptor.ends_with('w') {
                // Width descriptor (800w, 1200w) - treat as pixels
                descriptor.trim_end_matches('w').parse::<f32>().unwrap_or(0.0)
            } else {
                1.0
            };

            if value > max_descriptor {
                max_descriptor = value;
                max_url = Some(url);
            }
        }

        // Convert to absolute URL
        max_url.and_then(|url| Self::make_absolute_url(base_url, url).ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_images_basic() {
        let html = r#"
            <html>
                <body>
                    <img src="https://example.com/photo.jpg" alt="A photo" width="800" height="600">
                    <img src="/relative/image.png" alt="Relative image">
                </body>
            </html>
        "#;

        let images = ImageExtractor::extract_images(
            html,
            "https://example.com/page",
            "Test Page",
            "Test content",
        ).unwrap();

        assert_eq!(images.len(), 2);
        assert_eq!(images[0].image_url, "https://example.com/photo.jpg");
        assert_eq!(images[0].alt_text, Some("A photo".to_string()));
        assert_eq!(images[0].width, Some(800));
        assert_eq!(images[0].height, Some(600));
    }

    #[test]
    fn test_filter_small_images() {
        let html = r#"
            <html>
                <body>
                    <img src="https://example.com/big.jpg" width="1920" height="1080">
                    <img src="https://example.com/tiny.jpg" width="50" height="50">
                    <img src="https://example.com/icon.jpg" width="16" height="16">
                </body>
            </html>
        "#;

        let images = ImageExtractor::extract_images(
            html,
            "https://example.com/page",
            "Test Page",
            "Test content",
        ).unwrap();

        // Only the big image should be extracted
        assert_eq!(images.len(), 1);
        assert!(images[0].image_url.contains("big.jpg"));
    }

    #[test]
    fn test_skip_data_uri() {
        let html = r#"
            <html>
                <body>
                    <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==">
                    <img src="https://example.com/photo.jpg">
                </body>
            </html>
        "#;

        let images = ImageExtractor::extract_images(
            html,
            "https://example.com/page",
            "Test Page",
            "Test content",
        ).unwrap();

        // Only the regular image should be extracted
        assert_eq!(images.len(), 1);
        assert!(images[0].image_url.contains("photo.jpg"));
    }

    #[test]
    fn test_skip_tracking_images() {
        let html = r#"
            <html>
                <body>
                    <img src="https://www.google-analytics.com/collect?v=1">
                    <img src="https://example.com/pixel.gif" width="1" height="1">
                    <img src="https://example.com/photo.jpg">
                </body>
            </html>
        "#;

        let images = ImageExtractor::extract_images(
            html,
            "https://example.com/page",
            "Test Page",
            "Test content",
        ).unwrap();

        // Only the photo should be extracted
        assert_eq!(images.len(), 1);
        assert!(images[0].image_url.contains("photo.jpg"));
    }
}
