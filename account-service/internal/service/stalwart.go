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
)

// StalwartService handles Stalwart mail server operations
type StalwartService struct {
	baseURL       string
	adminUser     string
	adminPassword string
	defaultQuota  int64
	httpClient    *http.Client
}

// NewStalwartService creates a new Stalwart service
func NewStalwartService(cfg *config.StalwartConfig) *StalwartService {
	return &StalwartService{
		baseURL:       strings.TrimSuffix(cfg.BaseURL, "/"),
		adminUser:     cfg.AdminUser,
		adminPassword: cfg.AdminPassword,
		defaultQuota:  cfg.DefaultQuota,
		httpClient: &http.Client{
			Timeout: 30 * time.Second,
		},
	}
}

// EmailExistsRequest represents a request to check email availability
type EmailExistsRequest struct {
	Email string `json:"email"`
}

// EmailExistsResponse represents the response from email check
type EmailExistsResponse struct {
	Available bool   `json:"available"`
	Email     string `json:"email"`
}

// CreateEmailAccountRequest represents a request to create an email account
type CreateEmailAccountRequest struct {
	Email       string `json:"email"`
	Password    string `json:"password"`
	DisplayName string `json:"displayName,omitempty"`
	Quota       int64  `json:"quota,omitempty"`
}

// CreateEmailAccountResponse represents the response from account creation
type CreateEmailAccountResponse struct {
	Email  string `json:"email"`
	UserID uint64 `json:"userId"`
}

// EmailExists checks if an email account already exists in Stalwart
func (s *StalwartService) EmailExists(ctx context.Context, email string) (bool, error) {
	// Extract username from email
	parts := strings.Split(email, "@")
	if len(parts) != 2 {
		return false, fmt.Errorf("invalid email format")
	}
	username := parts[0]

	url := fmt.Sprintf("%s/api/principal/%s", s.baseURL, username)

	req, err := http.NewRequestWithContext(ctx, http.MethodGet, url, nil)
	if err != nil {
		return false, fmt.Errorf("create request: %w", err)
	}

	req.SetBasicAuth(s.adminUser, s.adminPassword)

	resp, err := s.httpClient.Do(req)
	if err != nil {
		return false, fmt.Errorf("send request: %w", err)
	}
	defer resp.Body.Close()

	// 200 = exists, 404 = not found
	if resp.StatusCode == http.StatusOK {
		return true, nil
	}
	if resp.StatusCode == http.StatusNotFound {
		return false, nil
	}

	return false, fmt.Errorf("unexpected status: %d", resp.StatusCode)
}

// CreateEmailAccount creates a new email account in Stalwart
func (s *StalwartService) CreateEmailAccount(ctx context.Context, req *CreateEmailAccountRequest) (*CreateEmailAccountResponse, error) {
	// Extract username from email
	parts := strings.Split(req.Email, "@")
	if len(parts) != 2 {
		return nil, fmt.Errorf("invalid email format")
	}
	username := parts[0]
	domain := parts[1]

	// Use default quota if not specified
	quota := req.Quota
	if quota == 0 {
		quota = s.defaultQuota
	}

	// Build Stalwart principal creation request
	body := map[string]interface{}{
		"type":        "individual",
		"name":        username,
		"secrets":     []string{req.Password},
		"emails":      []string{req.Email},
		"quota":       quota,
		"description": req.DisplayName,
		"memberOf":    []string{},
	}

	// Add domain membership if needed
	if domain == "arack.io" {
		// Could add domain-specific groups here
	}

	jsonBody, err := json.Marshal(body)
	if err != nil {
		return nil, fmt.Errorf("marshal request: %w", err)
	}

	url := s.baseURL + "/api/principal"

	httpReq, err := http.NewRequestWithContext(ctx, http.MethodPost, url, bytes.NewReader(jsonBody))
	if err != nil {
		return nil, fmt.Errorf("create request: %w", err)
	}

	httpReq.Header.Set("Content-Type", "application/json")
	httpReq.SetBasicAuth(s.adminUser, s.adminPassword)

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
			Str("email", req.Email).
			Msg("Stalwart create account failed")
		return nil, fmt.Errorf("create account failed: %s", string(respBody))
	}

	// Parse response to get user ID
	var result struct {
		Data uint64 `json:"data"`
	}
	if err := json.Unmarshal(respBody, &result); err != nil {
		// Account might still be created even if we can't parse response
		log.Warn().
			Err(err).
			Str("email", req.Email).
			Msg("Created account but failed to parse response")
		return &CreateEmailAccountResponse{
			Email:  req.Email,
			UserID: 0,
		}, nil
	}

	log.Info().
		Str("email", req.Email).
		Uint64("user_id", result.Data).
		Msg("Email account created in Stalwart")

	return &CreateEmailAccountResponse{
		Email:  req.Email,
		UserID: result.Data,
	}, nil
}

// GenerateEmailSuggestions generates email suggestions based on name
func (s *StalwartService) GenerateEmailSuggestions(ctx context.Context, firstName, lastName string) ([]string, error) {
	// Normalize names (lowercase, remove special characters)
	first := strings.ToLower(strings.TrimSpace(firstName))
	last := strings.ToLower(strings.TrimSpace(lastName))

	// Remove non-alphanumeric characters
	first = strings.Map(func(r rune) rune {
		if (r >= 'a' && r <= 'z') || (r >= '0' && r <= '9') {
			return r
		}
		return -1
	}, first)

	last = strings.Map(func(r rune) rune {
		if (r >= 'a' && r <= 'z') || (r >= '0' && r <= '9') {
			return r
		}
		return -1
	}, last)

	if first == "" || last == "" {
		return nil, fmt.Errorf("invalid name")
	}

	// Generate suggestions
	suggestions := []string{
		fmt.Sprintf("%s.%s@arack.io", first, last),   // firstname.lastname
		fmt.Sprintf("%s%s@arack.io", first, last),     // firstnamelastname
		fmt.Sprintf("%s.%s@arack.io", last, first),    // lastname.firstname
		fmt.Sprintf("%s%c@arack.io", first, last[0]),  // firstnamel
		fmt.Sprintf("%c%s@arack.io", first[0], last),  // flastname
	}

	// Check availability and return first 2 available
	var available []string
	for _, email := range suggestions {
		exists, err := s.EmailExists(ctx, email)
		if err != nil {
			log.Warn().Err(err).Str("email", email).Msg("Failed to check email availability")
			continue
		}
		if !exists {
			available = append(available, email)
			if len(available) >= 2 {
				break
			}
		}
	}

	return available, nil
}
