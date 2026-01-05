package handler

import (
	"net/http"
	"strings"

	"github.com/arack/account-service/pkg/httputil"
)

// JWKS returns the JSON Web Key Set for token validation
// GET /.well-known/jwks.json
func (h *Handler) JWKS(w http.ResponseWriter, r *http.Request) {
	jwks, err := h.localAuthService.GetJWKS()
	if err != nil {
		httputil.Error(w, http.StatusInternalServerError, "Failed to get JWKS")
		return
	}

	w.Header().Set("Content-Type", "application/json")
	w.Header().Set("Cache-Control", "public, max-age=3600")
	w.Write(jwks)
}

// OpenIDConfiguration returns the OpenID Connect discovery document
// GET /.well-known/openid-configuration
func (h *Handler) OpenIDConfiguration(w http.ResponseWriter, r *http.Request) {
	issuer := "https://account.arack.io"

	config := map[string]interface{}{
		"issuer":                                issuer,
		"authorization_endpoint":                issuer + "/oauth/authorize",
		"token_endpoint":                        issuer + "/oauth/token",
		"userinfo_endpoint":                     issuer + "/userinfo",
		"jwks_uri":                              issuer + "/.well-known/jwks.json",
		"response_types_supported":              []string{"code", "token"},
		"subject_types_supported":               []string{"public"},
		"id_token_signing_alg_values_supported": []string{"RS256"},
		"scopes_supported":                      []string{"openid", "email", "profile"},
		"token_endpoint_auth_methods_supported": []string{"none", "client_secret_post"},
		"claims_supported":                      []string{"sub", "email", "name", "iss", "aud", "exp", "iat"},
	}

	httputil.JSON(w, http.StatusOK, config)
}

// UserInfo returns the authenticated user's information
// GET /userinfo
// Requires: Authorization: Bearer <token>
func (h *Handler) UserInfo(w http.ResponseWriter, r *http.Request) {
	// Extract token from Authorization header
	authHeader := r.Header.Get("Authorization")
	if authHeader == "" {
		httputil.Error(w, http.StatusUnauthorized, "Missing Authorization header")
		return
	}

	// Expect "Bearer <token>"
	parts := strings.SplitN(authHeader, " ", 2)
	if len(parts) != 2 || strings.ToLower(parts[0]) != "bearer" {
		httputil.Error(w, http.StatusUnauthorized, "Invalid Authorization header format")
		return
	}

	token := parts[1]

	// Validate token and get user info
	userInfo, err := h.localAuthService.GetUserInfo(token)
	if err != nil {
		httputil.Error(w, http.StatusUnauthorized, "Invalid or expired token")
		return
	}

	httputil.JSON(w, http.StatusOK, userInfo)
}

// TokenRefresh refreshes an access token using a refresh token
// POST /oauth/token (grant_type=refresh_token)
func (h *Handler) TokenRefresh(w http.ResponseWriter, r *http.Request) {
	if err := r.ParseForm(); err != nil {
		httputil.Error(w, http.StatusBadRequest, "Invalid form data")
		return
	}

	grantType := r.FormValue("grant_type")
	if grantType != "refresh_token" {
		httputil.Error(w, http.StatusBadRequest, "Unsupported grant type")
		return
	}

	refreshToken := r.FormValue("refresh_token")
	if refreshToken == "" {
		httputil.Error(w, http.StatusBadRequest, "Missing refresh_token")
		return
	}

	ctx := r.Context()

	tokens, err := h.localAuthService.RefreshTokens(ctx, refreshToken)
	if err != nil {
		httputil.Error(w, http.StatusUnauthorized, "Invalid or expired refresh token")
		return
	}

	httputil.JSON(w, http.StatusOK, map[string]interface{}{
		"access_token":  tokens.AccessToken,
		"refresh_token": tokens.RefreshToken,
		"token_type":    tokens.TokenType,
		"expires_in":    tokens.ExpiresIn,
	})
}
