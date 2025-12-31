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
	Success bool         `json:"success"`
	User    *domain.User `json:"user"`
}

// CredentialsLogin handles email/password authentication
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

	// Authenticate with Zitadel
	loginResp, err := h.zitadelService.Login(ctx, &service.LoginRequest{
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
	// Note: For custom login, we create a local session and store the Zitadel session info
	tokens := &service.Tokens{
		AccessToken:  loginResp.SessionToken, // Using session token as access token
		RefreshToken: "",                      // Custom login doesn't provide refresh token directly
		IDToken:      "",
		ExpiresAt:    time.Now().Add(24 * time.Hour), // Default 24h session
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
		Success: true,
		User:    loginResp.User,
	})
}
