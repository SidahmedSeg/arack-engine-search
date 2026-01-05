package handler

import (
	"crypto/rand"
	"encoding/base64"
	"net/http"

	"github.com/go-chi/chi/v5"

	"github.com/arack/account-service/internal/config"
	"github.com/arack/account-service/internal/service"
	"github.com/arack/account-service/pkg/httputil"
)

// Handler handles HTTP requests
type Handler struct {
	sessionService   *service.SessionService
	authService      *service.AuthService
	zitadelService   *service.ZitadelService
	stalwartService  *service.StalwartService
	localAuthService *service.LocalAuthService
	cookieConfig     *config.CookieConfig
}

// New creates a new handler
func New(
	sessionSvc *service.SessionService,
	authSvc *service.AuthService,
	zitadelSvc *service.ZitadelService,
	stalwartSvc *service.StalwartService,
	localAuthSvc *service.LocalAuthService,
	cookieCfg *config.CookieConfig,
) *Handler {
	return &Handler{
		sessionService:   sessionSvc,
		authService:      authSvc,
		zitadelService:   zitadelSvc,
		stalwartService:  stalwartSvc,
		localAuthService: localAuthSvc,
		cookieConfig:     cookieCfg,
	}
}

// Routes returns the router with all routes
func (h *Handler) Routes() chi.Router {
	r := chi.NewRouter()

	// OAuth discovery endpoints (for Stalwart and other OAuth clients)
	r.Get("/.well-known/openid-configuration", h.OpenIDConfiguration)
	r.Get("/.well-known/jwks.json", h.JWKS)

	// OAuth endpoints
	r.Get("/userinfo", h.UserInfo)
	r.Post("/oauth/token", h.TokenRefresh)

	// Legacy OAuth flow (can be removed after Zitadel is fully replaced)
	r.Get("/login", h.Login)
	r.Get("/callback", h.Callback)

	// API routes
	r.Route("/api", func(r chi.Router) {
		// Session management
		r.Get("/session", h.GetSession)
		r.Post("/logout", h.Logout)

		// Custom login (email/password) - uses local auth
		r.Post("/login", h.CredentialsLogin)

		// Registration - uses local auth
		r.Post("/register", h.Register)
		r.Post("/register/check-email", h.CheckEmailAvailability)
		r.Get("/register/suggestions", h.GetEmailSuggestions)
	})

	// Health check
	r.Get("/health", h.Health)

	return r
}

// Health handles health check requests
func (h *Handler) Health(w http.ResponseWriter, r *http.Request) {
	httputil.JSON(w, http.StatusOK, map[string]string{
		"status":  "ok",
		"service": "account-service",
	})
}

// generateRandomString generates a cryptographically secure random string
func generateRandomString(length int) string {
	b := make([]byte, length)
	rand.Read(b)
	return base64.RawURLEncoding.EncodeToString(b)[:length]
}

// clearCookie clears a cookie with proper attributes to match how it was set
// Note: OAuth cookies (oauth_state, oauth_verifier, return_url) are set without Domain,
// so they are correctly cleared without Domain. Session cookie uses Domain and is
// cleared with Domain directly in logout.go.
func clearCookie(w http.ResponseWriter, name string) {
	http.SetCookie(w, &http.Cookie{
		Name:     name,
		Value:    "",
		Path:     "/",
		MaxAge:   -1,
		HttpOnly: true,
		Secure:   true,
		SameSite: http.SameSiteLaxMode,
	})
}

// getSameSite returns the SameSite value
func getSameSite(s string) http.SameSite {
	switch s {
	case "strict":
		return http.SameSiteStrictMode
	case "none":
		return http.SameSiteNoneMode
	default:
		return http.SameSiteLaxMode
	}
}
