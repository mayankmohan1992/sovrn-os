package config

import (
	"fmt"
	"os"

	"github.com/BurntSushi/toml"
)

// Config holds CDN agent configuration
type Config struct {
	SocketsDir    string           `toml:"sockets_dir"`
	DataDir       string           `toml:"data_dir"`
	CacheDir      string           `toml:"cache_dir"`
	MaxUploadMB   int              `toml:"max_upload_mb"`
	Providers     []ProviderConfig `toml:"providers"`
}

// ProviderConfig holds CDN provider configuration
type ProviderConfig struct {
	Name    string `toml:"name"`
	APIKey  string `toml:"api_key"`
	APIUrl  string `toml:"api_url"`
	Enabled bool   `toml:"enabled"`
}

// Load reads configuration from a TOML file
func Load(path string) (*Config, error) {
	cfg := &Config{
		SocketsDir:  "/var/lib/sovrn/sockets",
		DataDir:     "/var/lib/sovrn/cdn",
		CacheDir:    "/var/lib/sovrn/cdn/cache",
		MaxUploadMB: 100,
		Providers:   []ProviderConfig{},
	}

	data, err := os.ReadFile(path)
	if err != nil {
		return cfg, nil
	}

	if err := toml.Unmarshal(data, cfg); err != nil {
		return nil, fmt.Errorf("failed to parse config: %w", err)
	}

	return cfg, nil
}
