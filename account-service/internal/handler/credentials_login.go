package handler

import (
	"encoding/json"
	"net/http"
	"time"

	"github.com/rs/zerolog/log"

	"github.com/arack/account-service/internal/domain"
	"github.com/arack/account-service/internal/service"
	"github.com/arack/account-service/pkg/httputil"
)

// CredentialsLoginRequest represents an email/password login request
type CredentialsLoginRequest struct {
	Email    string `json:"email"`
	Password string `json:"password"`
}

// CredentialsLoginResponse represents a successful login response
type CredentialsLoginResponse struct {
	Success      bool         `json:"success"`
	User         *domain.User `json:"user"`
	AccessToken  string       `json:"accessToken"`
	RefreshToken string       `json:"refreshToken,omitempty"`
	ExpiresIn    int          `json:"expiresIn"`
}

// CredentialsLogin handles email/password authentication using local auth
func (h *Handler) CredentialsLogin(w http.ResponseWriter, r *http.Request) {
	var req CredentialsLoginRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		httputil.Error(w, http.StatusBadRequest, "Invalid request body")
		return
	}

	// Validate input
	if req.Email == "" || req.Password == "" {
		httputil.Error(w, http.StatusBadRequest, "Email and password are required")
		return
	}

	ctx := r.Context()

	// Authenticate with local auth service
	loginResp, err := h.localAuthService.Login(ctx, &service.LoginUserRequest{
		Email:    req.Email,
		Password: req.Password,
	})
	if err != nil {
		log.Warn().
			Err(err).
			Str("email", req.Email).
			Msg("Login failed")
		httputil.Error(w, http.StatusUnauthorized, "Invalid email or password")
		return
	}

	// Create session with tokens
	tokens := &service.Tokens{
		AccessToken:  loginResp.Tokens.AccessToken,
		RefreshToken: loginResp.Tokens.RefreshToken,
		IDToken:      "",
		ExpiresAt:    loginResp.Tokens.ExpiresAt,
	}

	session, err := h.sessionService.Create(ctx, *loginResp.User, tokens)
	if err != nil {
		log.Error().
			Err(err).
			Str("email", req.Email).
			Msg("Failed to create session")
		httputil.Error(w, http.StatusInternalServerError, "Failed to create session")
		return
	}

	// Set session cookie
	http.SetCookie(w, &http.Cookie{
		Name:     h.cookieConfig.Name,
		Value:    session.ID.String(),
		Path:     "/",
		Domain:   h.cookieConfig.Domain,
		MaxAge:   h.cookieConfig.MaxAge,
		HttpOnly: h.cookieConfig.HTTPOnly,
		Secure:   h.cookieConfig.Secure,
		SameSite: getSameSite(h.cookieConfig.SameSite),
	})

	log.Info().
		Str("email", req.Email).
		Str("session_id", session.ID.String()).
		Msg("User logged in via credentials")

	httputil.JSON(w, http.StatusOK, CredentialsLoginResponse{
		Success:      true,
		User:         loginResp.User,
		AccessToken:  loginResp.Tokens.AccessToken,
		RefreshToken: loginResp.Tokens.RefreshToken,
		ExpiresIn:    loginResp.Tokens.ExpiresIn,
	})
}

// Tokens represents OAuth tokens for session storage
// This is needed because service.Tokens is internal
func tokensFromLoginResponse(tokens *service.TokenPair) *service.Tokens {
	return &service.Tokens{
		AccessToken:  tokens.AccessToken,
		RefreshToken: tokens.RefreshToken,
		IDToken:      "",
		ExpiresAt:    time.Now().Add(time.Duration(tokens.ExpiresIn) * time.Second),
	}
}
