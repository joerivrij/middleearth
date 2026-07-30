package main

import (
	"context"
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestNormalizeURL(t *testing.T) {
	tests := []struct{ input, want string }{
		{"example.com", "https://example.com"},
		{" http://localhost:8080/path#part ", "http://localhost:8080/path"},
	}
	for _, test := range tests {
		got, err := normalizeURL(test.input)
		if err != nil {
			t.Fatalf("normalizeURL(%q): %v", test.input, err)
		}
		if got.String() != test.want {
			t.Errorf("normalizeURL(%q) = %q, want %q", test.input, got, test.want)
		}
	}
}

func TestNormalizeURLRejectsUnsupportedSchemes(t *testing.T) {
	if _, err := normalizeURL("file:///etc/passwd"); err == nil {
		t.Fatal("expected unsupported scheme error")
	}
}

func TestTraceURL(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) { _, _ = w.Write([]byte("hello")) }))
	defer server.Close()
	target, _ := normalizeURL(server.URL)
	result := traceURL(context.Background(), target)
	if result.Error != "" {
		t.Fatalf("trace failed: %s", result.Error)
	}
	if result.Status != "200 OK" || result.BytesRead != 5 {
		t.Fatalf("unexpected result: %+v", result)
	}
	if len(result.Events) == 0 {
		t.Fatal("expected trace events")
	}
}
