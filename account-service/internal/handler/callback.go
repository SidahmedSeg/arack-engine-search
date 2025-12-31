package handler

import (
	"net/http"

	"github.com/rs/zerolog/log"

	"github.com/arack/account-service/pkg/httputil"
)

// Callback handles the OAuth callback
func (h *Handler) Callback(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()

	// Check for error from Zitadel
	if errParam := r.URL.Query().Get("error"); errParam != "" {
		errDesc := r.URL.Query().Get("error_description")
		log.Error().
			Str("error", errParam).
			Str("description", errDesc).
			Msg("OAuth error from provider")
		httputil.Error(w, http.StatusBadRequest, "Authentication failed: "+errDesc)
		return
	}

	// Verify state
	stateCookie, err := r.Cookie("oauth_state")
	if err != nil {
		log.Error().Err(err).Msg("Missing state cookie")
		httputil.Error(w, http.StatusBadRequest, "Invalid state")
		return
	}

	if stateCookie.Value != r.URL.Query().Get("state") {
		log.Error().Msg("State mismatch")
		httputil.Error(w, http.StatusBadRequest, "Invalid state")
		return
	}

	// Get code verifier
	verifierCookie, err := r.Cookie("oauth_verifier")
	if err != nil {
		log.Error().Err(err).Msg("Missing verifier cookie")
		httputil.Error(w, http.StatusBadRequest, "Missing verifier")
		return
	}

	// Exchange code for tokens
	code := r.URL.Query().Get("code")
	if code == "" {
		log.Error().Msg("Missing authorization code")
		httputil.Error(w, http.StatusBadRequest, "Missing authorization code")
		return
	}

	tokens, user, err := h.authService.ExchangeCode(ctx, code, verifierCookie.Value)
	if err != nil {
		log.Error().Err(err).Msg("Failed to exchange code")
		httputil.Error(w, http.StatusInternalServerError, "Authentication failed")
		return
	}

	// Create session
	session, err := h.sessionService.Create(ctx, *user, tokens)
	if err != nil {
		log.Error().Err(err).Msg("Failed to create session")
		httputil.Error(w, http.StatusInternalServerError, "Failed to create session")
		return
	}

	// Set session cookie on .arack.io domain
	http.SetCookie(w, &http.Cookie{
		Name:     h.cookieConfig.Name,
		Value:    session.ID.String(),
		Domain:   h.cookieConfig.Domain,
		Path:     "/",
		MaxAge:   h.cookieConfig.MaxAge,
		HttpOnly: h.cookieConfig.HTTPOnly,
		Secure:   h.cookieConfig.Secure,
		SameSite: getSameSite(h.cookieConfig.SameSite),
	})

	// Clear OAuth cookies
	clearCookie(w, "oauth_state")
	clearCookie(w, "oauth_verifier")

	// Redirect to return URL or default
	returnURL := "https://arack.io"
	if cookie, err := r.Cookie("return_url"); err == nil && cookie.Value != "" {
		returnURL = cookie.Value
		clearCookie(w, "return_url")
	}

	log.Info().
		Str("session_id", session.ID.String()).
		Str("user_id", user.ID).
		Str("email", user.Email).
		Str("return_url", returnURL).
		Msg("OAuth callback successful")

	http.Redirect(w, r, returnURL, http.StatusFound)
}
