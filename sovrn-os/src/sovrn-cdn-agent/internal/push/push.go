package push

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"time"

	"github.com/sovrn-os/sovrn-cdn-agent/internal/meta"
)

// PushResult contains the result of a content push operation.
type PushResult struct {
	PushID    string            `json:"push_id"`
	CID       string            `json:"cid"`
	Size      int64             `json:"size"`
	Providers map[string]string `json:"providers"`
	Status    string            `json:"status"`
	PushedAt  time.Time         `json:"pushed_at"`
}

// Pusher handles pushing content to CDN providers.
type Pusher struct {
	cacheDir string
	providers map[string]Provider
}

// Provider is the interface for CDN push targets.
type Provider interface {
	Push(ctx context.Context, cid string, data []byte) (string, error)
	Name() string
}

// NewPusher creates a new content pusher.
func NewPusher(cacheDir string) *Pusher {
	return &Pusher{
		cacheDir: cacheDir,
		providers: map[string]Provider{
			"local": &LocalProvider{cacheDir: cacheDir},
		},
	}
}

// PushContent pushes a file to all configured CDN providers.
func (p *Pusher) PushContent(ctx context.Context, localPath string, providers []string) (*PushResult, error) {
	// Read and hash the file
	data, err := os.ReadFile(localPath)
	if err != nil {
		return nil, fmt.Errorf("reading %s: %w", localPath, err)
	}

	hash := sha256.Sum256(data)
	cid := hex.EncodeToString(hash[:])

	// Store in local cache
	cachePath := filepath.Join(p.cacheDir, "cache", cid)
	if err := os.MkdirAll(filepath.Dir(cachePath), 0755); err != nil {
		return nil, fmt.Errorf("creating cache dir: %w", err)
	}
	if err := os.WriteFile(cachePath, data, 0644); err != nil {
		return nil, fmt.Errorf("writing cache: %w", err)
	}

	// Push to providers
	result := &PushResult{
		PushID:    fmt.Sprintf("push-%d", time.Now().Unix()),
		CID:       cid,
		Size:      int64(len(data)),
		Providers: make(map[string]string),
		Status:    "complete",
		PushedAt:  time.Now(),
	}

	targetProviders := providers
	if len(targetProviders) == 0 {
		targetProviders = []string{"local"}
	}

	for _, name := range targetProviders {
		provider, ok := p.providers[name]
		if !ok {
			result.Providers[name] = "unknown provider"
			continue
		}

		ref, err := provider.Push(ctx, cid, data)
		if err != nil {
			result.Providers[name] = fmt.Sprintf("error: %v", err)
			continue
		}
		result.Providers[name] = ref
	}

	// Save metadata
	metaStore := meta.NewStore(filepath.Join(p.cacheDir, "meta"))
	if err := metaStore.SavePushResult(result); err != nil {
		// Non-fatal: metadata storage failure shouldn't block the push
		fmt.Fprintf(os.Stderr, "warning: failed to save push metadata: %v
", err)
	}

	return result, nil
}

// LocalProvider stores content in the local filesystem cache.
type LocalProvider struct {
	cacheDir string
}

func (l *LocalProvider) Push(ctx context.Context, cid string, data []byte) (string, error) {
	path := filepath.Join(l.cacheDir, "cache", cid)
	return path, nil
}

func (l *LocalProvider) Name() string { return "local" }
