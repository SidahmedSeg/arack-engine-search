package db

import (
	"database/sql"
	"embed"
	"fmt"
	"io/fs"
	"sort"
	"strings"

	_ "github.com/lib/pq"
	"github.com/rs/zerolog/log"
)

//go:embed migrations/*.sql
var migrationsFS embed.FS

// Connect connects to the PostgreSQL database
func Connect(databaseURL string) (*sql.DB, error) {
	db, err := sql.Open("postgres", databaseURL)
	if err != nil {
		return nil, fmt.Errorf("open database: %w", err)
	}

	if err := db.Ping(); err != nil {
		return nil, fmt.Errorf("ping database: %w", err)
	}

	// Set connection pool settings
	db.SetMaxOpenConns(25)
	db.SetMaxIdleConns(5)

	return db, nil
}

// RunMigrations runs all SQL migrations
func RunMigrations(db *sql.DB) error {
	// Create migrations tracking table
	_, err := db.Exec(`
		CREATE TABLE IF NOT EXISTS _migrations (
			id SERIAL PRIMARY KEY,
			name VARCHAR(255) NOT NULL UNIQUE,
			applied_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
		)
	`)
	if err != nil {
		return fmt.Errorf("create migrations table: %w", err)
	}

	// Read migration files
	entries, err := fs.ReadDir(migrationsFS, "migrations")
	if err != nil {
		return fmt.Errorf("read migrations dir: %w", err)
	}

	// Sort by filename
	var migrationFiles []string
	for _, entry := range entries {
		if !entry.IsDir() && strings.HasSuffix(entry.Name(), ".sql") {
			migrationFiles = append(migrationFiles, entry.Name())
		}
	}
	sort.Strings(migrationFiles)

	// Run each migration
	for _, fileName := range migrationFiles {
		// Check if already applied
		var exists bool
		err := db.QueryRow("SELECT EXISTS(SELECT 1 FROM _migrations WHERE name = $1)", fileName).Scan(&exists)
		if err != nil {
			return fmt.Errorf("check migration %s: %w", fileName, err)
		}

		if exists {
			log.Debug().Str("migration", fileName).Msg("Migration already applied, skipping")
			continue
		}

		// Read migration file
		content, err := fs.ReadFile(migrationsFS, "migrations/"+fileName)
		if err != nil {
			return fmt.Errorf("read migration %s: %w", fileName, err)
		}

		// Execute migration
		_, err = db.Exec(string(content))
		if err != nil {
			return fmt.Errorf("execute migration %s: %w", fileName, err)
		}

		// Record migration
		_, err = db.Exec("INSERT INTO _migrations (name) VALUES ($1)", fileName)
		if err != nil {
			return fmt.Errorf("record migration %s: %w", fileName, err)
		}

		log.Info().Str("migration", fileName).Msg("Migration applied successfully")
	}

	return nil
}
