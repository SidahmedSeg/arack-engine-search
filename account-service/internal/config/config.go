package config

import (
	"time"

	"github.com/kelseyhightower/envconfig"
)

type Config struct {
	Server   ServerConfig
	Redis    RedisConfig
	OAuth    OAuthConfig
	Session  SessionConfig
	Cookie   CookieConfig
	Zitadel  ZitadelConfig
	Stalwart StalwartConfig
}

type ZitadelConfig struct {
	APIBaseURL string `envconfig:"ZITADEL_API_URL" required:"true"`
	PAT        string `envconfig:"ZITADEL_PAT" required:"true"` // Personal Access Token for Management API
}

type StalwartConfig struct {
	BaseURL       string `envconfig:"STALWART_URL" required:"true"`
	AdminUser     string `envconfig:"STALWART_ADMIN_USER" required:"true"`
	AdminPassword string `envconfig:"STALWART_ADMIN_PASSWORD" required:"true"`
	DefaultQuota  int64  `envconfig:"STALWART_DEFAULT_QUOTA" default:"1073741824"` // 1GB
}

type ServerConfig struct {
	Host string `envconfig:"SERVER_HOST" default:"0.0.0.0"`
	Port int    `envconfig:"SERVER_PORT" default:"3002"`
}

type RedisConfig struct {
	URL string `envconfig:"REDIS_URL" default:"redis://localhost:6379"`
}

type OAuthConfig struct {
	IssuerURL   string   `envconfig:"ZITADEL_ISSUER_URL" required:"true"`
	ClientID    string   `envconfig:"ZITADEL_CLIENT_ID" required:"true"`
	RedirectURL string   `envconfig:"OAUTH_REDIRECT_URL" required:"true"`
	Scopes      []string `envconfig:"OAUTH_SCOPES" default:"openid,profile,email,offline_access"`
}

type SessionConfig struct {
	TTL              time.Duration `envconfig:"SESSION_TTL" default:"720h"`
	RefreshThreshold time.Duration `envconfig:"TOKEN_REFRESH_THRESHOLD" default:"5m"`
}

type CookieConfig struct {
	Name     string `envconfig:"COOKIE_NAME" default:"arack_session"`
	Domain   string `envconfig:"COOKIE_DOMAIN" default:".arack.io"`
	Secure   bool   `envconfig:"COOKIE_SECURE" default:"true"`
	HTTPOnly bool   `envconfig:"COOKIE_HTTP_ONLY" default:"true"`
	SameSite string `envconfig:"COOKIE_SAME_SITE" default:"lax"`
	MaxAge   int    `envconfig:"COOKIE_MAX_AGE" default:"2592000"`
}

func Load() (*Config, error) {
	var cfg Config
	if err := envconfig.Process("", &cfg); err != nil {
		return nil, err
	}
	return &cfg, nil
}
