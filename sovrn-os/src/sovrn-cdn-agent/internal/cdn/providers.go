package cdn

import (
	"log"
	"sync"

	"github.com/sovrn-os/sovrn-cdn-agent/internal/config"
)

// ProviderRegistry manages available CDN providers
type ProviderRegistry struct {
	providers map[string]Provider
	mu        sync.RWMutex
}

// NewProviderRegistry creates a new registry from config
func NewProviderRegistry(configs []config.ProviderConfig) *ProviderRegistry {
	reg := &ProviderRegistry{
		providers: make(map[string]Provider),
	}
	// v1: no actual CDN providers — stub implementation
	// Production would initialize Cloudflare R2, Backblaze B2, etc.
	log.Println("CDN provider registry initialized (no providers configured)")
	return reg
}

// Push queues content for CDN delivery
func (r *ProviderRegistry) Push(cid, localPath string, size int64, checksum string) {
	// v1: Store locally, CDN push is a stub
	log.Printf("CDN push queued: cid=%s path=%s size=%d", cid, localPath, size)
}

// List returns available provider names
func (r *ProviderRegistry) List() []map[string]interface{} {
	r.mu.RLock()
	defer r.mu.RUnlock()
	var providers []map[string]interface{}
	for name, p := range r.providers {
		providers = append(providers, map[string]interface{}{
			"id":      name,
			"name":    p.Name(),
			"enabled": p.IsEnabled(),
		})
	}
	return providers
}
