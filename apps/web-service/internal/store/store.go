package store

import (
	"database/sql"
	"fmt"
	"os"
	"path/filepath"
	"time"

	_ "modernc.org/sqlite"
)

type PublishedBundle struct {
	Slug        string
	Markdown    string
	BlobDir     string
	CreatedAt   string
	DeletedAt   sql.NullString
}

type PublishedBlob struct {
	ID          string
	Slug        string
	Filename    string
	ContentType string
	ByteSize    int64
}

type Store struct {
	db      *sql.DB
	dataDir string
}

func Open(dbPath string, dataDir string) (*Store, error) {
	if err := os.MkdirAll(filepath.Dir(dbPath), 0755); err != nil {
		return nil, fmt.Errorf("failed to create db directory: %w", err)
	}
	if err := os.MkdirAll(dataDir, 0755); err != nil {
		return nil, fmt.Errorf("failed to create data directory: %w", err)
	}

	db, err := sql.Open("sqlite", dbPath)
	if err != nil {
		return nil, fmt.Errorf("failed to open sqlite database: %w", err)
	}

	if _, err := db.Exec("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;"); err != nil {
		return nil, fmt.Errorf("failed to set pragmas: %w", err)
	}

	s := &Store{
		db:      db,
		dataDir: dataDir,
	}

	if err := s.migrate(); err != nil {
		return nil, fmt.Errorf("failed to run migrations: %w", err)
	}

	return s, nil
}

func (s *Store) migrate() error {
	ddl := `
	CREATE TABLE IF NOT EXISTS web_schema_version (
		version INTEGER PRIMARY KEY,
		applied_at TEXT NOT NULL
	);

	CREATE TABLE IF NOT EXISTS published_bundle (
		slug TEXT PRIMARY KEY,
		markdown TEXT NOT NULL,
		blob_dir TEXT NOT NULL,
		created_at TEXT NOT NULL,
		deleted_at TEXT
	);

	CREATE TABLE IF NOT EXISTS published_blob (
		id TEXT PRIMARY KEY,
		slug TEXT NOT NULL,
		filename TEXT NOT NULL,
		content_type TEXT NOT NULL,
		byte_size INTEGER NOT NULL,
		FOREIGN KEY(slug) REFERENCES published_bundle(slug) ON DELETE CASCADE
	);
	`
	_, err := s.db.Exec(ddl)
	return err
}

func (s *Store) Close() error {
	return s.db.Close()
}

func (s *Store) Publish(slug string, markdown string, files map[string][]byte) error {
	tx, err := s.db.Begin()
	if err != nil {
		return err
	}
	defer tx.Rollback()

	// 1. Stage files on disk
	slugDir := filepath.Join(s.dataDir, "blobs", slug)
	stagingDir := filepath.Join(s.dataDir, "staging", slug)

	if err := os.RemoveAll(stagingDir); err != nil {
		return err
	}
	if err := os.MkdirAll(stagingDir, 0755); err != nil {
		return err
	}

	for filename, bytes := range files {
		cleanFile := filepath.Base(filename)
		filePath := filepath.Join(stagingDir, cleanFile)
		if err := os.WriteFile(filePath, bytes, 0644); err != nil {
			return err
		}
	}

	// 2. Commit files to final directory atomically
	if err := os.RemoveAll(slugDir); err != nil {
		return err
	}
	if err := os.MkdirAll(filepath.Dir(slugDir), 0755); err != nil {
		return err
	}
	if err := os.Rename(stagingDir, slugDir); err != nil {
		// Fallback copy if rename across volumes fails
		if err := copyDir(stagingDir, slugDir); err != nil {
			return err
		}
		os.RemoveAll(stagingDir)
	}

	now := time.Now().UTC().Format(time.RFC3339)

	// 3. Upsert database record
	upsertQuery := `
	INSERT INTO published_bundle (slug, markdown, blob_dir, created_at, deleted_at)
	VALUES (?, ?, ?, ?, NULL)
	ON CONFLICT(slug) DO UPDATE SET
		markdown = excluded.markdown,
		blob_dir = excluded.blob_dir,
		deleted_at = NULL;
	`
	if _, err := tx.Exec(upsertQuery, slug, markdown, slugDir, now); err != nil {
		return err
	}

	// Remove old blobs
	if _, err := tx.Exec("DELETE FROM published_blob WHERE slug = ?;", slug); err != nil {
		return err
	}

	// Insert blob metadata
	blobQuery := `INSERT INTO published_blob (id, slug, filename, content_type, byte_size) VALUES (?, ?, ?, ?, ?);`
	for filename, bytes := range files {
		cleanFile := filepath.Base(filename)
		blobID := fmt.Sprintf("blob-%s-%s", slug, cleanFile)
		contentType := "image/png"
		if filepath.Ext(cleanFile) == ".webp" {
			contentType = "image/webp"
		}
		if _, err := tx.Exec(blobQuery, blobID, slug, cleanFile, contentType, len(bytes)); err != nil {
			return err
		}
	}

	return tx.Commit()
}

func (s *Store) GetPublishedBundle(slug string) (*PublishedBundle, error) {
	query := `SELECT slug, markdown, blob_dir, created_at, deleted_at FROM published_bundle WHERE slug = ? AND deleted_at IS NULL;`
	row := s.db.QueryRow(query, slug)

	var b PublishedBundle
	if err := row.Scan(&b.Slug, &b.Markdown, &b.BlobDir, &b.CreatedAt, &b.DeletedAt); err != nil {
		if err == sql.ErrNoRows {
			return nil, nil
		}
		return nil, err
	}
	return &b, nil
}

func (s *Store) GetBlobBytes(slug string, filename string) ([]byte, string, error) {
	b, err := s.GetPublishedBundle(slug)
	if err != nil || b == nil {
		return nil, "", fmt.Errorf("bundle not found")
	}

	cleanFile := filepath.Base(filename)
	filePath := filepath.Join(b.BlobDir, cleanFile)

	bytes, err := os.ReadFile(filePath)
	if err != nil {
		return nil, "", err
	}

	contentType := "image/png"
	if filepath.Ext(cleanFile) == ".webp" {
		contentType = "image/webp"
	}
	return bytes, contentType, nil
}

func (s *Store) Unpublish(slug string) error {
	tx, err := s.db.Begin()
	if err != nil {
		return err
	}
	defer tx.Rollback()

	now := time.Now().UTC().Format(time.RFC3339)
	if _, err := tx.Exec("UPDATE published_bundle SET deleted_at = ? WHERE slug = ?;", now, slug); err != nil {
		return err
	}

	// Delete from disk
	slugDir := filepath.Join(s.dataDir, "blobs", slug)
	_ = os.RemoveAll(slugDir)

	return tx.Commit()
}

func copyDir(src, dst string) error {
	entries, err := os.ReadDir(src)
	if err != nil {
		return err
	}
	if err := os.MkdirAll(dst, 0755); err != nil {
		return err
	}
	for _, entry := range entries {
		srcPath := filepath.Join(src, entry.Name())
		dstPath := filepath.Join(dst, entry.Name())
		if entry.IsDir() {
			if err := copyDir(srcPath, dstPath); err != nil {
				return err
			}
		} else {
			bytes, err := os.ReadFile(srcPath)
			if err != nil {
				return err
			}
			if err := os.WriteFile(dstPath, bytes, 0644); err != nil {
				return err
			}
		}
	}
	return nil
}
