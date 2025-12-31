package service

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strings"
	"time"

	"github.com/rs/zerolog/log"

	"github.com/arack/account-service/internal/config"
	"github.com/arack/account-service/internal/domain"
)

// ZitadelService handles Zitadel Management and Session API operations
type ZitadelService struct {
	baseURL    string
	pat        string
	httpClient *http.Client
}

// NewZitadelService creates a new Zitadel service
func NewZitadelService(cfg *config.ZitadelConfig) *ZitadelService {
	return &ZitadelService{
		baseURL: strings.TrimSuffix(cfg.APIBaseURL, "/"),
		pat:     cfg.PAT,
		httpClient: &http.Client{
			Timeout: 30 * time.Second,
		},
	}
}

// CreateUserRequest represents the request to create a new user
type CreateUserRequest struct {
	FirstName string `json:"firstName"`
	LastName  string `json:"lastName"`
	Email     string `json:"email"`
	Password  string `json:"password"`
	Gender    string `json:"gender,omitempty"`
	BirthDate string `json:"birthDate,omitempty"` // Format: YYYY-MM-DD
}

// CreateUserResponse represents the response from user creation
type CreateUserResponse struct {
	UserID string `json:"userId"`
}

// LoginRequest represents a login request
type LoginRequest struct {
	Email    string `json:"email"`
	Password string `json:"password"`
}

// LoginResponse represents a successful login response
type LoginResponse struct {
	SessionID    string       `json:"sessionId"`
	SessionToken string       `json:"sessionToken"`
	User         *domain.User `json:"user"`
}

// CreateUser creates a new user in Zitadel via Management API
func (s *ZitadelService) CreateUser(ctx context.Context, req *CreateUserRequest) (*CreateUserResponse, error) {
	// Build the Zitadel API request body
	// Using v2beta/users/human endpoint
	body := map[string]interface{}{
		"profile": map[string]interface{}{
			"givenName":  req.FirstName,
			"familyName": req.LastName,
		},
		"email": map[string]interface{}{
			"email":           req.Email,
			"isEmailVerified": true, // Skip email verification for @arack.io emails
		},
		"password": map[string]interface{}{
			"password":       req.Password,
			"changeRequired": false,
		},
	}

	// Add optional gender
	if req.Gender != "" {
		genderMap := map[string]int{
			"male":   1,
			"female": 2,
			"other":  3,
		}
		if genderCode, ok := genderMap[strings.ToLower(req.Gender)]; ok {
			body["profile"].(map[string]interface{})["gender"] = genderCode
		}
	}

	jsonBody, err := json.Marshal(body)
	if err != nil {
		return nil, fmt.Errorf("marshal request: %w", err)
	}

	url := s.baseURL + "/v2beta/users/human"

	httpReq, err := http.NewRequestWithContext(ctx, http.MethodPost, url, bytes.NewReader(jsonBody))
	if err != nil {
		return nil, fmt.Errorf("create request: %w", err)
	}

	httpReq.Header.Set("Content-Type", "application/json")
	httpReq.Header.Set("Authorization", "Bearer "+s.pat)

	resp, err := s.httpClient.Do(httpReq)
	if err != nil {
		return nil, fmt.Errorf("send request: %w", err)
	}
	defer resp.Body.Close()

	respBody, _ := io.ReadAll(resp.Body)

	if resp.StatusCode >= 400 {
		log.Error().
			Int("status", resp.StatusCode).
			Str("body", string(respBody)).
			Msg("Zitadel create user failed")
		return nil, fmt.Errorf("create user failed: %s", string(respBody))
	}

	var result struct {
		UserID string `json:"userId"`
	}
	if err := json.Unmarshal(respBody, &result); err != nil {
		return nil, fmt.Errorf("parse response: %w", err)
	}

	log.Info().
		Str("user_id", result.UserID).
		Str("email", req.Email).
		Msg("User created in Zitadel")

	return &CreateUserResponse{UserID: result.UserID}, nil
}

// Login authenticates a user with email/password via Zitadel Session API
func (s *ZitadelService) Login(ctx context.Context, req *LoginRequest) (*LoginResponse, error) {
	// Step 1: Create a session with password check
	sessionBody := map[string]interface{}{
		"checks": map[string]interface{}{
			"user": map[string]interface{}{
				"loginName": req.Email,
			},
			"password": map[string]interface{}{
				"password": req.Password,
			},
		},
	}

	jsonBody, err := json.Marshal(sessionBody)
	if err != nil {
		return nil, fmt.Errorf("marshal request: %w", err)
	}

	url := s.baseURL + "/v2beta/sessions"

	httpReq, err := http.NewRequestWithContext(ctx, http.MethodPost, url, bytes.NewReader(jsonBody))
	if err != nil {
		return nil, fmt.Errorf("create request: %w", err)
	}

	httpReq.Header.Set("Content-Type", "application/json")
	httpReq.Header.Set("Authorization", "Bearer "+s.pat)

	resp, err := s.httpClient.Do(httpReq)
	if err != nil {
		return nil, fmt.Errorf("send request: %w", err)
	}
	defer resp.Body.Close()

	respBody, _ := io.ReadAll(resp.Body)

	if resp.StatusCode >= 400 {
		log.Warn().
			Int("status", resp.StatusCode).
			Str("email", req.Email).
			Msg("Zitadel login failed")
		return nil, fmt.Errorf("invalid credentials")
	}

	var sessionResp struct {
		SessionID    string `json:"sessionId"`
		SessionToken string `json:"sessionToken"`
	}
	if err := json.Unmarshal(respBody, &sessionResp); err != nil {
		return nil, fmt.Errorf("parse session response: %w", err)
	}

	// Step 2: Get user details
	user, err := s.getUserBySession(ctx, sessionResp.SessionID)
	if err != nil {
		log.Warn().Err(err).Msg("Failed to get user details after login")
		// Return basic session info without full user details
		return &LoginResponse{
			SessionID:    sessionResp.SessionID,
			SessionToken: sessionResp.SessionToken,
			User: &domain.User{
				Email: req.Email,
			},
		}, nil
	}

	return &LoginResponse{
		SessionID:    sessionResp.SessionID,
		SessionToken: sessionResp.SessionToken,
		User:         user,
	}, nil
}

// getUserBySession retrieves user details from a session
func (s *ZitadelService) getUserBySession(ctx context.Context, sessionID string) (*domain.User, error) {
	url := s.baseURL + "/v2beta/sessions/" + sessionID

	httpReq, err := http.NewRequestWithContext(ctx, http.MethodGet, url, nil)
	if err != nil {
		return nil, fmt.Errorf("create request: %w", err)
	}

	httpReq.Header.Set("Authorization", "Bearer "+s.pat)

	resp, err := s.httpClient.Do(httpReq)
	if err != nil {
		return nil, fmt.Errorf("send request: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode >= 400 {
		return nil, fmt.Errorf("get session failed: %d", resp.StatusCode)
	}

	respBody, _ := io.ReadAll(resp.Body)

	var result struct {
		Session struct {
			UserID string `json:"userId"`
			Factors struct {
				User struct {
					ID          string `json:"id"`
					LoginName   string `json:"loginName"`
					DisplayName string `json:"displayName"`
				} `json:"user"`
			} `json:"factors"`
		} `json:"session"`
	}
	if err := json.Unmarshal(respBody, &result); err != nil {
		return nil, fmt.Errorf("parse response: %w", err)
	}

	return &domain.User{
		ID:    result.Session.Factors.User.ID,
		Email: result.Session.Factors.User.LoginName,
		Name:  result.Session.Factors.User.DisplayName,
	}, nil
}

// GetUserByEmail retrieves user details by email
func (s *ZitadelService) GetUserByEmail(ctx context.Context, email string) (*domain.User, error) {
	// Use search API to find user by login name
	body := map[string]interface{}{
		"queries": []map[string]interface{}{
			{
				"emailQuery": map[string]interface{}{
					"emailAddress": email,
					"method":       "TEXT_QUERY_METHOD_EQUALS",
				},
			},
		},
	}

	jsonBody, err := json.Marshal(body)
	if err != nil {
		return nil, fmt.Errorf("marshal request: %w", err)
	}

	url := s.baseURL + "/v2beta/users"

	httpReq, err := http.NewRequestWithContext(ctx, http.MethodPost, url, bytes.NewReader(jsonBody))
	if err != nil {
		return nil, fmt.Errorf("create request: %w", err)
	}

	httpReq.Header.Set("Content-Type", "application/json")
	httpReq.Header.Set("Authorization", "Bearer "+s.pat)

	resp, err := s.httpClient.Do(httpReq)
	if err != nil {
		return nil, fmt.Errorf("send request: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode >= 400 {
		return nil, fmt.Errorf("search user failed: %d", resp.StatusCode)
	}

	respBody, _ := io.ReadAll(resp.Body)

	var result struct {
		Result []struct {
			UserID      string `json:"userId"`
			Human       struct {
				Profile struct {
					GivenName  string `json:"givenName"`
					FamilyName string `json:"familyName"`
				} `json:"profile"`
				Email struct {
					Email string `json:"email"`
				} `json:"email"`
			} `json:"human"`
		} `json:"result"`
	}
	if err := json.Unmarshal(respBody, &result); err != nil {
		return nil, fmt.Errorf("parse response: %w", err)
	}

	if len(result.Result) == 0 {
		return nil, nil // User not found
	}

	user := result.Result[0]
	return &domain.User{
		ID:    user.UserID,
		Email: user.Human.Email.Email,
		Name:  user.Human.Profile.GivenName + " " + user.Human.Profile.FamilyName,
	}, nil
}

// UserExists checks if a user with the given email exists in Zitadel
func (s *ZitadelService) UserExists(ctx context.Context, email string) (bool, error) {
	user, err := s.GetUserByEmail(ctx, email)
	if err != nil {
		return false, err
	}
	return user != nil, nil
}
