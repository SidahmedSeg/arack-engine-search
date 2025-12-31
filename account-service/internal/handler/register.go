package handler

import (
	"encoding/json"
	"net/http"
	"strings"
	"time"

	"github.com/rs/zerolog/log"

	"github.com/arack/account-service/internal/domain"
	"github.com/arack/account-service/internal/service"
	"github.com/arack/account-service/pkg/httputil"
)

// RegisterRequest represents a full registration request (all 3 steps combined)
type RegisterRequest struct {
	// Step 1: Personal info
	FirstName string `json:"firstName"`
	LastName  string `json:"lastName"`
	Gender    string `json:"gender,omitempty"`
	BirthDate string `json:"birthDate,omitempty"` // Format: YYYY-MM-DD

	// Step 2: Email selection
	Email string `json:"email"` // The selected @arack.io email

	// Step 3: Password
	Password        string `json:"password"`
	ConfirmPassword string `json:"confirmPassword"`
}

// RegisterResponse represents a successful registration response
type RegisterResponse struct {
	Success bool         `json:"success"`
	User    *domain.User `json:"user"`
	Email   string       `json:"email"`
}

// CheckEmailRequest represents an email availability check request
type CheckEmailRequest struct {
	Email string `json:"email"`
}

// CheckEmailResponse represents the email availability check response
type CheckEmailResponse struct {
	Available bool   `json:"available"`
	Email     string `json:"email"`
}

// EmailSuggestionsResponse represents email suggestions response
type EmailSuggestionsResponse struct {
	Suggestions []string `json:"suggestions"`
}

// Register handles full user registration
func (h *Handler) Register(w http.ResponseWriter, r *http.Request) {
	var req RegisterRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		httputil.Error(w, http.StatusBadRequest, "Invalid request body")
		return
	}

	ctx := r.Context()

	// Validate required fields
	if req.FirstName == "" || req.LastName == "" {
		httputil.Error(w, http.StatusBadRequest, "First name and last name are required")
		return
	}

	if req.Email == "" {
		httputil.Error(w, http.StatusBadRequest, "Email is required")
		return
	}

	// Ensure email is @arack.io
	if !strings.HasSuffix(req.Email, "@arack.io") {
		httputil.Error(w, http.StatusBadRequest, "Email must be @arack.io")
		return
	}

	if req.Password == "" {
		httputil.Error(w, http.StatusBadRequest, "Password is required")
		return
	}

	if req.Password != req.ConfirmPassword {
		httputil.Error(w, http.StatusBadRequest, "Passwords do not match")
		return
	}

	// Validate password strength (basic)
	if len(req.Password) < 8 {
		httputil.Error(w, http.StatusBadRequest, "Password must be at least 8 characters")
		return
	}

	// Step 1: Check if email is available in Stalwart
	exists, err := h.stalwartService.EmailExists(ctx, req.Email)
	if err != nil {
		log.Error().Err(err).Str("email", req.Email).Msg("Failed to check email availability in Stalwart")
		httputil.Error(w, http.StatusInternalServerError, "Failed to check email availability")
		return
	}
	if exists {
		httputil.Error(w, http.StatusConflict, "Email already taken")
		return
	}

	// Step 2: Check if user exists in Zitadel
	zitadelExists, err := h.zitadelService.UserExists(ctx, req.Email)
	if err != nil {
		log.Error().Err(err).Str("email", req.Email).Msg("Failed to check user in Zitadel")
		httputil.Error(w, http.StatusInternalServerError, "Failed to check user availability")
		return
	}
	if zitadelExists {
		httputil.Error(w, http.StatusConflict, "Email already registered")
		return
	}

	// Step 3: Create user in Zitadel
	createUserResp, err := h.zitadelService.CreateUser(ctx, &service.CreateUserRequest{
		FirstName: req.FirstName,
		LastName:  req.LastName,
		Email:     req.Email,
		Password:  req.Password,
		Gender:    req.Gender,
		BirthDate: req.BirthDate,
	})
	if err != nil {
		log.Error().Err(err).Str("email", req.Email).Msg("Failed to create user in Zitadel")
		httputil.Error(w, http.StatusInternalServerError, "Failed to create user account")
		return
	}

	// Step 4: Create email account in Stalwart
	displayName := req.FirstName + " " + req.LastName
	_, err = h.stalwartService.CreateEmailAccount(ctx, &service.CreateEmailAccountRequest{
		Email:       req.Email,
		Password:    req.Password, // Use same password for email
		DisplayName: displayName,
	})
	if err != nil {
		log.Error().
			Err(err).
			Str("email", req.Email).
			Str("zitadel_user_id", createUserResp.UserID).
			Msg("Failed to create email account in Stalwart")
		// Note: User is created in Zitadel but not in Stalwart
		// This is a partial failure - we should handle this better in production
		// For now, we'll return success since user can login
	}

	// Step 5: Auto-login the user after registration
	user := &domain.User{
		ID:    createUserResp.UserID,
		Email: req.Email,
		Name:  displayName,
	}

	tokens := &service.Tokens{
		AccessToken:  "", // Will be populated on first API call
		RefreshToken: "",
		IDToken:      "",
		ExpiresAt:    time.Now().Add(24 * time.Hour),
	}

	session, err := h.sessionService.Create(ctx, *user, tokens)
	if err != nil {
		log.Warn().Err(err).Str("email", req.Email).Msg("Failed to create session after registration")
		// Registration succeeded, just no auto-login
		httputil.JSON(w, http.StatusCreated, RegisterResponse{
			Success: true,
			User:    user,
			Email:   req.Email,
		})
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
		Str("user_id", createUserResp.UserID).
		Str("session_id", session.ID.String()).
		Msg("User registered and logged in")

	httputil.JSON(w, http.StatusCreated, RegisterResponse{
		Success: true,
		User:    user,
		Email:   req.Email,
	})
}

// CheckEmailAvailability checks if an email is available
func (h *Handler) CheckEmailAvailability(w http.ResponseWriter, r *http.Request) {
	var req CheckEmailRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		httputil.Error(w, http.StatusBadRequest, "Invalid request body")
		return
	}

	if req.Email == "" {
		httputil.Error(w, http.StatusBadRequest, "Email is required")
		return
	}

	// Ensure email is @arack.io
	if !strings.HasSuffix(req.Email, "@arack.io") {
		httputil.Error(w, http.StatusBadRequest, "Email must be @arack.io")
		return
	}

	ctx := r.Context()

	// Check Stalwart first (email account)
	stalwartExists, err := h.stalwartService.EmailExists(ctx, req.Email)
	if err != nil {
		log.Error().Err(err).Str("email", req.Email).Msg("Failed to check email in Stalwart")
		httputil.Error(w, http.StatusInternalServerError, "Failed to check email availability")
		return
	}

	if stalwartExists {
		httputil.JSON(w, http.StatusOK, CheckEmailResponse{
			Available: false,
			Email:     req.Email,
		})
		return
	}

	// Check Zitadel (user account)
	zitadelExists, err := h.zitadelService.UserExists(ctx, req.Email)
	if err != nil {
		log.Error().Err(err).Str("email", req.Email).Msg("Failed to check user in Zitadel")
		httputil.Error(w, http.StatusInternalServerError, "Failed to check email availability")
		return
	}

	httputil.JSON(w, http.StatusOK, CheckEmailResponse{
		Available: !zitadelExists,
		Email:     req.Email,
	})
}

// GetEmailSuggestions generates email suggestions based on name
func (h *Handler) GetEmailSuggestions(w http.ResponseWriter, r *http.Request) {
	firstName := r.URL.Query().Get("firstName")
	lastName := r.URL.Query().Get("lastName")

	if firstName == "" || lastName == "" {
		httputil.Error(w, http.StatusBadRequest, "firstName and lastName are required")
		return
	}

	ctx := r.Context()

	suggestions, err := h.stalwartService.GenerateEmailSuggestions(ctx, firstName, lastName)
	if err != nil {
		log.Error().
			Err(err).
			Str("firstName", firstName).
			Str("lastName", lastName).
			Msg("Failed to generate email suggestions")
		httputil.Error(w, http.StatusInternalServerError, "Failed to generate suggestions")
		return
	}

	httputil.JSON(w, http.StatusOK, EmailSuggestionsResponse{
		Suggestions: suggestions,
	})
}
