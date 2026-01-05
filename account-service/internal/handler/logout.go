package handler

import (
	"net/http"

	"github.com/rs/zerolog/log"

	"github.com/arack/account-service/internal/domain"
	"github.com/arack/account-service/pkg/httputil"
)

// Logout destroys the session
func (h *Handler) Logout(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()

	// Get session cookie
	cookie, err := r.Cookie(h.cookieConfig.Name)
	if err == nil && cookie.Value != "" {
		// Destroy session
		if err := h.sessionService.Destroy(ctx, domain.SessionID(cookie.Value)); err != nil {
			log.Warn().Err(err).Msg("Failed to destroy session")
		}
	}

	// Clear session cookie
	http.SetCookie(w, &http.Cookie{
		Name:     h.cookieConfig.Name,
		Value:    "",
		Domain:   h.cookieConfig.Domain,
		Path:     "/",
		MaxAge:   -1,
		HttpOnly: true,
		Secure:   true,
		SameSite: getSameSite(h.cookieConfig.SameSite),
	})

	log.Info().Msg("User logged out")

	// Return success or redirect
	if returnURL := r.URL.Query().Get("return_url"); returnURL != "" {
		http.Redirect(w, r, returnURL, http.StatusFound)
		return
	}

	httputil.JSON(w, http.StatusOK, map[string]string{"status": "logged_out"})
}
