package service

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"strings"
	"time"

	"github.com/rs/zerolog/log"

	"github.com/arack/account-service/internal/config"
	"github.com/arack/account-service/internal/domain"
)

// ZitadelService handles Zitadel Management and Session API operations
type ZitadelService struct {
	baseURL       string
	pat           string
	emailClientID string
	httpClient    *http.Client
}

// NewZitadelService creates a new Zitadel service
func NewZitadelService(cfg *config.ZitadelConfig) *ZitadelService {
	return &ZitadelService{
		baseURL:       strings.TrimSuffix(cfg.APIBaseURL, "/"),
		pat:           cfg.PAT,
		emailClientID: cfg.EmailClientID,
		httpClient: &http.Client{
			Timeout: 30 * time.Second,
		},
	}
}

// GetEmailClientID returns the email app OAuth client ID
func (s *ZitadelService) GetEmailClientID() string {
	return s.emailClientID
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
	// Use management/v1 search API (v2beta has internal errors)
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

	url := s.baseURL + "/management/v1/users/_search"

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

	// Management v1 response format
	var result struct {
		Result []struct {
			ID    string `json:"id"`
			Human struct {
				Profile struct {
					FirstName   string `json:"firstName"`
					LastName    string `json:"lastName"`
					DisplayName string `json:"displayName"`
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
		ID:    user.ID,
		Email: user.Human.Email.Email,
		Name:  user.Human.Profile.FirstName + " " + user.Human.Profile.LastName,
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

// FinalizeAuthRequestResponse contains the callback URL with authorization code
type FinalizeAuthRequestResponse struct {
	CallbackURL string `json:"callbackUrl"`
}

// FinalizeAuthRequest completes an OAuth auth request with a valid session
// This links the authenticated session to the OAuth flow and returns the callback URL with auth code
func (s *ZitadelService) FinalizeAuthRequest(ctx context.Context, authRequestID, sessionID, sessionToken string) (*FinalizeAuthRequestResponse, error) {
	url := s.baseURL + "/v2/oidc/auth_requests/" + authRequestID

	body := map[string]interface{}{
		"session": map[string]interface{}{
			"sessionId":    sessionID,
			"sessionToken": sessionToken,
		},
	}

	jsonBody, err := json.Marshal(body)
	if err != nil {
		return nil, fmt.Errorf("marshal request: %w", err)
	}

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
			Str("authRequestID", authRequestID).
			Msg("Zitadel finalize auth request failed")
		return nil, fmt.Errorf("finalize auth request failed: %s", string(respBody))
	}

	var result struct {
		CallbackURL string `json:"callbackUrl"`
	}
	if err := json.Unmarshal(respBody, &result); err != nil {
		return nil, fmt.Errorf("parse response: %w", err)
	}

	log.Info().
		Str("authRequestID", authRequestID).
		Str("callbackUrl", result.CallbackURL).
		Msg("Auth request finalized successfully")

	return &FinalizeAuthRequestResponse{
		CallbackURL: result.CallbackURL,
	}, nil
}

// OAuthTokens represents OAuth tokens from Zitadel
type OAuthTokens struct {
	AccessToken  string `json:"access_token"`
	TokenType    string `json:"token_type"`
	ExpiresIn    int    `json:"expires_in"`
	RefreshToken string `json:"refresh_token,omitempty"`
	IDToken      string `json:"id_token,omitempty"`
}

// CreateAuthRequestResponse contains the auth request ID and authorization URL
type CreateAuthRequestResponse struct {
	AuthRequestID string `json:"authRequestId"`
	AuthURL       string `json:"authUrl"`
}

// CreateAuthRequest creates an OAuth authorization request server-side
// This initiates the Authorization Code flow programmatically
func (s *ZitadelService) CreateAuthRequest(ctx context.Context, clientID, redirectURI string, scopes []string) (*CreateAuthRequestResponse, error) {
	authURL := s.baseURL + "/v2/oidc/auth_requests"

	body := map[string]interface{}{
		"clientId":    clientID,
		"redirectUri": redirectURI,
		"scope":       scopes,
		"responseType": "code",
	}

	jsonBody, err := json.Marshal(body)
	if err != nil {
		return nil, fmt.Errorf("marshal request: %w", err)
	}

	httpReq, err := http.NewRequestWithContext(ctx, http.MethodPost, authURL, bytes.NewReader(jsonBody))
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
			Msg("Zitadel create auth request failed")
		return nil, fmt.Errorf("create auth request failed: %s", string(respBody))
	}

	var result struct {
		AuthRequestID string `json:"authRequestId"`
		AuthURL       string `json:"authUrl"`
	}
	if err := json.Unmarshal(respBody, &result); err != nil {
		return nil, fmt.Errorf("parse response: %w", err)
	}

	log.Info().
		Str("authRequestID", result.AuthRequestID).
		Msg("Auth request created successfully")

	return &CreateAuthRequestResponse{
		AuthRequestID: result.AuthRequestID,
		AuthURL:       result.AuthURL,
	}, nil
}

// GetTokensViaAuthFlow creates an auth request, finalizes it with the session, and exchanges the code for tokens
// This is a secure way to get OAuth tokens server-side after password authentication
func (s *ZitadelService) GetTokensViaAuthFlow(ctx context.Context, sessionID, sessionToken, clientID, redirectURI string, scopes []string) (*OAuthTokens, error) {
	// Step 1: Create an auth request
	authReq, err := s.CreateAuthRequest(ctx, clientID, redirectURI, scopes)
	if err != nil {
		return nil, fmt.Errorf("create auth request: %w", err)
	}

	// Step 2: Finalize the auth request with the session (user already authenticated)
	finalizeResp, err := s.FinalizeAuthRequest(ctx, authReq.AuthRequestID, sessionID, sessionToken)
	if err != nil {
		return nil, fmt.Errorf("finalize auth request: %w", err)
	}

	// Step 3: Extract the authorization code from the callback URL
	callbackURL, err := url.Parse(finalizeResp.CallbackURL)
	if err != nil {
		return nil, fmt.Errorf("parse callback URL: %w", err)
	}

	code := callbackURL.Query().Get("code")
	if code == "" {
		return nil, fmt.Errorf("no authorization code in callback URL")
	}

	// Step 4: Exchange the code for tokens
	tokens, err := s.ExchangeCodeForTokens(ctx, code, redirectURI, clientID)
	if err != nil {
		return nil, fmt.Errorf("exchange code for tokens: %w", err)
	}

	log.Info().
		Str("tokenType", tokens.TokenType).
		Int("expiresIn", tokens.ExpiresIn).
		Bool("hasRefreshToken", tokens.RefreshToken != "").
		Msg("OAuth tokens obtained via auth flow")

	return tokens, nil
}

// ExchangeCodeForTokens exchanges an authorization code for OAuth tokens
func (s *ZitadelService) ExchangeCodeForTokens(ctx context.Context, code, redirectURI, clientID string) (*OAuthTokens, error) {
	url := s.baseURL + "/oauth/v2/token"

	// Build form data
	formData := fmt.Sprintf(
		"grant_type=authorization_code&code=%s&redirect_uri=%s&client_id=%s",
		code, redirectURI, clientID,
	)

	httpReq, err := http.NewRequestWithContext(ctx, http.MethodPost, url, strings.NewReader(formData))
	if err != nil {
		return nil, fmt.Errorf("create request: %w", err)
	}

	httpReq.Header.Set("Content-Type", "application/x-www-form-urlencoded")

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
			Msg("Zitadel token exchange failed")
		return nil, fmt.Errorf("token exchange failed: %s", string(respBody))
	}

	var tokens OAuthTokens
	if err := json.Unmarshal(respBody, &tokens); err != nil {
		return nil, fmt.Errorf("parse response: %w", err)
	}

	log.Info().
		Str("tokenType", tokens.TokenType).
		Int("expiresIn", tokens.ExpiresIn).
		Bool("hasRefreshToken", tokens.RefreshToken != "").
		Bool("hasIDToken", tokens.IDToken != "").
		Msg("OAuth tokens obtained successfully")

	return &tokens, nil
}
