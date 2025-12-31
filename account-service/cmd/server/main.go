package main

import (
	"context"
	"fmt"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
	"github.com/go-chi/cors"
	"github.com/redis/go-redis/v9"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"

	"github.com/arack/account-service/internal/config"
	"github.com/arack/account-service/internal/handler"
	redisRepo "github.com/arack/account-service/internal/repository/redis"
	"github.com/arack/account-service/internal/service"
)

func main() {
	// Setup logging
	zerolog.TimeFieldFormat = zerolog.TimeFormatUnix
	log.Logger = log.Output(zerolog.ConsoleWriter{Out: os.Stderr, TimeFormat: time.RFC3339})

	// Load config
	cfg, err := config.Load()
	if err != nil {
		log.Fatal().Err(err).Msg("Failed to load config")
	}

	log.Info().
		Str("host", cfg.Server.Host).
		Int("port", cfg.Server.Port).
		Str("issuer", cfg.OAuth.IssuerURL).
		Msg("Configuration loaded")

	ctx := context.Background()

	// Initialize Redis
	redisOpt, err := redis.ParseURL(cfg.Redis.URL)
	if err != nil {
		log.Fatal().Err(err).Msg("Failed to parse Redis URL")
	}
	redisClient := redis.NewClient(redisOpt)

	if err := redisClient.Ping(ctx).Err(); err != nil {
		log.Fatal().Err(err).Msg("Failed to connect to Redis")
	}
	log.Info().Str("url", cfg.Redis.URL).Msg("Connected to Redis")

	// Initialize auth service (OAuth/OIDC)
	authService, err := service.NewAuthService(ctx, &cfg.OAuth)
	if err != nil {
		log.Fatal().Err(err).Msg("Failed to create auth service")
	}
	log.Info().Msg("Auth service initialized")

	// Initialize Zitadel service (Management API + Session API)
	zitadelService := service.NewZitadelService(&cfg.Zitadel)
	log.Info().Msg("Zitadel service initialized")

	// Initialize Stalwart service (Email provisioning)
	stalwartService := service.NewStalwartService(&cfg.Stalwart)
	log.Info().Msg("Stalwart service initialized")

	// Initialize session service
	sessionRepo := redisRepo.NewSessionRepository(redisClient)
	sessionService := service.NewSessionService(sessionRepo, authService, &cfg.Session)

	// Initialize handler
	h := handler.New(sessionService, authService, zitadelService, stalwartService, &cfg.Cookie)

	// Setup router
	r := chi.NewRouter()

	// Middleware
	r.Use(middleware.RequestID)
	r.Use(middleware.RealIP)
	r.Use(middleware.Logger)
	r.Use(middleware.Recoverer)
	r.Use(middleware.Timeout(30 * time.Second))

	// CORS - Allow all arack.io subdomains
	r.Use(cors.Handler(cors.Options{
		AllowedOrigins:   []string{"https://arack.io", "https://mail.arack.io", "https://admin.arack.io", "http://localhost:5000", "http://localhost:5001"},
		AllowedMethods:   []string{"GET", "POST", "OPTIONS"},
		AllowedHeaders:   []string{"Content-Type", "Authorization"},
		AllowCredentials: true,
		MaxAge:           3600,
	}))

	// Mount routes
	r.Mount("/", h.Routes())

	// Start server
	addr := fmt.Sprintf("%s:%d", cfg.Server.Host, cfg.Server.Port)
	server := &http.Server{
		Addr:         addr,
		Handler:      r,
		ReadTimeout:  15 * time.Second,
		WriteTimeout: 15 * time.Second,
		IdleTimeout:  60 * time.Second,
	}

	// Graceful shutdown
	go func() {
		log.Info().Str("addr", addr).Msg("Account service starting")
		if err := server.ListenAndServe(); err != http.ErrServerClosed {
			log.Fatal().Err(err).Msg("Server failed")
		}
	}()

	quit := make(chan os.Signal, 1)
	signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)
	<-quit

	log.Info().Msg("Shutting down server...")

	shutdownCtx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	if err := server.Shutdown(shutdownCtx); err != nil {
		log.Fatal().Err(err).Msg("Server forced to shutdown")
	}

	// Close Redis
	if err := redisClient.Close(); err != nil {
		log.Warn().Err(err).Msg("Failed to close Redis connection")
	}

	log.Info().Msg("Server stopped")
}
