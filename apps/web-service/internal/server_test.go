package server_test

import (
	"bytes"
	"encoding/base64"
	"encoding/json"
	"io"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"testing"

	"github.com/wiradigitalid/snapdown/apps/web-service/internal/server"
	"github.com/wiradigitalid/snapdown/apps/web-service/internal/store"
)

func TestWebServiceLifecycleAndPublicRoutes(t *testing.T) {
	tempDir, err := os.MkdirTemp("", "snapdown-web-test-*")
	if err != nil {
		t.Fatalf("failed to create temp dir: %v", err)
	}
	defer os.RemoveAll(tempDir)

	dbPath := filepath.Join(tempDir, "publication.db")
	st, err := store.Open(dbPath, tempDir)
	if err != nil {
		t.Fatalf("failed to open store: %v", err)
	}
	defer st.Close()

	secretKey := "test-publish-key-123"
	srv := server.New(st, secretKey)
	ts := httptest.NewServer(srv.Router())
	defer ts.Close()

	slug := "testslug42"
	markdown := "# Test Review\n\n- Finding 1"
	imageData := []byte("FAKE_PNG_IMAGE_BYTES")
	b64Image := base64.StdEncoding.EncodeToString(imageData)

	// 1. Publish without key -> 401 Unauthorized
	{
		reqBody, _ := json.Marshal(server.PublishRequest{
			Markdown: markdown,
			Files:    []server.PublishFileItem{},
		})
		req, _ := http.NewRequest(http.MethodPut, ts.URL+"/publish/"+slug, bytes.NewReader(reqBody))
		resp, err := http.DefaultClient.Do(req)
		if err != nil {
			t.Fatalf("failed request: %v", err)
		}
		if resp.StatusCode != http.StatusUnauthorized {
			t.Fatalf("expected 401, got %d", resp.StatusCode)
		}
	}

	// 2. Publish with key -> 200 OK
	{
		reqBody, _ := json.Marshal(server.PublishRequest{
			Markdown: markdown,
			Files: []server.PublishFileItem{
				{Filename: "img1.png", DataBase64: b64Image},
			},
		})
		req, _ := http.NewRequest(http.MethodPut, ts.URL+"/publish/"+slug, bytes.NewReader(reqBody))
		req.Header.Set("Authorization", "Bearer "+secretKey)
		resp, err := http.DefaultClient.Do(req)
		if err != nil {
			t.Fatalf("failed request: %v", err)
		}
		if resp.StatusCode != http.StatusOK {
			t.Fatalf("expected 200, got %d", resp.StatusCode)
		}
	}

	// 3. Query GET /b/{slug}/raw.md -> 200 verbatim Markdown
	{
		resp, err := http.Get(ts.URL + "/b/" + slug + "/raw.md")
		if err != nil {
			t.Fatalf("failed request: %v", err)
		}
		if resp.StatusCode != http.StatusOK {
			t.Fatalf("expected 200, got %d", resp.StatusCode)
		}
		body, _ := io.ReadAll(resp.Body)
		if string(body) != markdown {
			t.Fatalf("expected markdown %q, got %q", markdown, string(body))
		}
	}

	// 4. Query GET /b/{slug}/images/img1.png -> 200 image bytes
	{
		resp, err := http.Get(ts.URL + "/b/" + slug + "/images/img1.png")
		if err != nil {
			t.Fatalf("failed request: %v", err)
		}
		if resp.StatusCode != http.StatusOK {
			t.Fatalf("expected 200, got %d", resp.StatusCode)
		}
		body, _ := io.ReadAll(resp.Body)
		if !bytes.Equal(body, imageData) {
			t.Fatalf("expected image bytes %v, got %v", imageData, body)
		}
	}

	// 5. Query Traversal attempt -> 400 Bad Request
	{
		resp, err := http.Get(ts.URL + "/b/" + slug + "/images/..%2F..%2Fsecret.txt")
		if err != nil {
			t.Fatalf("failed request: %v", err)
		}
		if resp.StatusCode != http.StatusBadRequest {
			t.Fatalf("expected 400, got %d", resp.StatusCode)
		}
	}

	// 6. Reconcile endpoint GET /publish/{slug}
	{
		req, _ := http.NewRequest(http.MethodGet, ts.URL+"/publish/"+slug, nil)
		req.Header.Set("Authorization", "Bearer "+secretKey)
		resp, err := http.DefaultClient.Do(req)
		if err != nil {
			t.Fatalf("failed request: %v", err)
		}
		if resp.StatusCode != http.StatusOK {
			t.Fatalf("expected 200, got %d", resp.StatusCode)
		}
	}

	// 7. Unpublish DELETE /publish/{slug}
	{
		req, _ := http.NewRequest(http.MethodDelete, ts.URL+"/publish/"+slug, nil)
		req.Header.Set("Authorization", "Bearer "+secretKey)
		resp, err := http.DefaultClient.Do(req)
		if err != nil {
			t.Fatalf("failed request: %v", err)
		}
		if resp.StatusCode != http.StatusOK {
			t.Fatalf("expected 200, got %d", resp.StatusCode)
		}
	}

	// 8. Invariant NFR-15: Query unpublished slug returns 404
	{
		resp, err := http.Get(ts.URL + "/b/" + slug + "/raw.md")
		if err != nil {
			t.Fatalf("failed request: %v", err)
		}
		if resp.StatusCode != http.StatusNotFound {
			t.Fatalf("expected 404, got %d", resp.StatusCode)
		}
	}
}
