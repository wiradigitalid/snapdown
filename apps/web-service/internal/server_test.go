package server_test

import (
	"bytes"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"html"
	"io"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/wiradigitalid/snapdown/apps/web-service/internal/server"
	"github.com/wiradigitalid/snapdown/apps/web-service/internal/store"
)

func setupTestServer(t *testing.T) (*httptest.Server, *store.Store, string, func()) {
	t.Helper()
	tempDir, err := os.MkdirTemp("", "snapdown-web-test-*")
	if err != nil {
		t.Fatalf("failed to create temp dir: %v", err)
	}

	dbPath := filepath.Join(tempDir, "publication.db")
	st, err := store.Open(dbPath, tempDir)
	if err != nil {
		os.RemoveAll(tempDir)
		t.Fatalf("failed to open store: %v", err)
	}

	secretKey := "test-publish-key-123"
	srv := server.New(st, secretKey)
	ts := httptest.NewServer(srv.Router())

	cleanup := func() {
		ts.Close()
		st.Close()
		os.RemoveAll(tempDir)
	}

	return ts, st, secretKey, cleanup
}

func TestWebServiceLifecycleAndPublicRoutes(t *testing.T) {
	ts, _, secretKey, cleanup := setupTestServer(t)
	defer cleanup()

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

func testNoteContainingMarkupIsEscaped(t *testing.T, ts *httptest.Server, secretKey string) {
	slug := "markup-escape-slug"
	hostileMarkdown := "# Note with Markup\n\n<script>alert('xss')</script>\n<b>bold finding</b>\n<img src=x onerror=alert(1)>"

	// Publish bundle
	reqBody, _ := json.Marshal(server.PublishRequest{
		Markdown: hostileMarkdown,
		Files:    []server.PublishFileItem{},
	})
	req, err := http.NewRequest(http.MethodPut, ts.URL+"/publish/"+slug, bytes.NewReader(reqBody))
	if err != nil {
		t.Fatalf("failed to create publish request: %v", err)
	}
	req.Header.Set("Authorization", "Bearer "+secretKey)
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		t.Fatalf("publish request failed: %v", err)
	}
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("expected status 200, got %d", resp.StatusCode)
	}

	// 1. Default HTML response: GET /b/{slug}
	{
		htmlResp, err := http.Get(ts.URL + "/b/" + slug)
		if err != nil {
			t.Fatalf("GET /b/%s failed: %v", slug, err)
		}
		defer htmlResp.Body.Close()

		if htmlResp.StatusCode != http.StatusOK {
			t.Fatalf("expected status 200, got %d", htmlResp.StatusCode)
		}
		contentType := htmlResp.Header.Get("Content-Type")
		if !strings.HasPrefix(contentType, "text/html") {
			t.Fatalf("expected text/html content type, got %q", contentType)
		}

		bodyBytes, err := io.ReadAll(htmlResp.Body)
		if err != nil {
			t.Fatalf("failed to read body: %v", err)
		}
		body := string(bodyBytes)

		// Assert behavior: hostile markup MUST NOT exist as unescaped DOM/markup elements
		if strings.Contains(body, "<script>") || strings.Contains(body, "</script>") {
			t.Errorf("rendered HTML body contains unescaped <script> tag: %s", body)
		}
		if strings.Contains(body, "<b>") || strings.Contains(body, "</b>") {
			t.Errorf("rendered HTML body contains unescaped <b> tag: %s", body)
		}
		if strings.Contains(body, "<img") {
			t.Errorf("rendered HTML body contains unescaped <img tag: %s", body)
		}

		// Assert behavior: the escaped entities are present in the text container
		if !strings.Contains(body, "&lt;script&gt;") || !strings.Contains(body, "&lt;/script&gt;") {
			t.Errorf("rendered HTML body is missing escaped script tags: %s", body)
		}
		if !strings.Contains(body, "&lt;b&gt;") || !strings.Contains(body, "&lt;/b&gt;") {
			t.Errorf("rendered HTML body is missing escaped bold tags: %s", body)
		}
		if !strings.Contains(body, "&lt;img") {
			t.Errorf("rendered HTML body is missing escaped img tag: %s", body)
		}
	}

	// 2. Raw endpoint GET /b/{slug}/raw.md MUST stay unescaped
	{
		rawResp, err := http.Get(ts.URL + "/b/" + slug + "/raw.md")
		if err != nil {
			t.Fatalf("GET /b/%s/raw.md failed: %v", slug, err)
		}
		defer rawResp.Body.Close()

		if rawResp.StatusCode != http.StatusOK {
			t.Fatalf("expected status 200, got %d", rawResp.StatusCode)
		}
		contentType := rawResp.Header.Get("Content-Type")
		if !strings.HasPrefix(contentType, "text/markdown") {
			t.Fatalf("expected text/markdown content type, got %q", contentType)
		}

		bodyBytes, err := io.ReadAll(rawResp.Body)
		if err != nil {
			t.Fatalf("failed to read body: %v", err)
		}
		if string(bodyBytes) != hostileMarkdown {
			t.Fatalf("raw markdown was modified: expected %q, got %q", hostileMarkdown, string(bodyBytes))
		}
	}

	// 3. Content negotiation Accept: text/markdown MUST stay unescaped
	{
		req, _ := http.NewRequest(http.MethodGet, ts.URL+"/b/"+slug, nil)
		req.Header.Set("Accept", "text/markdown")
		mdResp, err := http.DefaultClient.Do(req)
		if err != nil {
			t.Fatalf("GET /b/%s with Accept: text/markdown failed: %v", slug, err)
		}
		defer mdResp.Body.Close()

		if mdResp.StatusCode != http.StatusOK {
			t.Fatalf("expected status 200, got %d", mdResp.StatusCode)
		}
		bodyBytes, err := io.ReadAll(mdResp.Body)
		if err != nil {
			t.Fatalf("failed to read body: %v", err)
		}
		if string(bodyBytes) != hostileMarkdown {
			t.Fatalf("accept text/markdown was modified: expected %q, got %q", hostileMarkdown, string(bodyBytes))
		}
	}
}

func testNoteThatClosesPreBlockCannotReachBrowserAsMarkup(t *testing.T, ts *httptest.Server, secretKey string) {
	slug := "breakout-test-slug"
	breakoutMarkdown := "</pre><script>alert(1)</script><pre>"

	reqBody, _ := json.Marshal(server.PublishRequest{
		Markdown: breakoutMarkdown,
		Files:    []server.PublishFileItem{},
	})
	req, err := http.NewRequest(http.MethodPut, ts.URL+"/publish/"+slug, bytes.NewReader(reqBody))
	if err != nil {
		t.Fatalf("failed to create publish request: %v", err)
	}
	req.Header.Set("Authorization", "Bearer "+secretKey)
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		t.Fatalf("publish request failed: %v", err)
	}
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("expected status 200, got %d", resp.StatusCode)
	}

	htmlResp, err := http.Get(ts.URL + "/b/" + slug)
	if err != nil {
		t.Fatalf("GET /b/%s failed: %v", slug, err)
	}
	defer htmlResp.Body.Close()

	if htmlResp.StatusCode != http.StatusOK {
		t.Fatalf("expected status 200, got %d", htmlResp.StatusCode)
	}

	bodyBytes, err := io.ReadAll(htmlResp.Body)
	if err != nil {
		t.Fatalf("failed to read body: %v", err)
	}
	body := string(bodyBytes)

	// Assert behavior: only the single outer wrapper <pre> and </pre> tags exist
	preOpenCount := strings.Count(body, "<pre>")
	preCloseCount := strings.Count(body, "</pre>")
	if preOpenCount != 1 {
		t.Errorf("expected exactly 1 <pre> tag in document, got %d in body: %s", preOpenCount, body)
	}
	if preCloseCount != 1 {
		t.Errorf("expected exactly 1 </pre> tag in document, got %d in body: %s", preCloseCount, body)
	}

	// Assert behavior: breakout sequence </pre><script> is not present as markup
	if strings.Contains(body, "</pre><script>") {
		t.Errorf("pre breakout sequence reached browser unescaped: %s", body)
	}
	if !strings.Contains(body, "&lt;/pre&gt;&lt;script&gt;alert(1)&lt;/script&gt;&lt;pre&gt;") {
		t.Errorf("escaped pre breakout sequence not found in body: %s", body)
	}
}

func testSlugIsEscapedInRenderedPage(t *testing.T, ts *httptest.Server, secretKey string) {
	// Finding M6: Slug fixture hostile to HTML but valid on Windows filenames (no < > : " / \ | ? *)
	slug := "slug-with-meta-&-and-'"
	markdown := "# Simple Note"

	reqBody, _ := json.Marshal(server.PublishRequest{
		Markdown: markdown,
		Files:    []server.PublishFileItem{},
	})
	req, err := http.NewRequest(http.MethodPut, ts.URL+"/publish/"+slug, bytes.NewReader(reqBody))
	if err != nil {
		t.Fatalf("failed to create publish request: %v", err)
	}
	req.Header.Set("Authorization", "Bearer "+secretKey)
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		t.Fatalf("publish request failed: %v", err)
	}
	if resp.StatusCode != http.StatusOK {
		t.Fatalf("expected status 200, got %d", resp.StatusCode)
	}

	htmlResp, err := http.Get(ts.URL + "/b/" + slug)
	if err != nil {
		t.Fatalf("GET /b/%s failed: %v", slug, err)
	}
	defer htmlResp.Body.Close()

	if htmlResp.StatusCode != http.StatusOK {
		t.Fatalf("expected status 200, got %d", htmlResp.StatusCode)
	}

	bodyBytes, err := io.ReadAll(htmlResp.Body)
	if err != nil {
		t.Fatalf("failed to read body: %v", err)
	}
	body := string(bodyBytes)

	// Assert behavior: <title> does not contain raw unescaped '&' or '\'' from slug
	expectedTitle := fmt.Sprintf("<title>Snapdown Review - %s</title>", html.EscapeString(slug))
	if !strings.Contains(body, expectedTitle) {
		t.Errorf("expected escaped title %q in HTML body, got body:\n%s", expectedTitle, body)
	}
	if strings.Contains(body, "<title>Snapdown Review - slug-with-meta-&-and-'</title>") {
		t.Errorf("un-escaped slug appeared in <title> element: %s", body)
	}
}

func testUnknownSlugStillReturnsIdenticalRefusal(t *testing.T, ts *httptest.Server, secretKey string) {
	unknownSlug := "non-existent-unknown-slug-999"
	revokedSlug := "revoked-slug-to-unpublish-456"

	// Publish and then unpublish revokedSlug
	{
		reqBody, _ := json.Marshal(server.PublishRequest{
			Markdown: "# Temporary",
			Files:    []server.PublishFileItem{},
		})
		req, _ := http.NewRequest(http.MethodPut, ts.URL+"/publish/"+revokedSlug, bytes.NewReader(reqBody))
		req.Header.Set("Authorization", "Bearer "+secretKey)
		resp, err := http.DefaultClient.Do(req)
		if err != nil || resp.StatusCode != http.StatusOK {
			t.Fatalf("failed to publish temporary bundle: %v", err)
		}

		delReq, _ := http.NewRequest(http.MethodDelete, ts.URL+"/publish/"+revokedSlug, nil)
		delReq.Header.Set("Authorization", "Bearer "+secretKey)
		delResp, err := http.DefaultClient.Do(delReq)
		if err != nil || delResp.StatusCode != http.StatusOK {
			t.Fatalf("failed to unpublish bundle: %v", err)
		}
	}

	type errorResponse struct {
		Error struct {
			Code    string `json:"code"`
			Message string `json:"message"`
		} `json:"error"`
	}

	verifyRefusal := func(url string, authHeader string) []byte {
		req, err := http.NewRequest(http.MethodGet, url, nil)
		if err != nil {
			t.Fatalf("failed to create request: %v", err)
		}
		if authHeader != "" {
			req.Header.Set("Authorization", authHeader)
		}
		resp, err := http.DefaultClient.Do(req)
		if err != nil {
			t.Fatalf("request to %s failed: %v", url, err)
		}
		defer resp.Body.Close()

		if resp.StatusCode != http.StatusNotFound {
			t.Fatalf("expected status 404 from %s, got %d", url, resp.StatusCode)
		}
		contentType := resp.Header.Get("Content-Type")
		if !strings.HasPrefix(contentType, "application/json") {
			t.Fatalf("expected application/json content type from %s, got %q", url, contentType)
		}

		bodyBytes, err := io.ReadAll(resp.Body)
		if err != nil {
			t.Fatalf("failed to read response body from %s: %v", url, err)
		}

		var errResp errorResponse
		if err := json.Unmarshal(bodyBytes, &errResp); err != nil {
			t.Fatalf("failed to parse json error response from %s: %v", url, err)
		}

		if errResp.Error.Code != "not_found" {
			t.Errorf("expected error code 'not_found' from %s, got %q", url, errResp.Error.Code)
		}
		if errResp.Error.Message != "Publication not found or has been unpublished" {
			t.Errorf("expected standard NFR-15 refusal message from %s, got %q", url, errResp.Error.Message)
		}

		return bodyBytes
	}

	// Verify 404 responses for unknown and revoked slug paths
	unknownHTMLBody := verifyRefusal(ts.URL+"/b/"+unknownSlug, "")
	unknownRawBody := verifyRefusal(ts.URL+"/b/"+unknownSlug+"/raw.md", "")
	revokedHTMLBody := verifyRefusal(ts.URL+"/b/"+revokedSlug, "")
	revokedRawBody := verifyRefusal(ts.URL+"/b/"+revokedSlug+"/raw.md", "")
	revokedReconcileBody := verifyRefusal(ts.URL+"/publish/"+revokedSlug, "Bearer "+secretKey)

	// Assert NFR-15 invariant: identical response bytes across all refusal paths
	if !bytes.Equal(unknownHTMLBody, revokedHTMLBody) {
		t.Errorf("unknown slug HTML refusal does not match revoked slug refusal:\nunknown: %s\nrevoked: %s",
			string(unknownHTMLBody), string(revokedHTMLBody))
	}
	if !bytes.Equal(unknownHTMLBody, unknownRawBody) {
		t.Errorf("unknown slug HTML refusal does not match unknown slug raw refusal:\nhtml: %s\nraw: %s",
			string(unknownHTMLBody), string(unknownRawBody))
	}
	if !bytes.Equal(unknownHTMLBody, revokedRawBody) {
		t.Errorf("unknown slug HTML refusal does not match revoked slug raw refusal:\nhtml: %s\nraw: %s",
			string(unknownHTMLBody), string(revokedRawBody))
	}
	if !bytes.Equal(unknownHTMLBody, revokedReconcileBody) {
		t.Errorf("unknown slug HTML refusal does not match revoked reconcile refusal:\nhtml: %s\nreconcile: %s",
			string(unknownHTMLBody), string(revokedReconcileBody))
	}
}

// Top-level suite registering t.Run matching registry names exactly
func TestW7S2MarkdownAndSlugEscaping(t *testing.T) {
	ts, _, secretKey, cleanup := setupTestServer(t)
	defer cleanup()

	t.Run("a_note_containing_markup_is_escaped_in_the_rendered_page", func(t *testing.T) {
		testNoteContainingMarkupIsEscaped(t, ts, secretKey)
	})

	t.Run("a_note_that_closes_the_pre_block_cannot_reach_the_browser_as_markup", func(t *testing.T) {
		testNoteThatClosesPreBlockCannotReachBrowserAsMarkup(t, ts, secretKey)
	})

	t.Run("the_slug_is_escaped_in_the_rendered_page", func(t *testing.T) {
		testSlugIsEscapedInRenderedPage(t, ts, secretKey)
	})

	t.Run("an_unknown_slug_still_returns_the_identical_refusal", func(t *testing.T) {
		testUnknownSlugStillReturnsIdenticalRefusal(t, ts, secretKey)
	})
}

// Explicit top-level test functions matching registry names so go test -run matches directly
func Test_a_note_containing_markup_is_escaped_in_the_rendered_page(t *testing.T) {
	ts, _, secretKey, cleanup := setupTestServer(t)
	defer cleanup()
	testNoteContainingMarkupIsEscaped(t, ts, secretKey)
}

func Test_a_note_that_closes_the_pre_block_cannot_reach_the_browser_as_markup(t *testing.T) {
	ts, _, secretKey, cleanup := setupTestServer(t)
	defer cleanup()
	testNoteThatClosesPreBlockCannotReachBrowserAsMarkup(t, ts, secretKey)
}

func Test_the_slug_is_escaped_in_the_rendered_page(t *testing.T) {
	ts, _, secretKey, cleanup := setupTestServer(t)
	defer cleanup()
	testSlugIsEscapedInRenderedPage(t, ts, secretKey)
}

func Test_an_unknown_slug_still_returns_the_identical_refusal(t *testing.T) {
	ts, _, secretKey, cleanup := setupTestServer(t)
	defer cleanup()
	testUnknownSlugStillReturnsIdenticalRefusal(t, ts, secretKey)
}
