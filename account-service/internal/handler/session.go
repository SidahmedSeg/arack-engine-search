package handler

import (
	"errors"
	"net/http"

	"github.com/rs/zerolog/log"

	"github.com/arack/account-service/internal/domain"
	"github.com/arack/account-service/internal/service"
	"github.com/arack/account-service/pkg/httputil"
)

// SessionResponse is the response for GET /api/session
type SessionResponse struct {
	UserID      string `json:"user_id"`
	Email       string `json:"email"`
	Name        string `json:"name"`
	Picture     string `json:"picture,omitempty"`
	AccessToken string `json:"access_token"`
}

// GetSession returns the current session
func (h *Handler) GetSession(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()

	// Get session cookie
	cookie, err := r.Cookie(h.cookieConfig.Name)
	if err != nil {
		log.Debug().Err(err).Msg("No session cookie")
		httputil.Error(w, http.StatusUnauthorized, "Not authenticated")
		return
	}

	// Validate session
	session, err := h.sessionService.Get(ctx, domain.SessionID(cookie.Value))
	if err != nil {
		if errors.Is(err, service.ErrSessionNotFound) || errors.Is(err, service.ErrSessionExpired) {
			// Clear invalid cookie
			http.SetCookie(w, &http.Cookie{
				Name:     h.cookieConfig.Name,
				Value:    "",
				Domain:   h.cookieConfig.Domain,
				Path:     "/",
				MaxAge:   -1,
				HttpOnly: true,
				Secure:   true,
			})
			httputil.Error(w, http.StatusUnauthorized, "Session expired")
			return
		}

		log.Error().Err(err).Msg("Failed to get session")
		httputil.Error(w, http.StatusInternalServerError, "Failed to validate session")
		return
	}

	httputil.JSON(w, http.StatusOK, SessionResponse{
		UserID:      session.User.ID,
		Email:       session.User.Email,
		Name:        session.User.Name,
		Picture:     session.User.Picture,
		AccessToken: session.AccessToken,
	})
}
