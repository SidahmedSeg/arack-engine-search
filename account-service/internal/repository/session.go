package repository

import (
	"context"
	"time"

	"github.com/arack/account-service/internal/domain"
)

// SessionRepository defines the interface for session storage
type SessionRepository interface {
	// Create stores a new session
	Create(ctx context.Context, session *domain.Session, ttl time.Duration) error

	// Get retrieves a session by ID
	Get(ctx context.Context, id domain.SessionID) (*domain.Session, error)

	// Update updates an existing session
	Update(ctx context.Context, session *domain.Session, ttl time.Duration) error

	// Delete removes a session
	Delete(ctx context.Context, id domain.SessionID) error

	// Touch extends the session TTL
	Touch(ctx context.Context, id domain.SessionID, ttl time.Duration) error
}
