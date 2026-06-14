package api

import (
	"encoding/json"
	"io"
	"log"
	"net"

	"github.com/sovrn-os/sovrn-cdn-agent/internal/cdn"
	"github.com/sovrn-os/sovrn-cdn-agent/internal/cache"
	"github.com/sovrn-os/sovrn-cdn-agent/internal/config"
)

// Handler processes JSON-RPC requests on the Unix socket
type Handler struct {
	cfg    *config.Config
	cache  *cache.Cache
	providers *cdn.ProviderRegistry
}

// NewHandler creates a new API handler
func NewHandler(cfg *config.Config) *Handler {
	return &Handler{
		cfg:       cfg,
		cache:     cache.New(cfg.CacheDir),
		providers: cdn.NewProviderRegistry(cfg.Providers),
	}
}

// HandleConnection processes a single Unix socket connection
func (h *Handler) HandleConnection(conn net.Conn) {
	defer conn.Close()

	buf := make([]byte, 65536)
	for {
		n, err := conn.Read(buf)
		if err != nil {
			if err != io.EOF {
				log.Printf("Read error: %v", err)
			}
			return
		}

		var req struct {
			JSONRPC string          `json:"jsonrpc"`
			Method  string          `json:"method"`
			Params  json.RawMessage `json:"params"`
			ID      json.RawMessage `json:"id"`
		}

		if err := json.Unmarshal(buf[:n], &req); err != nil {
			h.sendError(conn, -32700, "Parse error", nil)
			continue
		}

		result := h.dispatch(req.Method, req.Params)
		h.sendResult(conn, result, req.ID)
	}
}

func (h *Handler) dispatch(method string, params json.RawMessage) interface{} {
	switch method {
	case "cdn.push":
		return h.handlePush(params)
	case "cdn.get_status":
		return h.handleGetStatus(params)
	case "cdn.get_providers":
		return h.handleGetProviders()
	case "cdn.subscribe":
		return h.handleSubscribe(params)
	case "cdn.health":
		return h.handleHealth()
	default:
		return map[string]interface{}{
			"error": map[string]interface{}{
				"code":    -32601,
				"message": "Method not found: " + method,
			},
		}
	}
}

func (h *Handler) handlePush(params json.RawMessage) interface{} {
	var p struct {
		CID      string `json:"cid"`
		LocalPath string `json:"local_path"`
		Size     int64  `json:"size"`
		Checksum string `json:"checksum"`
	}
	if err := json.Unmarshal(params, &p); err != nil {
		return map[string]interface{}{"error": map[string]interface{}{"code": -32602, "message": "Invalid params"}}
	}

	pushID := "push_" + p.CID
	// Queue content for CDN push
	go h.providers.Push(p.CID, p.LocalPath, p.Size, p.Checksum)

	return map[string]interface{}{
		"push_id": pushID,
		"status":  "queued",
	}
}

func (h *Handler) handleGetStatus(params json.RawMessage) interface{} {
	var p struct {
		PushID string `json:"push_id"`
	}
	json.Unmarshal(params, &p)
	return map[string]interface{}{
		"push_id":   p.PushID,
		"status":    "completed",
		"providers": h.providers.List(),
	}
}

func (h *Handler) handleGetProviders() interface{} {
	return map[string]interface{}{
		"providers": h.providers.List(),
	}
}

func (h *Handler) handleSubscribe(params json.RawMessage) interface{} {
	var p struct {
		ProviderID    string `json:"provider_id"`
		Plan          string `json:"plan"`
		PaymentMethod string `json:"payment_method"`
	}
	json.Unmarshal(params, &p)
	return map[string]interface{}{
		"subscription": map[string]interface{}{
			"id":       "sub_001",
			"provider": p.ProviderID,
			"plan":     p.Plan,
			"status":   "active",
		},
	}
}

func (h *Handler) handleHealth() interface{} {
	return map[string]interface{}{
		"status":      "ok",
		"queue_depth": 0,
		"uptime":      0,
	}
}

func (h *Handler) sendResult(conn net.Conn, result interface{}, id json.RawMessage) {
	resp := map[string]interface{}{
		"jsonrpc": "2.0",
		"result":  result,
		"id":      id,
	}
	data, _ := json.Marshal(resp)
	conn.Write(append(data, '\n'))
}

func (h *Handler) sendError(conn net.Conn, code int, message string, id json.RawMessage) {
	resp := map[string]interface{}{
		"jsonrpc": "2.0",
		"error":   map[string]interface{}{"code": code, "message": message},
		"id":      id,
	}
	data, _ := json.Marshal(resp)
	conn.Write(append(data, '\n'))
}
