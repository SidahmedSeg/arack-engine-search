package service

import (
	"context"
	"errors"

	"github.com/google/uuid"
	"github.com/rs/zerolog/log"

	"github.com/arack/account-service/internal/domain"
	"github.com/arack/account-service/internal/repository/postgres"
)

var (
	ErrInvalidCredentials = errors.New("invalid email or password")
	ErrUserExists         = errors.New("user already exists")
)

// LocalAuthService handles local authentication (replaces Zitadel)
type LocalAuthService struct {
	userRepo        *postgres.UserRepository
	passwordService *PasswordService
	jwtService      *JWTService
}

// NewLocalAuthService creates a new local auth service
func NewLocalAuthService(
	userRepo *postgres.UserRepository,
	passwordService *PasswordService,
	jwtService *JWTService,
) *LocalAuthService {
	return &LocalAuthService{
		userRepo:        userRepo,
		passwordService: passwordService,
		jwtService:      jwtService,
	}
}

// RegisterRequest represents a registration request
type RegisterUserRequest struct {
	Email     string
	Password  string
	FirstName string
	LastName  string
	Gender    *string
	BirthDate *string
}

// RegisterResponse represents a registration response
type RegisterUserResponse struct {
	User   *domain.User
	Tokens *TokenPair
}

// Register creates a new user account
func (s *LocalAuthService) Register(ctx context.Context, req *RegisterUserRequest) (*RegisterUserResponse, error) {
	// Check if email already exists
	exists, err := s.userRepo.EmailExists(ctx, req.Email)
	if err != nil {
		return nil, err
	}
	if exists {
		return nil, ErrUserExists
	}

	// Hash password
	passwordHash, err := s.passwordService.Hash(req.Password)
	if err != nil {
		return nil, err
	}

	// Create user
	dbUser, err := s.userRepo.Create(ctx, req.Email, passwordHash, req.FirstName, req.LastName, req.Gender, req.BirthDate)
	if err != nil {
		if errors.Is(err, postgres.ErrEmailAlreadyExists) {
			return nil, ErrUserExists
		}
		return nil, err
	}

	user := dbUser.ToDomain()

	// Generate tokens
	tokens, err := s.jwtService.GenerateTokens(user)
	if err != nil {
		return nil, err
	}

	log.Info().
		Str("email", user.Email).
		Str("user_id", user.ID).
		Msg("User registered successfully")

	return &RegisterUserResponse{
		User:   user,
		Tokens: tokens,
	}, nil
}

// LoginRequest represents a login request
type LoginUserRequest struct {
	Email    string
	Password string
}

// LoginResponse represents a login response
type LoginUserResponse struct {
	User   *domain.User
	Tokens *TokenPair
}

// Login authenticates a user and returns tokens
func (s *LocalAuthService) Login(ctx context.Context, req *LoginUserRequest) (*LoginUserResponse, error) {
	// Get user by email
	dbUser, err := s.userRepo.GetByEmail(ctx, req.Email)
	if err != nil {
		if errors.Is(err, postgres.ErrUserNotFound) {
			return nil, ErrInvalidCredentials
		}
		return nil, err
	}

	// Verify password
	valid, err := s.passwordService.Verify(req.Password, dbUser.PasswordHash)
	if err != nil {
		return nil, err
	}
	if !valid {
		return nil, ErrInvalidCredentials
	}

	user := dbUser.ToDomain()

	// Generate tokens
	tokens, err := s.jwtService.GenerateTokens(user)
	if err != nil {
		return nil, err
	}

	log.Info().
		Str("email", user.Email).
		Str("user_id", user.ID).
		Msg("User logged in successfully")

	return &LoginUserResponse{
		User:   user,
		Tokens: tokens,
	}, nil
}

// GetUserByID retrieves a user by ID
func (s *LocalAuthService) GetUserByID(ctx context.Context, id string) (*domain.User, error) {
	userID, err := uuid.Parse(id)
	if err != nil {
		return nil, errors.New("invalid user ID")
	}

	dbUser, err := s.userRepo.GetByID(ctx, userID)
	if err != nil {
		return nil, err
	}

	return dbUser.ToDomain(), nil
}

// GetUserByEmail retrieves a user by email
func (s *LocalAuthService) GetUserByEmail(ctx context.Context, email string) (*domain.User, error) {
	dbUser, err := s.userRepo.GetByEmail(ctx, email)
	if err != nil {
		return nil, err
	}

	return dbUser.ToDomain(), nil
}

// UserExists checks if a user with the given email exists
func (s *LocalAuthService) UserExists(ctx context.Context, email string) (bool, error) {
	return s.userRepo.EmailExists(ctx, email)
}

// ValidateToken validates a JWT token and returns the claims
func (s *LocalAuthService) ValidateToken(tokenString string) (*Claims, error) {
	return s.jwtService.ValidateToken(tokenString)
}

// GetUserInfo returns user info from a token (for OAuth /userinfo endpoint)
func (s *LocalAuthService) GetUserInfo(tokenString string) (map[string]interface{}, error) {
	return s.jwtService.GetUserInfoFromToken(tokenString)
}

// GetJWKS returns the JSON Web Key Set
func (s *LocalAuthService) GetJWKS() ([]byte, error) {
	return s.jwtService.MarshalJWKS()
}

// RefreshTokens generates new tokens from a refresh token
func (s *LocalAuthService) RefreshTokens(ctx context.Context, refreshToken string) (*TokenPair, error) {
	claims, err := s.jwtService.ValidateToken(refreshToken)
	if err != nil {
		return nil, err
	}

	user, err := s.GetUserByID(ctx, claims.Subject)
	if err != nil {
		return nil, err
	}

	return s.jwtService.GenerateTokens(user)
}
