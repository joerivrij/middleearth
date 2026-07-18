package main

import (
	"context"
	"crypto/tls"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"log"
	"net"
	"net/http"
	"net/http/httptrace"
	"net/url"
	"os"
	"strings"
	"time"
)

const (
	defaultAddress = "127.0.0.1:8080"
	requestTimeout = 15 * time.Second
	maxBodyBytes   = 1 << 20
)

type event struct {
	Name        string `json:"name"`
	Protocol    string `json:"protocol"`
	OSILayer    int    `json:"osi_layer"`
	OSIName     string `json:"osi_name"`
	Explanation string `json:"explanation"`
	AtMS        int64  `json:"at_ms"`
	Detail      string `json:"detail,omitempty"`
	Duration    int64  `json:"duration_ms,omitempty"`
	Facts       []fact `json:"facts,omitempty"`
}

type fact struct {
	Label string `json:"label"`
	Value string `json:"value"`
}

type traceResult struct {
	URL           string  `json:"url"`
	Method        string  `json:"method"`
	Status        string  `json:"status,omitempty"`
	Protocol      string  `json:"protocol,omitempty"`
	RemoteAddress string  `json:"remote_address,omitempty"`
	BytesRead     int64   `json:"bytes_read"`
	TotalMS       int64   `json:"total_ms"`
	Events        []event `json:"events"`
	Error         string  `json:"error,omitempty"`
}

type traceRequest struct {
	URL string `json:"url"`
}

func main() {
	address := os.Getenv("GREY_HAVENS_ADDR")
	if address == "" {
		address = defaultAddress
	}

	mux := http.NewServeMux()
	mux.HandleFunc("GET /", serveIndex)
	mux.HandleFunc("POST /api/trace", handleTrace)
	mux.HandleFunc("GET /healthz", func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusNoContent)
	})

	server := &http.Server{
		Addr:              address,
		Handler:           requestLogger(mux),
		ReadHeaderTimeout: 5 * time.Second,
		IdleTimeout:       30 * time.Second,
	}

	log.Printf("Grey Havens is watching the water at http://%s", address)
	log.Fatal(server.ListenAndServe())
}

func serveIndex(w http.ResponseWriter, r *http.Request) {
	if r.URL.Path != "/" {
		http.NotFound(w, r)
		return
	}
	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	_, _ = io.WriteString(w, indexHTML)
}

func handleTrace(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	var input traceRequest
	decoder := json.NewDecoder(io.LimitReader(r.Body, 8<<10))
	decoder.DisallowUnknownFields()
	if err := decoder.Decode(&input); err != nil {
		writeJSONError(w, http.StatusBadRequest, "expected JSON containing a url")
		return
	}

	target, err := normalizeURL(input.URL)
	if err != nil {
		writeJSONError(w, http.StatusBadRequest, err.Error())
		return
	}

	result := traceURL(r.Context(), target)
	for _, step := range result.Events {
		log.Printf("trace %-15s +%4dms %s", step.Name, step.AtMS, step.Detail)
	}
	if result.Error != "" {
		log.Printf("trace failed: %s", result.Error)
	} else {
		log.Printf("trace complete: %s via %s in %dms (%d bytes)", result.Status, result.Protocol, result.TotalMS, result.BytesRead)
	}
	_ = json.NewEncoder(w).Encode(result)
}

func normalizeURL(raw string) (*url.URL, error) {
	raw = strings.TrimSpace(raw)
	if raw == "" {
		return nil, errors.New("url is required")
	}
	if !strings.Contains(raw, "://") {
		raw = "https://" + raw
	}
	target, err := url.Parse(raw)
	if err != nil || target.Hostname() == "" {
		return nil, errors.New("enter a valid URL")
	}
	if target.Scheme != "http" && target.Scheme != "https" {
		return nil, errors.New("only http and https URLs are supported")
	}
	target.Fragment = ""
	return target, nil
}

func traceURL(parent context.Context, target *url.URL) traceResult {
	started := time.Now()
	result := traceResult{URL: target.String(), Method: http.MethodGet}
	add := func(name, protocol string, osiLayer int, osiName, explanation, detail string, duration time.Duration, facts ...fact) {
		result.Events = append(result.Events, event{
			Name: name, Protocol: protocol, OSILayer: osiLayer, OSIName: osiName, Explanation: explanation,
			AtMS: time.Since(started).Milliseconds(), Detail: detail,
			Duration: duration.Milliseconds(), Facts: facts,
		})
	}
	port := target.Port()
	if port == "" {
		if target.Scheme == "https" {
			port = "443"
		} else {
			port = "80"
		}
	}
	path := target.EscapedPath()
	if path == "" {
		path = "/"
	}
	if target.RawQuery != "" {
		path += "?" + target.RawQuery
	}
	add("URL parsed", "URL", 7, "Application", "The URL is application-level information. It is split into the pieces needed by the layers below: the scheme selects encryption and a default port, while the hostname will be resolved by DNS.", target.String(), 0,
		fact{"Scheme", target.Scheme}, fact{"Host", target.Hostname()}, fact{"Port", port}, fact{"Request target", path})

	var dnsStarted, connectStarted, tlsStarted time.Time
	trace := &httptrace.ClientTrace{
		DNSStart: func(info httptrace.DNSStartInfo) {
			dnsStarted = time.Now()
			add("DNS query", "DNS", 7, "Application", "DNS is an application-layer protocol. It translates the human-friendly hostname into IP addresses that Layer 3 can route toward. The operating system resolver performs this lookup.", info.Host, 0,
				fact{"Question", info.Host}, fact{"Looking for", "A (IPv4) and AAAA (IPv6) addresses"})
		},
		DNSDone: func(info httptrace.DNSDoneInfo) {
			addresses := make([]string, 0, len(info.Addrs))
			for _, address := range info.Addrs {
				addresses = append(addresses, address.String())
			}
			detail := strings.Join(addresses, ", ")
			if info.Err != nil {
				detail = info.Err.Error()
			}
			add("DNS answer", "DNS", 7, "Application", "The application-layer DNS answer returned candidate IP addresses. Go will pass one to the lower layers and attempt connections until one succeeds.", detail, time.Since(dnsStarted),
				fact{"Addresses", detail}, fact{"From shared cache", fmt.Sprintf("%t", info.Coalesced)})
		},
		ConnectStart: func(network, address string) {
			connectStarted = time.Now()
			add("TCP handshake", "TCP", 4, "Transport", "TCP lives at the transport layer. The client starts the three-way SYN → SYN-ACK → ACK handshake, creating a reliable end-to-end byte stream between two ports.", network+" "+address, 0,
				fact{"Network", network}, fact{"Destination", address})
		},
		ConnectDone: func(network, address string, err error) {
			detail := network + " " + address
			if err != nil {
				detail = err.Error()
			} else {
				result.RemoteAddress = address
			}
			add("TCP connected", "TCP", 4, "Transport", "The Layer 4 connection is ready. TCP ports identify the applications, while sequence numbers, acknowledgements, retransmission, and flow control are handled by the operating system.", detail, time.Since(connectStarted),
				fact{"Remote socket", detail})
		},
		GotConn: func(info httptrace.GotConnInfo) {
			add("Connection chosen", "TCP", 4, "Transport", "The HTTP client selected a Layer 4 connection for this request. Connections can be reused to avoid repeating DNS, TCP, and TLS setup.", info.Conn.RemoteAddr().String(), 0,
				fact{"Remote", info.Conn.RemoteAddr().String()}, fact{"Local", info.Conn.LocalAddr().String()}, fact{"Reused", fmt.Sprintf("%t", info.Reused)}, fact{"Idle time", info.IdleTime.String()})
		},
		TLSHandshakeStart: func() {
			tlsStarted = time.Now()
			add("TLS handshake", "TLS", 6, "Presentation", "TLS is commonly mapped to the presentation layer because it transforms application data through encryption. Client and server negotiate a version and cipher, authenticate the certificate, and derive session keys.", "sending ClientHello", 0,
				fact{"Server name", target.Hostname()})
		},
		TLSHandshakeDone: func(state tls.ConnectionState, err error) {
			if err != nil {
				add("TLS failed", "TLS", 6, "Presentation", "The Layer 6 encrypted session could not be established or the server identity could not be verified.", err.Error(), time.Since(tlsStarted))
				return
			}
			facts := []fact{{"Version", tls.VersionName(state.Version)}, {"Cipher suite", tls.CipherSuiteName(state.CipherSuite)}, {"ALPN protocol", valueOr(state.NegotiatedProtocol, "not negotiated")}, {"Session resumed", fmt.Sprintf("%t", state.DidResume)}}
			if len(state.PeerCertificates) > 0 {
				certificate := state.PeerCertificates[0]
				facts = append(facts, fact{"Certificate subject", certificate.Subject.String()}, fact{"Certificate issuer", certificate.Issuer.String()}, fact{"Certificate expires", certificate.NotAfter.Format(time.RFC3339)}, fact{"DNS names", strings.Join(certificate.DNSNames, ", ")})
			}
			add("TLS ready", "TLS", 6, "Presentation", "The presentation-layer security transformation is ready: the server identity was verified and all following HTTP bytes are encrypted before reaching TCP.", tls.VersionName(state.Version)+" · "+tls.CipherSuiteName(state.CipherSuite), time.Since(tlsStarted), facts...)
		},
		WroteHeaders: func() {
			add("Headers written", "HTTP", 7, "Application", "HTTP is the application-layer conversation. Its request line and headers are serialized, transformed by TLS for HTTPS, and then divided into TCP segments by Layer 4.", http.MethodGet+" "+path, 0,
				fact{"Method", http.MethodGet}, fact{"Host header", target.Host}, fact{"User-Agent", "grey-havens/0.1"}, fact{"Body", "none"})
		},
		WroteRequest: func(info httptrace.WroteRequestInfo) {
			detail := "request headers sent"
			if info.Err != nil {
				detail = info.Err.Error()
			}
			add("Request sent", "HTTP", 7, "Application", "The complete Layer 7 request has been handed down the stack. It now waits for the remote application to process it and build an HTTP response.", detail, 0)
		},
		GotFirstResponseByte: func() {
			add("First response byte", "HTTP", 7, "Application", "The first application-layer response byte marks time-to-first-byte (TTFB): lower-layer setup, network travel, server queueing, and server work have all happened by now.", "response began arriving", time.Since(started))
		},
	}

	ctx, cancel := context.WithTimeout(httptrace.WithClientTrace(parent, trace), requestTimeout)
	defer cancel()
	req, _ := http.NewRequestWithContext(ctx, http.MethodGet, target.String(), nil)
	req.Header.Set("User-Agent", "grey-havens/0.1")

	client := &http.Client{
		Transport:     &http.Transport{DialContext: (&net.Dialer{Timeout: 5 * time.Second}).DialContext},
		CheckRedirect: func(_ *http.Request, _ []*http.Request) error { return http.ErrUseLastResponse },
	}
	response, err := client.Do(req)
	if err != nil {
		result.Error = err.Error()
		result.TotalMS = time.Since(started).Milliseconds()
		return result
	}
	defer response.Body.Close()
	result.Status = response.Status
	result.Protocol = response.Proto
	responseFacts := []fact{{"Status", response.Status}, {"Protocol", response.Proto}, {"Content length", formatContentLength(response.ContentLength)}}
	for _, name := range []string{"Content-Type", "Content-Encoding", "Cache-Control", "Server", "Location"} {
		if value := response.Header.Get(name); value != "" {
			responseFacts = append(responseFacts, fact{name, value})
		}
	}
	add("Response headers", "HTTP", 7, "Application", "At Layer 7, the status describes the outcome while response headers describe the body, caching policy, server, and—when present—a redirect destination.", response.Status, 0, responseFacts...)
	readStarted := time.Now()
	read, readErr := io.Copy(io.Discard, io.LimitReader(response.Body, maxBodyBytes))
	result.BytesRead = read
	if readErr != nil {
		result.Error = fmt.Sprintf("reading response: %v", readErr)
	}
	add("Response body", "HTTP", 7, "Application", "Go consumes the Layer 7 response body while lower layers decrypt records, order TCP bytes, route packets, deliver frames, and transmit signals. The demo discards content after measuring it.", fmt.Sprintf("%d bytes read (capped at 1 MiB)", read), time.Since(readStarted),
		fact{"Bytes read", fmt.Sprintf("%d", read)}, fact{"Read limit", "1,048,576 bytes"})
	result.TotalMS = time.Since(started).Milliseconds()
	return result
}

func valueOr(value, fallback string) string {
	if value == "" {
		return fallback
	}
	return value
}

func formatContentLength(length int64) string {
	if length < 0 {
		return "unknown / streamed"
	}
	return fmt.Sprintf("%d bytes", length)
}

func requestLogger(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		log.Printf("%s %s from %s", r.Method, r.URL.Path, r.RemoteAddr)
		next.ServeHTTP(w, r)
	})
}

func writeJSONError(w http.ResponseWriter, status int, message string) {
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(map[string]string{"error": message})
}
