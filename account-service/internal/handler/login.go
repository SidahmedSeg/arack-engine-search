package handler

import (
	"net/http"

	"github.com/rs/zerolog/log"
)

// Login initiates the OAuth flow
func (h *Handler) Login(w http.ResponseWriter, r *http.Request) {
	// Generate state (CSRF protection)
	state := generateRandomString(32)

	// Generate PKCE code verifier
	codeVerifier := generateRandomString(64)

	// Store state and verifier in secure cookies (short-lived)
	http.SetCookie(w, &http.Cookie{
		Name:     "oauth_state",
		Value:    state,
		Path:     "/",
		MaxAge:   300, // 5 minutes
		HttpOnly: true,
		Secure:   true,
		SameSite: http.SameSiteLaxMode,
	})

	http.SetCookie(w, &http.Cookie{
		Name:     "oauth_verifier",
		Value:    codeVerifier,
		Path:     "/",
		MaxAge:   300,
		HttpOnly: true,
		Secure:   true,
		SameSite: http.SameSiteLaxMode,
	})

	// Store return URL if provided
	if returnURL := r.URL.Query().Get("return_url"); returnURL != "" {
		http.SetCookie(w, &http.Cookie{
			Name:     "return_url",
			Value:    returnURL,
			Path:     "/",
			MaxAge:   300,
			HttpOnly: true,
			Secure:   true,
			SameSite: http.SameSiteLaxMode,
		})
	}

	// Redirect to Zitadel
	authURL := h.authService.GetAuthURL(state, codeVerifier)

	log.Info().
		Str("return_url", r.URL.Query().Get("return_url")).
		Msg("Starting OAuth flow")

	http.Redirect(w, r, authURL, http.StatusFound)
}
