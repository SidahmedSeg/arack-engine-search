package domain

import (
	"time"

	"github.com/google/uuid"
)

// SessionID is a unique identifier for a session
type SessionID string

// NewSessionID generates a new random session ID
func NewSessionID() SessionID {
	return SessionID(uuid.New().String())
}

// String returns the string representation of the session ID
func (s SessionID) String() string {
	return string(s)
}

// Session represents a user session with OAuth tokens
type Session struct {
	ID             SessionID `json:"id"`
	User           User      `json:"user"`
	AccessToken    string    `json:"access_token"`
	RefreshToken   string    `json:"refresh_token,omitempty"`
	IDToken        string    `json:"id_token,omitempty"`
	TokenExpiresAt time.Time `json:"token_expires_at"`
	CreatedAt      time.Time `json:"created_at"`
	LastAccessedAt time.Time `json:"last_accessed_at"`
}

// NewSession creates a new session with the given user and tokens
func NewSession(user User, accessToken, refreshToken, idToken string, expiresAt time.Time) *Session {
	now := time.Now()
	return &Session{
		ID:             NewSessionID(),
		User:           user,
		AccessToken:    accessToken,
		RefreshToken:   refreshToken,
		IDToken:        idToken,
		TokenExpiresAt: expiresAt,
		CreatedAt:      now,
		LastAccessedAt: now,
	}
}

// NeedsRefresh returns true if the access token needs to be refreshed
func (s *Session) NeedsRefresh(threshold time.Duration) bool {
	return time.Now().Add(threshold).After(s.TokenExpiresAt)
}

// IsExpired returns true if the access token is expired
func (s *Session) IsExpired() bool {
	return time.Now().After(s.TokenExpiresAt)
}

// Touch updates the last accessed time
func (s *Session) Touch() {
	s.LastAccessedAt = time.Now()
}

// UpdateTokens updates the session tokens
func (s *Session) UpdateTokens(accessToken, refreshToken string, expiresAt time.Time) {
	s.AccessToken = accessToken
	if refreshToken != "" {
		s.RefreshToken = refreshToken
	}
	s.TokenExpiresAt = expiresAt
}
