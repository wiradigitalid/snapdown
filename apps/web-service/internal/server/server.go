package server

import (
	"encoding/base64"
	"encoding/json"
	"fmt"
	"net/http"
	"strings"

	"github.com/go-chi/chi/v5"
	"github.com/wiradigitalid/snapdown/apps/web-service/internal/store"
)

type Server struct {
	store      *store.Store
	publishKey string
	router     chi.Router
}

type PublishRequest struct {
	Markdown string            `json:"markdown"`
	Files    []PublishFileItem `json:"files"`
}

type PublishFileItem struct {
	Filename   string `json:"filename"`
	DataBase64 string `json:"data_base64"`
}

func New(st *store.Store, publishKey string) *Server {
	s := &Server{
		store:      st,
		publishKey: publishKey,
	}

	r := chi.NewRouter()

	// Admin / Credential-Gated routes
	r.Group(func(admin chi.Router) {
		admin.Use(s.authMiddleware)
		admin.Put("/publish/{slug}", s.handlePublish)
		admin.Delete("/publish/{slug}", s.handleUnpublish)
		admin.Get("/publish/{slug}", s.handleReconcile)
	})

	// Public routes
	r.Get("/b/{slug}", s.handleGetBundle)
	r.Get("/b/{slug}/raw.md", s.handleGetRawMarkdown)
	r.Get("/b/{slug}/images/{filename}", s.handleGetImage)

	s.router = r
	return s
}

func (s *Server) Router() http.Handler {
	return s.router
}

func (s *Server) authMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		authHeader := r.Header.Get("Authorization")
		token := strings.TrimPrefix(authHeader, "Bearer ")
		token = strings.TrimPrefix(token, "bearer ")

		if s.publishKey != "" && token != s.publishKey {
			s.writeError(w, http.StatusUnauthorized, "unauthorized", "Valid publish key required")
			return
		}
		next.ServeHTTP(w, r)
	})
}

func (s *Server) handlePublish(w http.ResponseWriter, r *http.Request) {
	slug := chi.URLParam(r, "slug")
	if slug == "" {
		s.writeError(w, http.StatusBadRequest, "bad_request", "Missing slug")
		return
	}

	var req PublishRequest
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		s.writeError(w, http.StatusBadRequest, "bad_request", fmt.Sprintf("Invalid JSON: %v", err))
		return
	}

	fileMap := make(map[string][]byte)
	for _, f := range req.Files {
		bytes, err := base64.StdEncoding.DecodeString(f.DataBase64)
		if err != nil {
			s.writeError(w, http.StatusBadRequest, "bad_request", fmt.Sprintf("Invalid base64 in %s: %v", f.Filename, err))
			return
		}
		fileMap[f.Filename] = bytes
	}

	if err := s.store.Publish(slug, req.Markdown, fileMap); err != nil {
		s.writeError(w, http.StatusInternalServerError, "internal_error", err.Error())
		return
	}

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	_ = json.NewEncoder(w).Encode(map[string]string{"status": "published", "slug": slug})
}

func (s *Server) handleUnpublish(w http.ResponseWriter, r *http.Request) {
	slug := chi.URLParam(r, "slug")
	if err := s.store.Unpublish(slug); err != nil {
		s.writeError(w, http.StatusInternalServerError, "internal_error", err.Error())
		return
	}

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	_ = json.NewEncoder(w).Encode(map[string]string{"status": "unpublished", "slug": slug})
}

func (s *Server) handleReconcile(w http.ResponseWriter, r *http.Request) {
	slug := chi.URLParam(r, "slug")
	b, err := s.store.GetPublishedBundle(slug)
	if err != nil || b == nil {
		s.writeIdentical404(w)
		return
	}

	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	_ = json.NewEncoder(w).Encode(map[string]string{"slug": slug, "status": "served"})
}

func (s *Server) handleGetBundle(w http.ResponseWriter, r *http.Request) {
	slug := chi.URLParam(r, "slug")
	b, err := s.store.GetPublishedBundle(slug)
	if err != nil || b == nil {
		s.writeIdentical404(w)
		return
	}

	accept := r.Header.Get("Accept")
	if strings.Contains(accept, "text/markdown") || strings.Contains(accept, "text/plain") {
		w.Header().Set("Content-Type", "text/markdown; charset=utf-8")
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte(b.Markdown))
		return
	}

	// Default HTML render wrapper
	html := fmt.Sprintf(`<!DOCTYPE html>
<html>
<head><meta charset="utf-8"><title>Snapdown Review - %s</title></head>
<body><pre>%s</pre></body>
</html>`, slug, b.Markdown)

	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	w.WriteHeader(http.StatusOK)
	_, _ = w.Write([]byte(html))
}

func (s *Server) handleGetRawMarkdown(w http.ResponseWriter, r *http.Request) {
	slug := chi.URLParam(r, "slug")
	b, err := s.store.GetPublishedBundle(slug)
	if err != nil || b == nil {
		s.writeIdentical404(w)
		return
	}

	w.Header().Set("Content-Type", "text/markdown; charset=utf-8")
	w.WriteHeader(http.StatusOK)
	_, _ = w.Write([]byte(b.Markdown))
}

func (s *Server) handleGetImage(w http.ResponseWriter, r *http.Request) {
	slug := chi.URLParam(r, "slug")
	filename := chi.URLParam(r, "filename")

	if strings.Contains(filename, "..") || strings.Contains(filename, "/") || strings.Contains(filename, "\\") {
		s.writeError(w, http.StatusBadRequest, "bad_request", "Invalid filename")
		return
	}

	bytes, contentType, err := s.store.GetBlobBytes(slug, filename)
	if err != nil || bytes == nil {
		s.writeIdentical404(w)
		return
	}

	w.Header().Set("Content-Type", contentType)
	w.WriteHeader(http.StatusOK)
	_, _ = w.Write(bytes)
}

func (s *Server) writeIdentical404(w http.ResponseWriter) {
	// NFR-15 Invariant: Unknown, deleted, or revoked slugs return IDENTICAL 404 response
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusNotFound)
	_ = json.NewEncoder(w).Encode(map[string]interface{}{
		"error": map[string]string{
			"code":    "not_found",
			"message": "Publication not found or has been unpublished",
		},
	})
}

func (s *Server) writeError(w http.ResponseWriter, status int, code, msg string) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(map[string]interface{}{
		"error": map[string]string{
			"code":    code,
			"message": msg,
		},
	})
}
