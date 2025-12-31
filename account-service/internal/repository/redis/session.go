package redis

import (
	"context"
	"encoding/json"
	"fmt"
	"time"

	"github.com/redis/go-redis/v9"

	"github.com/arack/account-service/internal/domain"
	"github.com/arack/account-service/internal/repository"
)

type sessionRepository struct {
	client    *redis.Client
	keyPrefix string
}

// NewSessionRepository creates a new Redis-backed session repository
func NewSessionRepository(client *redis.Client) repository.SessionRepository {
	return &sessionRepository{
		client:    client,
		keyPrefix: "account:session:",
	}
}

func (r *sessionRepository) key(id domain.SessionID) string {
	return fmt.Sprintf("%s%s", r.keyPrefix, id.String())
}

func (r *sessionRepository) Create(ctx context.Context, session *domain.Session, ttl time.Duration) error {
	data, err := json.Marshal(session)
	if err != nil {
		return fmt.Errorf("marshal session: %w", err)
	}

	if err := r.client.Set(ctx, r.key(session.ID), data, ttl).Err(); err != nil {
		return fmt.Errorf("redis set: %w", err)
	}

	return nil
}

func (r *sessionRepository) Get(ctx context.Context, id domain.SessionID) (*domain.Session, error) {
	data, err := r.client.Get(ctx, r.key(id)).Bytes()
	if err == redis.Nil {
		return nil, nil // Not found
	}
	if err != nil {
		return nil, fmt.Errorf("redis get: %w", err)
	}

	var session domain.Session
	if err := json.Unmarshal(data, &session); err != nil {
		return nil, fmt.Errorf("unmarshal session: %w", err)
	}

	return &session, nil
}

func (r *sessionRepository) Update(ctx context.Context, session *domain.Session, ttl time.Duration) error {
	return r.Create(ctx, session, ttl) // Redis SET is idempotent
}

func (r *sessionRepository) Delete(ctx context.Context, id domain.SessionID) error {
	if err := r.client.Del(ctx, r.key(id)).Err(); err != nil {
		return fmt.Errorf("redis del: %w", err)
	}
	return nil
}

func (r *sessionRepository) Touch(ctx context.Context, id domain.SessionID, ttl time.Duration) error {
	if err := r.client.Expire(ctx, r.key(id), ttl).Err(); err != nil {
		return fmt.Errorf("redis expire: %w", err)
	}
	return nil
}
