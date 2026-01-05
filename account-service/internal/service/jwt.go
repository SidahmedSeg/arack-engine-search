package service

import (
	"crypto/rand"
	"crypto/rsa"
	"crypto/x509"
	"encoding/base64"
	"encoding/json"
	"encoding/pem"
	"errors"
	"fmt"
	"math/big"
	"os"
	"time"

	"github.com/golang-jwt/jwt/v5"
	"github.com/google/uuid"

	"github.com/arack/account-service/internal/config"
	"github.com/arack/account-service/internal/domain"
)

var (
	ErrInvalidToken     = errors.New("invalid token")
	ErrExpiredToken     = errors.New("token expired")
	ErrInvalidSignature = errors.New("invalid signature")
)

// JWTService handles JWT token generation and validation
type JWTService struct {
	privateKey *rsa.PrivateKey
	publicKey  *rsa.PublicKey
	keyID      string
	issuer     string
	audience   string
	accessTTL  time.Duration
	refreshTTL time.Duration
}

// Claims represents the JWT claims
type Claims struct {
	jwt.RegisteredClaims
	Email string `json:"email,omitempty"`
	Name  string `json:"name,omitempty"`
}

// TokenPair represents access and refresh tokens
type TokenPair struct {
	AccessToken  string    `json:"access_token"`
	RefreshToken string    `json:"refresh_token"`
	TokenType    string    `json:"token_type"`
	ExpiresIn    int       `json:"expires_in"`
	ExpiresAt    time.Time `json:"expires_at"`
}

// JWKS represents a JSON Web Key Set
type JWKS struct {
	Keys []JWK `json:"keys"`
}

// JWK represents a JSON Web Key
type JWK struct {
	Kty string `json:"kty"`
	Use string `json:"use"`
	Kid string `json:"kid"`
	Alg string `json:"alg"`
	N   string `json:"n"`
	E   string `json:"e"`
}

// NewJWTService creates a new JWT service
func NewJWTService(cfg *config.JWTConfig) (*JWTService, error) {
	var privateKey *rsa.PrivateKey
	var publicKey *rsa.PublicKey

	// Try to load existing keys
	if _, err := os.Stat(cfg.PrivateKeyPath); err == nil {
		privateKey, err = loadPrivateKey(cfg.PrivateKeyPath)
		if err != nil {
			return nil, fmt.Errorf("load private key: %w", err)
		}
		publicKey = &privateKey.PublicKey
	} else {
		// Generate new key pair
		privateKey, err = rsa.GenerateKey(rand.Reader, 2048)
		if err != nil {
			return nil, fmt.Errorf("generate key: %w", err)
		}
		publicKey = &privateKey.PublicKey

		// Save keys to disk
		if err := savePrivateKey(cfg.PrivateKeyPath, privateKey); err != nil {
			return nil, fmt.Errorf("save private key: %w", err)
		}
		if err := savePublicKey(cfg.PublicKeyPath, publicKey); err != nil {
			return nil, fmt.Errorf("save public key: %w", err)
		}
	}

	// Generate a stable key ID based on the public key
	keyID := generateKeyID(publicKey)

	return &JWTService{
		privateKey: privateKey,
		publicKey:  publicKey,
		keyID:      keyID,
		issuer:     cfg.Issuer,
		audience:   cfg.Audience,
		accessTTL:  cfg.AccessTTL,
		refreshTTL: cfg.RefreshTTL,
	}, nil
}

// GenerateTokens creates a new access and refresh token pair for a user
func (s *JWTService) GenerateTokens(user *domain.User) (*TokenPair, error) {
	now := time.Now()
	accessExpiry := now.Add(s.accessTTL)
	refreshExpiry := now.Add(s.refreshTTL)

	// Access token claims
	accessClaims := Claims{
		RegisteredClaims: jwt.RegisteredClaims{
			Issuer:    s.issuer,
			Subject:   user.ID,
			Audience:  jwt.ClaimStrings{s.audience},
			ExpiresAt: jwt.NewNumericDate(accessExpiry),
			IssuedAt:  jwt.NewNumericDate(now),
			NotBefore: jwt.NewNumericDate(now),
			ID:        uuid.New().String(),
		},
		Email: user.Email,
		Name:  user.Name,
	}

	accessToken := jwt.NewWithClaims(jwt.SigningMethodRS256, accessClaims)
	accessToken.Header["kid"] = s.keyID

	accessTokenString, err := accessToken.SignedString(s.privateKey)
	if err != nil {
		return nil, fmt.Errorf("sign access token: %w", err)
	}

	// Refresh token claims (minimal)
	refreshClaims := jwt.RegisteredClaims{
		Issuer:    s.issuer,
		Subject:   user.ID,
		Audience:  jwt.ClaimStrings{s.audience},
		ExpiresAt: jwt.NewNumericDate(refreshExpiry),
		IssuedAt:  jwt.NewNumericDate(now),
		NotBefore: jwt.NewNumericDate(now),
		ID:        uuid.New().String(),
	}

	refreshToken := jwt.NewWithClaims(jwt.SigningMethodRS256, refreshClaims)
	refreshToken.Header["kid"] = s.keyID

	refreshTokenString, err := refreshToken.SignedString(s.privateKey)
	if err != nil {
		return nil, fmt.Errorf("sign refresh token: %w", err)
	}

	return &TokenPair{
		AccessToken:  accessTokenString,
		RefreshToken: refreshTokenString,
		TokenType:    "Bearer",
		ExpiresIn:    int(s.accessTTL.Seconds()),
		ExpiresAt:    accessExpiry,
	}, nil
}

// ValidateToken validates a JWT token and returns the claims
func (s *JWTService) ValidateToken(tokenString string) (*Claims, error) {
	token, err := jwt.ParseWithClaims(tokenString, &Claims{}, func(token *jwt.Token) (interface{}, error) {
		if _, ok := token.Method.(*jwt.SigningMethodRSA); !ok {
			return nil, fmt.Errorf("unexpected signing method: %v", token.Header["alg"])
		}
		return s.publicKey, nil
	})

	if err != nil {
		if errors.Is(err, jwt.ErrTokenExpired) {
			return nil, ErrExpiredToken
		}
		return nil, ErrInvalidToken
	}

	claims, ok := token.Claims.(*Claims)
	if !ok || !token.Valid {
		return nil, ErrInvalidToken
	}

	return claims, nil
}

// GetJWKS returns the JSON Web Key Set
func (s *JWTService) GetJWKS() *JWKS {
	return &JWKS{
		Keys: []JWK{
			{
				Kty: "RSA",
				Use: "sig",
				Kid: s.keyID,
				Alg: "RS256",
				N:   base64.RawURLEncoding.EncodeToString(s.publicKey.N.Bytes()),
				E:   base64.RawURLEncoding.EncodeToString(big.NewInt(int64(s.publicKey.E)).Bytes()),
			},
		},
	}
}

// GetUserInfoFromToken extracts user info from a token (for /userinfo endpoint)
func (s *JWTService) GetUserInfoFromToken(tokenString string) (map[string]interface{}, error) {
	claims, err := s.ValidateToken(tokenString)
	if err != nil {
		return nil, err
	}

	return map[string]interface{}{
		"sub":   claims.Subject,
		"email": claims.Email,
		"name":  claims.Name,
	}, nil
}

// Helper functions

func loadPrivateKey(path string) (*rsa.PrivateKey, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}

	block, _ := pem.Decode(data)
	if block == nil {
		return nil, errors.New("failed to decode PEM block")
	}

	return x509.ParsePKCS1PrivateKey(block.Bytes)
}

func savePrivateKey(path string, key *rsa.PrivateKey) error {
	// Create directory if it doesn't exist
	dir := path[:len(path)-len("/private.pem")]
	if err := os.MkdirAll(dir, 0700); err != nil {
		return err
	}

	data := pem.EncodeToMemory(&pem.Block{
		Type:  "RSA PRIVATE KEY",
		Bytes: x509.MarshalPKCS1PrivateKey(key),
	})

	return os.WriteFile(path, data, 0600)
}

func savePublicKey(path string, key *rsa.PublicKey) error {
	data := pem.EncodeToMemory(&pem.Block{
		Type:  "RSA PUBLIC KEY",
		Bytes: x509.MarshalPKCS1PublicKey(key),
	})

	return os.WriteFile(path, data, 0644)
}

func generateKeyID(key *rsa.PublicKey) string {
	// Use first 8 bytes of the modulus as key ID
	nBytes := key.N.Bytes()
	if len(nBytes) < 8 {
		return base64.RawURLEncoding.EncodeToString(nBytes)
	}
	return base64.RawURLEncoding.EncodeToString(nBytes[:8])
}

// MarshalJWKS returns the JWKS as JSON bytes
func (s *JWTService) MarshalJWKS() ([]byte, error) {
	return json.Marshal(s.GetJWKS())
}
