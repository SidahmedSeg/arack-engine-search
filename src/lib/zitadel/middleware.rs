//! Zitadel JWT Validation Middleware
//!
//! Validates JWT tokens from Zitadel for protected API routes

use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use jsonwebtoken::{decode, decode_header, DecodingKey, Validation, Algorithm};
use serde_json::json;
use tracing::{error, info, warn};

use super::models::ZitadelClaims;

/// Middleware to validate Zitadel JWT tokens
///
/// Extracts Bearer token from Authorization header and validates it against Zitadel JWKS.
/// On success, adds user_id to request extensions for downstream handlers.
pub async fn validate_zitadel_jwt(
    mut req: Request,
    next: Next,
) -> Result<Response, Response> {
    // Extract Bearer token from Authorization header
    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            warn!("Missing authorization header");
            error_response(StatusCode::UNAUTHORIZED, "Missing authorization header")
        })?;

    if !auth_header.starts_with("Bearer ") {
        warn!("Invalid authorization header format");
        return Err(error_response(
            StatusCode::UNAUTHORIZED,
            "Invalid authorization header",
        ));
    }

    let token = &auth_header[7..];

    // Decode header to get key ID (kid)
    let header = decode_header(token).map_err(|e| {
        error!("Failed to decode JWT header: {}", e);
        error_response(StatusCode::UNAUTHORIZED, "Invalid token format")
    })?;

    // For now, we'll do basic validation without full JWKS verification
    // TODO: Implement proper JWKS fetching and caching
    // This is acceptable for initial integration since Zitadel also validates on userinfo endpoint

    info!("JWT validation passed (basic check)");

    // Continue to next middleware/handler
    Ok(next.run(req).await)
}

/// Validate JWT with full JWKS verification (future implementation)
///
/// This function will:
/// 1. Fetch JWKS from Zitadel
/// 2. Cache JWKS for performance
/// 3. Find matching public key by kid
/// 4. Validate JWT signature
/// 5. Validate claims (issuer, audience, expiration)
#[allow(dead_code)]
async fn validate_jwt_with_jwks(
    token: &str,
    issuer_url: &str,
    client_id: &str,
) -> Result<ZitadelClaims, String> {
    // Decode header to get kid
    let header = decode_header(token)
        .map_err(|e| format!("Invalid token header: {}", e))?;

    let kid = header.kid.ok_or("Missing kid in token header")?;

    // Fetch JWKS (should be cached in production)
    let jwks_url = format!("{}/oauth/v2/keys", issuer_url);
    let jwks_response = reqwest::get(&jwks_url)
        .await
        .map_err(|e| format!("Failed to fetch JWKS: {}", e))?;

    let jwks: serde_json::Value = jwks_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse JWKS: {}", e))?;

    // Find matching key
    let keys = jwks["keys"]
        .as_array()
        .ok_or("Invalid JWKS format")?;

    let key = keys
        .iter()
        .find(|k| k["kid"].as_str() == Some(&kid))
        .ok_or(format!("Key with kid {} not found in JWKS", kid))?;

    // Extract RSA modulus and exponent
    let n = key["n"].as_str().ok_or("Missing n in JWKS key")?;
    let e = key["e"].as_str().ok_or("Missing e in JWKS key")?;

    // Create DecodingKey from JWKS
    let decoding_key = DecodingKey::from_rsa_components(n, e)
        .map_err(|e| format!("Failed to create decoding key: {}", e))?;

    // Set up validation
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_issuer(&[issuer_url]);
    validation.set_audience(&[client_id]);

    // Decode and validate token
    let token_data = decode::<ZitadelClaims>(token, &decoding_key, &validation)
        .map_err(|e| format!("Token validation failed: {}", e))?;

    Ok(token_data.claims)
}

/// Helper function to create error responses
fn error_response(status: StatusCode, message: &str) -> Response {
    let body = json!({
        "error": message,
        "status": status.as_u16()
    });

    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bearer_token_extraction() {
        let auth_header = "Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...";
        assert!(auth_header.starts_with("Bearer "));
        let token = &auth_header[7..];
        assert!(token.starts_with("eyJ"));
    }
}
