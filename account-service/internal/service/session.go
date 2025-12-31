package service

import (
	"context"
	"errors"
	"time"

	"github.com/rs/zerolog/log"

	"github.com/arack/account-service/internal/config"
	"github.com/arack/account-service/internal/domain"
	"github.com/arack/account-service/internal/repository"
)

var (
	// ErrSessionNotFound is returned when a session is not found
	ErrSessionNotFound = errors.New("session not found")
	// ErrSessionExpired is returned when a session is expired
	ErrSessionExpired = errors.New("session expired")
)

// SessionService handles session management
type SessionService struct {
	repo             repository.SessionRepository
	auth             *AuthService
	sessionTTL       time.Duration
	refreshThreshold time.Duration
}

// NewSessionService creates a new session service
func NewSessionService(
	repo repository.SessionRepository,
	auth *AuthService,
	cfg *config.SessionConfig,
) *SessionService {
	return &SessionService{
		repo:             repo,
		auth:             auth,
		sessionTTL:       cfg.TTL,
		refreshThreshold: cfg.RefreshThreshold,
	}
}

// Create creates a new session
func (s *SessionService) Create(ctx context.Context, user domain.User, tokens *Tokens) (*domain.Session, error) {
	session := domain.NewSession(user, tokens.AccessToken, tokens.RefreshToken, tokens.IDToken, tokens.ExpiresAt)

	if err := s.repo.Create(ctx, session, s.sessionTTL); err != nil {
		return nil, err
	}

	log.Info().
		Str("session_id", session.ID.String()).
		Str("user_id", user.ID).
		Str("email", user.Email).
		Msg("Session created")

	return session, nil
}

// Get retrieves and validates a session, refreshing tokens if needed
func (s *SessionService) Get(ctx context.Context, id domain.SessionID) (*domain.Session, error) {
	session, err := s.repo.Get(ctx, id)
	if err != nil {
		return nil, err
	}
	if session == nil {
		return nil, ErrSessionNotFound
	}

	// Check if tokens need refresh
	if session.NeedsRefresh(s.refreshThreshold) && session.RefreshToken != "" {
		newTokens, err := s.auth.RefreshToken(ctx, session.RefreshToken)
		if err != nil {
			log.Warn().
				Err(err).
				Str("session_id", id.String()).
				Msg("Token refresh failed")

			// Continue with existing token if not expired
			if session.IsExpired() {
				return nil, ErrSessionExpired
			}
		} else {
			session.UpdateTokens(newTokens.AccessToken, newTokens.RefreshToken, newTokens.ExpiresAt)

			if err := s.repo.Update(ctx, session, s.sessionTTL); err != nil {
				log.Warn().Err(err).Msg("Failed to update session after refresh")
			} else {
				log.Info().Str("session_id", id.String()).Msg("Tokens refreshed")
			}
		}
	}

	// Update last accessed
	session.Touch()
	_ = s.repo.Touch(ctx, id, s.sessionTTL)

	return session, nil
}

// Destroy destroys a session
func (s *SessionService) Destroy(ctx context.Context, id domain.SessionID) error {
	session, err := s.repo.Get(ctx, id)
	if err != nil {
		log.Warn().Err(err).Str("session_id", id.String()).Msg("Failed to get session for destruction")
	}

	if session != nil {
		// Revoke tokens at Zitadel (best effort)
		if err := s.auth.RevokeToken(ctx, session.AccessToken); err != nil {
			log.Warn().Err(err).Msg("Failed to revoke access token")
		}
	}

	if err := s.repo.Delete(ctx, id); err != nil {
		return err
	}

	log.Info().Str("session_id", id.String()).Msg("Session destroyed")
	return nil
}
