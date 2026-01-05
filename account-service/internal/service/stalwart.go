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
	baseURL := strings.TrimSuffix(cfg.BaseURL, "/")
	log.Debug().
		Str("baseURL", baseURL).
		Str("adminUser", cfg.AdminUser).
		Int64("defaultQuota", cfg.DefaultQuota).
		Msg("Creating Stalwart service")

	return &StalwartService{
		baseURL:       baseURL,
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

// StalwartErrorResponse represents an error response from Stalwart API
type StalwartErrorResponse struct {
	Error string `json:"error"`
	Item  string `json:"item,omitempty"`
}

// EmailExists checks if an email account already exists in Stalwart
func (s *StalwartService) EmailExists(ctx context.Context, email string) (bool, error) {
	log.Debug().
		Str("email", email).
		Str("baseURL", s.baseURL).
		Str("adminUser", s.adminUser).
		Msg("EmailExists called")

	// Extract username from email
	parts := strings.Split(email, "@")
	if len(parts) != 2 {
		log.Error().Str("email", email).Msg("Invalid email format")
		return false, fmt.Errorf("invalid email format")
	}
	username := parts[0]

	url := fmt.Sprintf("%s/api/principal/%s", s.baseURL, username)
	log.Debug().Str("url", url).Msg("Making request to Stalwart")

	req, err := http.NewRequestWithContext(ctx, http.MethodGet, url, nil)
	if err != nil {
		log.Error().Err(err).Str("url", url).Msg("Failed to create request")
		return false, fmt.Errorf("create request: %w", err)
	}

	req.SetBasicAuth(s.adminUser, s.adminPassword)

	log.Debug().Str("url", url).Msg("Sending HTTP request to Stalwart")
	resp, err := s.httpClient.Do(req)
	if err != nil {
		log.Error().Err(err).Str("url", url).Msg("HTTP request failed")
		return false, fmt.Errorf("send request: %w", err)
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		log.Error().Err(err).Str("url", url).Msg("Failed to read response body")
		return false, fmt.Errorf("read response: %w", err)
	}

	log.Debug().
		Str("url", url).
		Int("status", resp.StatusCode).
		Str("body", string(body)).
		Msg("Stalwart response received")

	// Check for HTTP errors first
	if resp.StatusCode == http.StatusNotFound {
		log.Debug().Str("email", email).Msg("Email does not exist (404)")
		return false, nil
	}

	if resp.StatusCode != http.StatusOK {
		log.Error().
			Int("status", resp.StatusCode).
			Str("body", string(body)).
			Str("email", email).
			Msg("Unexpected status from Stalwart")
		return false, fmt.Errorf("unexpected status: %d", resp.StatusCode)
	}

	// Stalwart returns 200 even for "notFound" - check the response body
	// Response format: {"error":"notFound","item":"username"} when not found
	// Response format: {"data":{...principal data...}} when found
	var errorResp StalwartErrorResponse
	if err := json.Unmarshal(body, &errorResp); err == nil && errorResp.Error == "notFound" {
		log.Debug().Str("email", email).Msg("Email does not exist (notFound in body)")
		return false, nil
	}

	// If no error in response, the principal exists
	log.Debug().Str("email", email).Msg("Email exists")
	return true, nil
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
	log.Info().
		Str("firstName", firstName).
		Str("lastName", lastName).
		Msg("GenerateEmailSuggestions called")

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

	log.Debug().
		Str("normalizedFirst", first).
		Str("normalizedLast", last).
		Msg("Names normalized")

	if first == "" || last == "" {
		log.Error().
			Str("firstName", firstName).
			Str("lastName", lastName).
			Msg("Invalid name after normalization")
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

	log.Debug().
		Strs("suggestions", suggestions).
		Msg("Generated email suggestions")

	// Check availability and return first 2 available
	var available []string
	for i, email := range suggestions {
		log.Debug().
			Int("index", i).
			Str("email", email).
			Msg("Checking email availability")

		exists, err := s.EmailExists(ctx, email)
		if err != nil {
			log.Warn().Err(err).Str("email", email).Msg("Failed to check email availability")
			continue
		}

		log.Debug().
			Str("email", email).
			Bool("exists", exists).
			Msg("Email check result")

		if !exists {
			available = append(available, email)
			log.Debug().
				Str("email", email).
				Int("availableCount", len(available)).
				Msg("Added to available list")
			if len(available) >= 2 {
				break
			}
		}
	}

	log.Info().
		Strs("available", available).
		Int("count", len(available)).
		Msg("GenerateEmailSuggestions completed")

	return available, nil
}
