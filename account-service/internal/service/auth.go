package service

import (
	"context"
	"fmt"
	"net/http"
	"net/url"
	"strings"
	"time"

	"github.com/coreos/go-oidc/v3/oidc"
	"golang.org/x/oauth2"

	"github.com/arack/account-service/internal/config"
	"github.com/arack/account-service/internal/domain"
)

// Tokens holds OAuth tokens
type Tokens struct {
	AccessToken  string
	RefreshToken string
	IDToken      string
	ExpiresAt    time.Time
}

// AuthService handles OAuth/OIDC operations
type AuthService struct {
	provider     *oidc.Provider
	oauth2Config *oauth2.Config
	verifier     *oidc.IDTokenVerifier
	issuerURL    string
}

// NewAuthService creates a new auth service
func NewAuthService(ctx context.Context, cfg *config.OAuthConfig) (*AuthService, error) {
	provider, err := oidc.NewProvider(ctx, cfg.IssuerURL)
	if err != nil {
		return nil, fmt.Errorf("create OIDC provider: %w", err)
	}

	oauth2Config := &oauth2.Config{
		ClientID:    cfg.ClientID,
		Endpoint:    provider.Endpoint(),
		RedirectURL: cfg.RedirectURL,
		Scopes:      cfg.Scopes,
	}

	verifier := provider.Verifier(&oidc.Config{ClientID: cfg.ClientID})

	return &AuthService{
		provider:     provider,
		oauth2Config: oauth2Config,
		verifier:     verifier,
		issuerURL:    cfg.IssuerURL,
	}, nil
}

// GetAuthURL generates the OAuth authorization URL with PKCE
func (s *AuthService) GetAuthURL(state, codeVerifier string) string {
	return s.oauth2Config.AuthCodeURL(
		state,
		oauth2.S256ChallengeOption(codeVerifier),
	)
}

// ExchangeCode exchanges an authorization code for tokens
func (s *AuthService) ExchangeCode(ctx context.Context, code, codeVerifier string) (*Tokens, *domain.User, error) {
	token, err := s.oauth2Config.Exchange(ctx, code, oauth2.VerifierOption(codeVerifier))
	if err != nil {
		return nil, nil, fmt.Errorf("exchange code: %w", err)
	}

	rawIDToken, ok := token.Extra("id_token").(string)
	if !ok {
		return nil, nil, fmt.Errorf("no id_token in response")
	}

	idToken, err := s.verifier.Verify(ctx, rawIDToken)
	if err != nil {
		return nil, nil, fmt.Errorf("verify id_token: %w", err)
	}

	var claims struct {
		Sub     string `json:"sub"`
		Email   string `json:"email"`
		Name    string `json:"name"`
		Picture string `json:"picture"`
	}
	if err := idToken.Claims(&claims); err != nil {
		return nil, nil, fmt.Errorf("parse claims: %w", err)
	}

	user := &domain.User{
		ID:      claims.Sub,
		Email:   claims.Email,
		Name:    claims.Name,
		Picture: claims.Picture,
	}

	tokens := &Tokens{
		AccessToken:  token.AccessToken,
		RefreshToken: token.RefreshToken,
		IDToken:      rawIDToken,
		ExpiresAt:    token.Expiry,
	}

	return tokens, user, nil
}

// RefreshToken refreshes the access token
func (s *AuthService) RefreshToken(ctx context.Context, refreshToken string) (*Tokens, error) {
	token := &oauth2.Token{RefreshToken: refreshToken}
	newToken, err := s.oauth2Config.TokenSource(ctx, token).Token()
	if err != nil {
		return nil, fmt.Errorf("refresh token: %w", err)
	}

	return &Tokens{
		AccessToken:  newToken.AccessToken,
		RefreshToken: newToken.RefreshToken,
		ExpiresAt:    newToken.Expiry,
	}, nil
}

// RevokeToken revokes the token at Zitadel
func (s *AuthService) RevokeToken(ctx context.Context, token string) error {
	// Zitadel revocation endpoint
	revokeURL := strings.TrimSuffix(s.issuerURL, "/") + "/oauth/v2/revoke"

	data := url.Values{}
	data.Set("token", token)
	data.Set("client_id", s.oauth2Config.ClientID)

	req, err := http.NewRequestWithContext(ctx, http.MethodPost, revokeURL, strings.NewReader(data.Encode()))
	if err != nil {
		return fmt.Errorf("create request: %w", err)
	}
	req.Header.Set("Content-Type", "application/x-www-form-urlencoded")

	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		return fmt.Errorf("revoke request: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode >= 400 {
		return fmt.Errorf("revoke failed with status: %d", resp.StatusCode)
	}

	return nil
}
