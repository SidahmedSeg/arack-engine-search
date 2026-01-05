package postgres

import (
	"context"
	"database/sql"
	"errors"
	"time"

	"github.com/google/uuid"

	"github.com/arack/account-service/internal/domain"
)

var (
	ErrUserNotFound      = errors.New("user not found")
	ErrEmailAlreadyExists = errors.New("email already exists")
)

// UserRepository handles user persistence in PostgreSQL
type UserRepository struct {
	db *sql.DB
}

// NewUserRepository creates a new user repository
func NewUserRepository(db *sql.DB) *UserRepository {
	return &UserRepository{db: db}
}

// User represents a user in the database
type User struct {
	ID            uuid.UUID
	Email         string
	PasswordHash  string
	FirstName     string
	LastName      string
	DisplayName   sql.NullString
	Gender        sql.NullString
	BirthDate     sql.NullTime
	PictureURL    sql.NullString
	EmailVerified bool
	IsActive      bool
	CreatedAt     time.Time
	UpdatedAt     time.Time
}

// ToDomain converts database user to domain user
func (u *User) ToDomain() *domain.User {
	displayName := u.FirstName + " " + u.LastName
	if u.DisplayName.Valid {
		displayName = u.DisplayName.String
	}

	picture := ""
	if u.PictureURL.Valid {
		picture = u.PictureURL.String
	}

	return &domain.User{
		ID:      u.ID.String(),
		Email:   u.Email,
		Name:    displayName,
		Picture: picture,
	}
}

// Create creates a new user
func (r *UserRepository) Create(ctx context.Context, email, passwordHash, firstName, lastName string, gender, birthDate *string) (*User, error) {
	var genderVal, birthDateVal sql.NullString
	if gender != nil {
		genderVal = sql.NullString{String: *gender, Valid: true}
	}
	if birthDate != nil {
		birthDateVal = sql.NullString{String: *birthDate, Valid: true}
	}

	displayName := firstName + " " + lastName

	query := `
		INSERT INTO users (email, password_hash, first_name, last_name, display_name, gender, birth_date)
		VALUES ($1, $2, $3, $4, $5, $6, $7::date)
		RETURNING id, email, password_hash, first_name, last_name, display_name, gender, birth_date,
		          picture_url, email_verified, is_active, created_at, updated_at
	`

	var user User
	var birthDateResult sql.NullTime
	err := r.db.QueryRowContext(ctx, query,
		email, passwordHash, firstName, lastName, displayName,
		genderVal, birthDateVal,
	).Scan(
		&user.ID, &user.Email, &user.PasswordHash, &user.FirstName, &user.LastName,
		&user.DisplayName, &user.Gender, &birthDateResult,
		&user.PictureURL, &user.EmailVerified, &user.IsActive, &user.CreatedAt, &user.UpdatedAt,
	)

	if err != nil {
		if isUniqueViolation(err) {
			return nil, ErrEmailAlreadyExists
		}
		return nil, err
	}

	user.BirthDate = birthDateResult
	return &user, nil
}

// GetByEmail retrieves a user by email
func (r *UserRepository) GetByEmail(ctx context.Context, email string) (*User, error) {
	query := `
		SELECT id, email, password_hash, first_name, last_name, display_name, gender, birth_date,
		       picture_url, email_verified, is_active, created_at, updated_at
		FROM users
		WHERE email = $1 AND is_active = TRUE
	`

	var user User
	err := r.db.QueryRowContext(ctx, query, email).Scan(
		&user.ID, &user.Email, &user.PasswordHash, &user.FirstName, &user.LastName,
		&user.DisplayName, &user.Gender, &user.BirthDate,
		&user.PictureURL, &user.EmailVerified, &user.IsActive, &user.CreatedAt, &user.UpdatedAt,
	)

	if err != nil {
		if errors.Is(err, sql.ErrNoRows) {
			return nil, ErrUserNotFound
		}
		return nil, err
	}

	return &user, nil
}

// GetByID retrieves a user by ID
func (r *UserRepository) GetByID(ctx context.Context, id uuid.UUID) (*User, error) {
	query := `
		SELECT id, email, password_hash, first_name, last_name, display_name, gender, birth_date,
		       picture_url, email_verified, is_active, created_at, updated_at
		FROM users
		WHERE id = $1 AND is_active = TRUE
	`

	var user User
	err := r.db.QueryRowContext(ctx, query, id).Scan(
		&user.ID, &user.Email, &user.PasswordHash, &user.FirstName, &user.LastName,
		&user.DisplayName, &user.Gender, &user.BirthDate,
		&user.PictureURL, &user.EmailVerified, &user.IsActive, &user.CreatedAt, &user.UpdatedAt,
	)

	if err != nil {
		if errors.Is(err, sql.ErrNoRows) {
			return nil, ErrUserNotFound
		}
		return nil, err
	}

	return &user, nil
}

// EmailExists checks if an email is already registered
func (r *UserRepository) EmailExists(ctx context.Context, email string) (bool, error) {
	query := `SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)`
	var exists bool
	err := r.db.QueryRowContext(ctx, query, email).Scan(&exists)
	return exists, err
}

// UpdatePassword updates a user's password
func (r *UserRepository) UpdatePassword(ctx context.Context, userID uuid.UUID, passwordHash string) error {
	query := `UPDATE users SET password_hash = $1 WHERE id = $2`
	_, err := r.db.ExecContext(ctx, query, passwordHash, userID)
	return err
}

// isUniqueViolation checks if the error is a unique constraint violation
func isUniqueViolation(err error) bool {
	// PostgreSQL unique violation error code is 23505
	return err != nil && (contains(err.Error(), "23505") || contains(err.Error(), "unique"))
}

func contains(s, substr string) bool {
	return len(s) >= len(substr) && (s == substr || len(s) > 0 && containsRune(s, substr))
}

func containsRune(s, substr string) bool {
	for i := 0; i <= len(s)-len(substr); i++ {
		if s[i:i+len(substr)] == substr {
			return true
		}
	}
	return false
}
