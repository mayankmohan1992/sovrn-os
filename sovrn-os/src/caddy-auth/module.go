// Package caddyauth provides a Caddy v2 authentication module for Sovrn OS.
package caddyauth

import (
	"fmt"
	"net/http"
	"strings"

	"github.com/caddyserver/caddy/v2"
	"github.com/caddyserver/caddy/v2/caddyconfig/httpcaddyfile"
	"github.com/caddyserver/caddy/v2/modules/caddyhttp"
	"go.uber.org/zap"
)

func init() {
	caddy.RegisterModule(SovrnAuth{})
	httpcaddyfile.RegisterHandlerDirective("sovrn_auth", parseCaddyfile)
}

// SovrnAuth implements caddyhttp.MiddlewareHandler for JWT + policy-based auth.
type SovrnAuth struct {
	JWTPublicKeyFile string `json:"jwt_public_key_file,omitempty"`
	JWTSecret        string `json:"jwt_secret,omitempty"`
	JWTAlgorithm     string `json:"jwt_algorithm,omitempty"`
	SovrndSocket     string `json:"sovrnd_socket,omitempty"`
	SkipPaths        []string `json:"skip_paths,omitempty"`

	logger *zap.Logger
}

// CaddyModule returns the Caddy module info.
func (SovrnAuth) CaddyModule() caddy.ModuleInfo {
	return caddy.ModuleInfo{
		ID:  "http.handlers.sovrn_auth",
		New: func() caddy.Module { return new(SovrnAuth) },
	}
}

// Provision sets up the handler.
func (s *SovrnAuth) Provision(ctx caddy.Context) error {
	s.logger = ctx.Logger(s)
	if s.JWTAlgorithm == "" {
		s.JWTAlgorithm = "HS256"
	}
	if s.SovrndSocket == "" {
		s.SovrndSocket = "/var/lib/sovrn/sockets/sovrnd.sock"
	}
	return nil
}

// Validate implements caddy.Validator.
func (s *SovrnAuth) Validate() error {
	if s.JWTSecret == "" && s.JWTPublicKeyFile == "" {
		return fmt.Errorf("either jwt_secret or jwt_public_key_file must be set")
	}
	return nil
}

// ServeHTTP implements caddyhttp.MiddlewareHandler.
func (s SovrnAuth) ServeHTTP(w http.ResponseWriter, r *http.Request, next caddyhttp.Handler) error {
	// Skip auth for configured paths
	for _, path := range s.SkipPaths {
		if strings.HasPrefix(r.URL.Path, path) {
			return next.ServeHTTP(w, r)
		}
	}

	// Skip auth for .sovrn TLD static content
	if strings.HasSuffix(r.URL.Hostname(), ".sovrn") &&
		(r.Method == "GET" || r.Method == "HEAD") {
		// Static .sovrn content is public
		return next.ServeHTTP(w, r)
	}

	// Extract JWT from Authorization header
	authHeader := r.Header.Get("Authorization")
	if authHeader == "" {
		// Also check cookie
		if cookie, err := r.Cookie("sovrn_token"); err == nil {
			authHeader = "Bearer " + cookie.Value
		}
	}

	if authHeader == "" {
		w.Header().Set("WWW-Authenticate", `Bearer realm="sovrn"`)
		http.Error(w, "Unauthorized", http.StatusUnauthorized)
		return nil
	}

	tokenStr := strings.TrimPrefix(authHeader, "Bearer ")
	if tokenStr == authHeader {
		http.Error(w, "Invalid authorization header", http.StatusUnauthorized)
		return nil
	}

	// Validate JWT (simplified — production needs proper JWT library)
	claims, err := validateJWT(tokenStr, s.JWTSecret, s.JWTAlgorithm)
	if err != nil {
		s.logger.Debug("JWT validation failed", zap.Error(err))
		http.Error(w, "Invalid token", http.StatusUnauthorized)
		return nil
	}

	// Set user info in headers for downstream services
	r.Header.Set("X-Sovrn-User", claims["sub"].(string))
	if domain, ok := claims["domain"].(string); ok && domain != "" {
		r.Header.Set("X-Sovrn-Domain", domain)
	}

	// Check permissions via sovrn-auth service
	resource, action := pathToPermission(r.URL.Path, r.Method)
	if resource != "" && action != "" {
		if err := s.checkPermission(claims["sub"].(string), resource, action); err != nil {
			s.logger.Debug("Permission denied", zap.String("resource", resource), zap.String("action", action))
			http.Error(w, "Forbidden", http.StatusForbidden)
			return nil
		}
	}

	return next.ServeHTTP(w, r)
}

// validateJWT validates a JWT token (simplified — use proper library in production)
func validateJWT(tokenStr, secret, algorithm string) (map[string]interface{}, error) {
	// v1: Simplified validation using HMAC-SHA256
	parts := strings.Split(tokenStr, ".")
	if len(parts) != 3 {
		return nil, fmt.Errorf("invalid token format")
	}

	// In production, use github.com/golang-jwt/jwt/v5
	// For v1, we proxy validation to sovrnd
	return map[string]interface{}{
		"sub": "", // Will be populated by proper JWT parsing
	}, fmt.Errorf("JWT validation not yet implemented — use sovrnd proxy")
}

// pathToPermission converts an HTTP path+method to a resource:action pair
func pathToPermission(path, method string) (string, string) {
	parts := strings.Split(strings.TrimPrefix(path, "/api/"), "/")

	var resource string
	if len(parts) > 0 {
		resource = parts[0]
	}

	var action string
	switch method {
	case "GET", "HEAD":
		action = "read"
	case "POST", "PUT":
		action = "write"
	case "DELETE":
		action = "delete"
	default:
		action = "write"
	}

	return resource, action
}

// checkPermission checks with the sovrn-auth service
func (s *SovrnAuth) checkPermission(user, resource, action string) error {
	// For v1, all authenticated users have basic access
	// Production: proxy to sovrn-auth service
	return nil
}

// parseCaddyfile parses the Caddyfile directive
func parseCaddyfile(h httpcaddyfile.Helper) (caddyhttp.MiddlewareHandler, error) {
	var auth SovrnAuth
	for h.Next() {
		for h.NextBlock(0) {
			switch h.Val() {
			case "jwt_secret":
				if !h.NextArg() {
					return nil, h.ArgErr()
				}
				auth.JWTSecret = h.Val()
			case "jwt_public_key_file":
				if !h.NextArg() {
					return nil, h.ArgErr()
				}
				auth.JWTPublicKeyFile = h.Val()
			case "jwt_algorithm":
				if !h.NextArg() {
					return nil, h.ArgErr()
				}
				auth.JWTAlgorithm = h.Val()
			case "skip_paths":
				for h.NextArg() {
					auth.SkipPaths = append(auth.SkipPaths, h.Val())
				}
			default:
				return nil, h.Errf("unrecognized directive: %s", h.Val())
			}
		}
	}
	return auth, nil
}

// Interface guards
var (
	_ caddy.Provisioner     = (*SovrnAuth)(nil)
	_ caddy.Validator       = (*SovrnAuth)(nil)
	_ caddyhttp.MiddlewareHandler = (*SovrnAuth)(nil)
)
